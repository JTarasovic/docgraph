use crate::{CanonicalCorpus, GraphIndex, GraphNode, RelationOrigin, RepositoryConfig};
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
                report.diagnostics.push(ChangeDiagnostic {
                    code: "unsupported-entity-removal",
                    message: format!(
                        "entity {id:?} was removed or its identity changed; no docgraph operation supports that change"
                    ),
                    path: base_path,
                });
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
        validate_section_identity(&base_graph, &candidate_graph, &mut report);
        report
    }
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
        .map(|(name, value)| (name.clone(), value.to_string()))
        .collect()
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
    fn rejects_identity_illegal_transition_and_surviving_heading_anchor_changes() {
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
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "unsupported-entity-removal")
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
}
