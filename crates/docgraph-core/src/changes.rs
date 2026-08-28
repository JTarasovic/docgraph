use crate::{CanonicalCorpus, GraphIndex, GraphNode, RelationOrigin, RepositoryConfig};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagedChange {
    EntityAdded(String),
    EntityRemoved(String),
    EntityMoved(String),
    StateChanged(String),
    PropertiesChanged(String),
    RelationsChanged,
    SectionsChanged,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManagedChangeReport {
    pub changes: Vec<ManagedChange>,
    pub diagnostics: Vec<ChangeDiagnostic>,
}

impl ManagedChangeReport {
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SemanticSection {
    pub document: String,
    pub heading: String,
    pub level: u8,
    pub parent: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct SemanticRelation {
    pub source: String,
    pub predicate: String,
    pub target: String,
    pub properties: BTreeMap<String, String>,
    pub origin: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SemanticChange {
    EntityAdded {
        entity: String,
        entity_type: String,
        path: String,
    },
    EntityRemoved {
        entity: String,
        entity_type: String,
        path: String,
    },
    EntityMoved {
        entity: String,
        before: String,
        after: String,
    },
    EntityTypeChanged {
        entity: String,
        before: String,
        after: String,
    },
    WorkflowStateChanged {
        entity: String,
        before: Option<String>,
        after: Option<String>,
    },
    PropertyChanged {
        entity: String,
        property: String,
        before: Option<String>,
        after: Option<String>,
    },
    SectionAdded {
        section: String,
        after: SemanticSection,
    },
    SectionRemoved {
        section: String,
        before: SemanticSection,
    },
    SectionChanged {
        section: String,
        before: SemanticSection,
        after: SemanticSection,
    },
    RelationAdded {
        relation: SemanticRelation,
    },
    RelationRemoved {
        relation: SemanticRelation,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticChangeReport {
    pub changes: Vec<SemanticChange>,
    pub diagnostics: Vec<ChangeDiagnostic>,
}

impl SemanticChangeReport {
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub struct SemanticChangeReviewer;

impl SemanticChangeReviewer {
    pub fn review(
        base: &CanonicalCorpus,
        candidate: &CanonicalCorpus,
        config: &RepositoryConfig,
    ) -> SemanticChangeReport {
        let base_graph = GraphIndex::build(base, config);
        let candidate_graph = GraphIndex::build(candidate, config);
        let diagnostics = ManagedChangeValidator::validate(base, candidate, config).diagnostics;
        let mut changes = Vec::new();
        review_entities(&base_graph, &candidate_graph, &mut changes);
        review_sections(&base_graph, &candidate_graph, &mut changes);
        review_relations(&base_graph, &candidate_graph, &mut changes);
        SemanticChangeReport {
            changes,
            diagnostics,
        }
    }
}

pub struct ManagedChangeValidator;

impl ManagedChangeValidator {
    pub fn validate(
        base: &CanonicalCorpus,
        candidate: &CanonicalCorpus,
        config: &RepositoryConfig,
    ) -> ManagedChangeReport {
        let base_graph = GraphIndex::build(base, config);
        let candidate_graph = GraphIndex::build(candidate, config);
        let mut report = ManagedChangeReport::default();
        let base_entities: BTreeMap<_, _> = base_graph
            .entities
            .iter()
            .map(|entity| (entity.id.as_str(), entity))
            .collect();
        let candidate_entities: BTreeMap<_, _> = candidate_graph
            .entities
            .iter()
            .map(|entity| (entity.id.as_str(), entity))
            .collect();

        for (id, base_entity) in &base_entities {
            let base_path = base_graph.documents[base_entity.document].path.clone();
            let Some(candidate_entity) = candidate_entities.get(id) else {
                report
                    .changes
                    .push(ManagedChange::EntityRemoved((*id).to_owned()));
                continue;
            };
            let candidate_path = candidate_graph.documents[candidate_entity.document]
                .path
                .clone();
            if base_entity.entity_type != candidate_entity.entity_type {
                report.diagnostics.push(ChangeDiagnostic {
                    code: "unsupported-entity-type-change",
                    message: format!(
                        "entity {id:?} changed type from {:?} to {:?}",
                        base_entity.entity_type, candidate_entity.entity_type
                    ),
                    path: candidate_path.clone(),
                });
            }
            if base_path != candidate_path {
                report
                    .changes
                    .push(ManagedChange::EntityMoved((*id).to_owned()));
            }
            if base_entity.state != candidate_entity.state {
                if !supported_state_change(base_entity, candidate_entity, config) {
                    report.diagnostics.push(ChangeDiagnostic {
                        code: "unsupported-workflow-state-change",
                        message: format!(
                            "entity {id:?} changed state from {:?} to {:?} without a configured transition",
                            base_entity.state, candidate_entity.state
                        ),
                        path: candidate_path,
                    });
                }
                report
                    .changes
                    .push(ManagedChange::StateChanged((*id).to_owned()));
            }
            if property_key(&base_entity.properties) != property_key(&candidate_entity.properties) {
                report
                    .changes
                    .push(ManagedChange::PropertiesChanged((*id).to_owned()));
            }
        }

        for (id, entity) in &candidate_entities {
            if base_entities.contains_key(id) {
                continue;
            }
            let path = candidate_graph.documents[entity.document].path.clone();
            if let Some(entity_config) = config.entities.get(&entity.entity_type)
                && let Some(workflow_name) = &entity_config.workflow
                && let Some(workflow) = config.workflows.get(workflow_name)
            {
                let supported = entity
                    .state
                    .as_deref()
                    .is_some_and(|state| workflow_reaches(workflow, &workflow.initial, state));
                if !supported {
                    report.diagnostics.push(ChangeDiagnostic {
                        code: "unsupported-adoption-state",
                        message: format!(
                            "new entity {id:?} state {:?} is not reachable from workflow {workflow_name:?} initial state {:?}",
                            entity.state, workflow.initial
                        ),
                        path,
                    });
                }
            }
            report
                .changes
                .push(ManagedChange::EntityAdded((*id).to_owned()));
        }

        if explicit_relations(&base_graph) != explicit_relations(&candidate_graph) {
            report.changes.push(ManagedChange::RelationsChanged);
        }
        let base_unresolved = unresolved_explicit_relations(&base_graph);
        for (key, path) in unresolved_explicit_relations(&candidate_graph) {
            if !base_unresolved.contains_key(&key) {
                report.diagnostics.push(ChangeDiagnostic {
                    code: "unsupported-dangling-managed-reference",
                    message: "a managed relation became unresolved".to_owned(),
                    path,
                });
            }
        }
        validate_section_identity(&base_graph, &candidate_graph, &mut report);
        report
    }
}

fn review_entities(base: &GraphIndex, candidate: &GraphIndex, changes: &mut Vec<SemanticChange>) {
    let base_entities: BTreeMap<_, _> = base
        .entities
        .iter()
        .map(|entity| (entity.id.as_str(), entity))
        .collect();
    let candidate_entities: BTreeMap<_, _> = candidate
        .entities
        .iter()
        .map(|entity| (entity.id.as_str(), entity))
        .collect();
    for (id, entity) in &base_entities {
        let path = path_reference(&base.documents[entity.document].path);
        let Some(after) = candidate_entities.get(id) else {
            changes.push(SemanticChange::EntityRemoved {
                entity: (*id).to_owned(),
                entity_type: entity.entity_type.clone(),
                path,
            });
            continue;
        };
        let after_path = path_reference(&candidate.documents[after.document].path);
        if path != after_path {
            changes.push(SemanticChange::EntityMoved {
                entity: (*id).to_owned(),
                before: path,
                after: after_path,
            });
        }
        if entity.entity_type != after.entity_type {
            changes.push(SemanticChange::EntityTypeChanged {
                entity: (*id).to_owned(),
                before: entity.entity_type.clone(),
                after: after.entity_type.clone(),
            });
        }
        if entity.state != after.state {
            changes.push(SemanticChange::WorkflowStateChanged {
                entity: (*id).to_owned(),
                before: entity.state.clone(),
                after: after.state.clone(),
            });
        }
        let property_names: BTreeSet<_> = entity
            .properties
            .keys()
            .chain(after.properties.keys())
            .collect();
        for property in property_names {
            let before = entity.properties.get(property).map(semantic_value);
            let after = after.properties.get(property).map(semantic_value);
            if before != after {
                changes.push(SemanticChange::PropertyChanged {
                    entity: (*id).to_owned(),
                    property: property.clone(),
                    before,
                    after,
                });
            }
        }
    }
    for (id, entity) in candidate_entities {
        if !base_entities.contains_key(id) {
            changes.push(SemanticChange::EntityAdded {
                entity: id.to_owned(),
                entity_type: entity.entity_type.clone(),
                path: path_reference(&candidate.documents[entity.document].path),
            });
        }
    }
}

fn review_sections(base: &GraphIndex, candidate: &GraphIndex, changes: &mut Vec<SemanticChange>) {
    let base_sections = semantic_sections(base);
    let candidate_sections = semantic_sections(candidate);
    for (id, before) in &base_sections {
        match candidate_sections.get(id) {
            None => changes.push(SemanticChange::SectionRemoved {
                section: id.clone(),
                before: before.clone(),
            }),
            Some(after) if before != after => changes.push(SemanticChange::SectionChanged {
                section: id.clone(),
                before: before.clone(),
                after: after.clone(),
            }),
            Some(_) => {}
        }
    }
    for (id, after) in candidate_sections {
        if !base_sections.contains_key(&id) {
            changes.push(SemanticChange::SectionAdded { section: id, after });
        }
    }
}

fn semantic_sections(graph: &GraphIndex) -> BTreeMap<String, SemanticSection> {
    graph
        .sections
        .iter()
        .enumerate()
        .filter_map(|(index, section)| {
            let reference = section_reference(graph, index)?;
            Some((
                reference,
                SemanticSection {
                    document: document_reference(graph, section.document),
                    heading: section.heading.clone(),
                    level: section.level,
                    parent: section
                        .parent
                        .and_then(|parent| section_reference(graph, parent)),
                },
            ))
        })
        .collect()
}

fn review_relations(base: &GraphIndex, candidate: &GraphIndex, changes: &mut Vec<SemanticChange>) {
    let base_relations = semantic_relations(base);
    let candidate_relations = semantic_relations(candidate);
    for relation in base_relations.difference(&candidate_relations) {
        changes.push(SemanticChange::RelationRemoved {
            relation: relation.clone(),
        });
    }
    for relation in candidate_relations.difference(&base_relations) {
        changes.push(SemanticChange::RelationAdded {
            relation: relation.clone(),
        });
    }
}

fn semantic_relations(graph: &GraphIndex) -> BTreeSet<SemanticRelation> {
    graph
        .relations
        .iter()
        .map(|relation| SemanticRelation {
            source: semantic_node_reference(graph, &relation.source),
            predicate: relation.predicate.clone(),
            target: semantic_node_reference(graph, &relation.target),
            properties: relation
                .properties
                .iter()
                .map(|(name, value)| (name.clone(), semantic_value(value)))
                .collect(),
            origin: match relation.origin {
                RelationOrigin::Explicit => "managed",
                RelationOrigin::MarkdownLink => "markdown",
            }
            .to_owned(),
        })
        .collect()
}

fn semantic_node_reference(graph: &GraphIndex, node: &GraphNode) -> String {
    match node {
        GraphNode::Document(document) => document_reference(graph, *document),
        GraphNode::Entity(entity) => entity.clone(),
        GraphNode::Section(section) => {
            section_reference(graph, *section).unwrap_or_else(|| format!("section-index:{section}"))
        }
        GraphNode::ExternalUri(uri) => uri.clone(),
        GraphNode::Unresolved(reference) => reference.clone(),
    }
}

fn document_reference(graph: &GraphIndex, document: usize) -> String {
    graph.documents[document]
        .entity
        .clone()
        .unwrap_or_else(|| path_reference(&graph.documents[document].path))
}

fn path_reference(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn section_reference(graph: &GraphIndex, section: usize) -> Option<String> {
    let section = graph.sections.get(section)?;
    let id = section.id.as_ref()?;
    Some(format!(
        "{}#{}",
        document_reference(graph, section.document),
        id.as_str()
    ))
}

fn unresolved_explicit_relations(graph: &GraphIndex) -> BTreeMap<String, PathBuf> {
    graph
        .relations
        .iter()
        .filter(|relation| relation.origin == RelationOrigin::Explicit)
        .filter(|relation| {
            matches!(relation.source, GraphNode::Unresolved(_))
                || matches!(relation.target, GraphNode::Unresolved(_))
        })
        .map(|relation| {
            (
                format!(
                    "{}\0{}\0{}",
                    relation.location.path.display(),
                    relation.predicate,
                    node_key(graph, &relation.target)
                ),
                relation.location.path.clone(),
            )
        })
        .collect()
}

fn supported_state_change(
    base: &crate::EntityNode,
    candidate: &crate::EntityNode,
    config: &RepositoryConfig,
) -> bool {
    let Some(entity) = config.entities.get(&base.entity_type) else {
        return false;
    };
    let Some(workflow_name) = &entity.workflow else {
        return false;
    };
    let Some(workflow) = config.workflows.get(workflow_name) else {
        return false;
    };
    match (base.state.as_deref(), candidate.state.as_deref()) {
        (None, Some(to)) => to == workflow.initial,
        (Some(from), Some(to)) => workflow_reaches(workflow, from, to),
        _ => false,
    }
}

fn workflow_reaches(workflow: &crate::WorkflowConfig, from: &str, to: &str) -> bool {
    let mut pending = vec![from];
    let mut visited = BTreeSet::new();
    while let Some(state) = pending.pop() {
        if state == to {
            return true;
        }
        if visited.insert(state)
            && let Some(config) = workflow.states.get(state)
        {
            pending.extend(config.transitions.iter().map(String::as_str));
        }
    }
    false
}

fn explicit_relations(graph: &GraphIndex) -> BTreeSet<String> {
    graph
        .relations
        .iter()
        .filter(|relation| relation.origin == RelationOrigin::Explicit)
        .map(|relation| {
            format!(
                "{}\0{}\0{}\0{:?}",
                node_key(graph, &relation.source),
                relation.predicate,
                node_key(graph, &relation.target),
                relation.properties
            )
        })
        .collect()
}

fn property_key(properties: &BTreeMap<String, toml_edit::Value>) -> Vec<(String, String)> {
    properties
        .iter()
        .map(|(name, value)| (name.clone(), semantic_value(value)))
        .collect()
}

fn semantic_value(value: &toml_edit::Value) -> String {
    if let Some(value) = value.as_str() {
        format!("{value:?}")
    } else if let Some(value) = value.as_integer() {
        value.to_string()
    } else if let Some(value) = value.as_float() {
        value.to_string()
    } else if let Some(value) = value.as_bool() {
        value.to_string()
    } else if let Some(value) = value.as_datetime() {
        value.to_string()
    } else if let Some(value) = value.as_array() {
        format!(
            "[{}]",
            value
                .iter()
                .map(semantic_value)
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        value.to_string()
    }
}

fn node_key(graph: &GraphIndex, node: &GraphNode) -> String {
    match node {
        GraphNode::Document(index) => {
            format!("document:{}", graph.documents[*index].path.display())
        }
        GraphNode::Entity(id) => format!("entity:{id}"),
        GraphNode::Section(index) => graph.sections[*index].id.as_ref().map_or_else(
            || format!("section-index:{index}"),
            |id| format!("section:{id}"),
        ),
        GraphNode::ExternalUri(uri) => format!("external:{uri}"),
        GraphNode::Unresolved(reference) => format!("unresolved:{reference}"),
    }
}

fn validate_section_identity(
    base_graph: &GraphIndex,
    candidate_graph: &GraphIndex,
    report: &mut ManagedChangeReport,
) {
    let candidate_ids: BTreeSet<_> = candidate_graph
        .sections
        .iter()
        .filter_map(|section| section.id.as_ref().map(|id| id.as_str()))
        .collect();
    let base_ids: BTreeSet<_> = base_graph
        .sections
        .iter()
        .filter_map(|section| section.id.as_ref().map(|id| id.as_str()))
        .collect();
    if base_ids != candidate_ids {
        report.changes.push(ManagedChange::SectionsChanged);
    }

    for section in &base_graph.sections {
        let Some(id) = &section.id else { continue };
        if candidate_ids.contains(id.as_str()) {
            continue;
        }
        let base_document = &base_graph.documents[section.document];
        let candidate_document = base_document
            .entity
            .as_ref()
            .and_then(|entity| {
                candidate_graph
                    .documents
                    .iter()
                    .position(|document| document.entity.as_ref() == Some(entity))
            })
            .or_else(|| {
                candidate_graph
                    .documents
                    .iter()
                    .position(|document| document.path == base_document.path)
            });
        let Some(candidate_document) = candidate_document else {
            continue;
        };
        if candidate_graph.sections.iter().any(|candidate_section| {
            candidate_section.document == candidate_document
                && candidate_section.level == section.level
                && candidate_section.heading == section.heading
                && candidate_section.id.as_ref() != Some(id)
        }) {
            let path = candidate_graph.documents[candidate_document].path.clone();
            report.diagnostics.push(ChangeDiagnostic {
                code: "unsupported-section-id-change",
                message: format!(
                    "stable section ID {id} was replaced on the surviving heading {:?}",
                    section.heading
                ),
                path,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Repository, RepositoryConfig};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf, Repository, RepositoryConfig);

    impl Fixture {
        fn new() -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "docgraph-change-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(root.join(".git")).unwrap();
            fs::create_dir_all(root.join(".docgraph")).unwrap();
            fs::create_dir_all(root.join("docs")).unwrap();
            fs::write(
                root.join(".docgraph/project.toml"),
                "schema_version = 1\n[project]\nname = \"fixture\"\n[documents]\nroot = \"docs\"\n",
            )
            .unwrap();
            fs::write(
                root.join(".docgraph/entities.toml"),
                "[entity.task]\ndescription = \"Task\"\nworkflow = \"task\"\n",
            )
            .unwrap();
            fs::write(
                root.join(".docgraph/workflows.toml"),
                "[workflow.task]\ninitial = \"open\"\n[workflow.task.states.open]\ndescription = \"Open\"\ntransitions = [\"review\"]\n[workflow.task.states.review]\ndescription = \"Review\"\ntransitions = [\"done\"]\n[workflow.task.states.done]\ndescription = \"Done\"\n",
            )
            .unwrap();
            let repository = Repository::discover(&root).unwrap();
            let config = RepositoryConfig::load(&repository).unwrap();
            Self(root, repository, config)
        }

        fn corpus(&self, source: &str) -> CanonicalCorpus {
            CanonicalCorpus::from_contents(
                &self.1,
                vec![(PathBuf::from("docs/task.md"), source.to_owned())],
            )
            .unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn document(id: &str, state: &str, section: &str, prose: &str) -> String {
        format!(
            "+++\nid = \"{id}\"\ntype = \"task\"\nstate = \"{state}\"\n+++\n<a id=\"{section}\"></a>\n# Task\n\n{prose}\n"
        )
    }

    #[test]
    fn accepts_prose_and_supported_state_changes() {
        let fixture = Fixture::new();
        let base = fixture.corpus(&document("task:1", "open", "s-83JRT4K2P6", "Before."));
        let candidate = fixture.corpus(&document("task:1", "review", "s-83JRT4K2P6", "After."));

        let report = ManagedChangeValidator::validate(&base, &candidate, &fixture.2);

        assert!(report.is_valid());
        assert!(
            report
                .changes
                .contains(&ManagedChange::StateChanged("task:1".to_owned()))
        );
    }

    #[test]
    fn accounts_for_lifecycle_illegal_transition_and_surviving_heading_anchor_changes() {
        let fixture = Fixture::new();
        let base = fixture.corpus(&document("task:1", "open", "s-83JRT4K2P6", "Before."));

        let terminal = fixture.corpus(&document("task:1", "done", "s-83JRT4K2P6", "Before."));
        let report = ManagedChangeValidator::validate(&terminal, &base, &fixture.2);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "unsupported-workflow-state-change")
        );

        let renamed = fixture.corpus(&document("task:2", "open", "s-83JRT4K2P6", "Before."));
        let report = ManagedChangeValidator::validate(&base, &renamed, &fixture.2);
        assert!(
            report
                .changes
                .contains(&ManagedChange::EntityRemoved("task:1".to_owned()))
        );
        assert!(
            report
                .changes
                .contains(&ManagedChange::EntityAdded("task:2".to_owned()))
        );

        let replaced = fixture.corpus(&document("task:1", "open", "s-7K3M9Q2W0", "Before."));
        let report = ManagedChangeValidator::validate(&base, &replaced, &fixture.2);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "unsupported-section-id-change")
        );
    }

    #[test]
    fn semantic_review_reports_granular_graph_changes() {
        let fixture = Fixture::new();
        let base = CanonicalCorpus::from_contents(
            &fixture.1,
            vec![(
                PathBuf::from("docs/task.md"),
                "+++\nid = \"task:1\"\ntype = \"task\"\nstate = \"open\"\n[properties]\npriority = \"low\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# Old heading\n"
                    .to_owned(),
            )],
        )
        .unwrap();
        let candidate = CanonicalCorpus::from_contents(
            &fixture.1,
            vec![(
                PathBuf::from("docs/moved.md"),
                "+++\nid = \"task:1\"\ntype = \"task\"\nstate = \"review\"\n[properties]\npriority = \"high\"\n[[relations]]\ntype = \"blocks\"\ntarget = \"task:1\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# New heading\n\n<a id=\"s-7K3M9Q2W0\"></a>\n## Added section\n"
                    .to_owned(),
            )],
        )
        .unwrap();

        let report = SemanticChangeReviewer::review(&base, &candidate, &fixture.2);

        assert!(report.is_valid());
        assert!(report.changes.iter().any(|change| matches!(
            change,
            SemanticChange::EntityMoved { entity, .. } if entity == "task:1"
        )));
        assert!(report.changes.iter().any(|change| matches!(
            change,
            SemanticChange::WorkflowStateChanged { before, after, .. }
                if before.as_deref() == Some("open") && after.as_deref() == Some("review")
        )));
        assert!(report.changes.iter().any(|change| matches!(
            change,
            SemanticChange::PropertyChanged { property, before, after, .. }
                if property == "priority"
                    && before.as_deref() == Some("\"low\"")
                    && after.as_deref() == Some("\"high\"")
        )));
        assert!(report.changes.iter().any(|change| matches!(
            change,
            SemanticChange::SectionChanged { section, before, after }
                if section == "task:1#s-83JRT4K2P6"
                    && before.heading == "Old heading"
                    && after.heading == "New heading"
        )));
        assert!(report.changes.iter().any(|change| matches!(
            change,
            SemanticChange::SectionAdded { section, .. }
                if section == "task:1#s-7K3M9Q2W0"
        )));
        assert!(report.changes.iter().any(|change| matches!(
            change,
            SemanticChange::RelationAdded { relation }
                if relation.source == "task:1"
                    && relation.predicate == "blocks"
                    && relation.target == "task:1"
        )));
    }
}
