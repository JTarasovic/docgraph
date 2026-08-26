use crate::{CanonicalCorpus, RepositoryConfig};
use docgraph_markdown::{ReferenceClassifier, ReferenceTarget, SourceSpan, StableSectionId};
use std::collections::{BTreeMap, HashMap};
use std::path::{Component, Path, PathBuf};
use toml_edit::{Item, Table, Value};

#[derive(Clone, Debug)]
pub struct GraphIndex {
    pub documents: Vec<DocumentNode>,
    pub entities: Vec<EntityNode>,
    pub sections: Vec<SectionNode>,
    pub relations: Vec<Relation>,
    pub diagnostics: Vec<GraphDiagnostic>,
}

#[derive(Clone, Debug)]
pub struct DocumentNode {
    pub path: PathBuf,
    pub entity: Option<String>,
    pub content_hash: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct EntityNode {
    pub id: String,
    pub entity_type: String,
    pub state: Option<String>,
    pub document: usize,
    pub properties: BTreeMap<String, Value>,
    pub location: GraphLocation,
}

#[derive(Clone, Debug)]
pub struct SectionNode {
    pub id: Option<StableSectionId>,
    pub document: usize,
    pub parent: Option<usize>,
    pub level: u8,
    pub heading: String,
    pub location: GraphLocation,
    pub content_hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphNode {
    Document(usize),
    Entity(String),
    Section(usize),
    ExternalUri(String),
    Unresolved(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationOrigin {
    Explicit,
    MarkdownLink,
}

#[derive(Clone, Debug)]
pub struct Relation {
    pub source: GraphNode,
    pub predicate: String,
    pub target: GraphNode,
    pub properties: BTreeMap<String, Value>,
    pub origin: RelationOrigin,
    pub location: GraphLocation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphLocation {
    pub path: PathBuf,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphDiagnostic {
    pub kind: DiagnosticKind,
    pub location: GraphLocation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagnosticKind {
    InvalidManagedField {
        field: String,
        expected: &'static str,
    },
    MalformedRelation {
        reason: String,
    },
}

struct RawRelation {
    source: RawSource,
    predicate: String,
    target: String,
    properties: BTreeMap<String, Value>,
    origin: RelationOrigin,
    document: usize,
    location: GraphLocation,
}

enum RawSource {
    Node(GraphNode),
    Reference(String),
}

impl GraphIndex {
    pub fn build(corpus: &CanonicalCorpus, config: &RepositoryConfig) -> Self {
        let mut graph = Self {
            documents: Vec::with_capacity(corpus.files.len()),
            entities: Vec::new(),
            sections: Vec::new(),
            relations: Vec::new(),
            diagnostics: Vec::new(),
        };
        let classifier = ReferenceClassifier::new(config.entities.keys().cloned());
        let mut raw_relations = Vec::new();

        for file in &corpus.files {
            let document_index = graph.documents.len();
            let mut entity_id = None;
            let mut entity_type = None;
            let mut state = None;
            let mut properties = BTreeMap::new();

            if let Some(frontmatter) = &file.document.frontmatter {
                entity_id = managed_string(
                    frontmatter.item(&config.project.frontmatter.id),
                    &config.project.frontmatter.id,
                    file,
                    frontmatter,
                    &mut graph.diagnostics,
                );
                entity_type = managed_string(
                    frontmatter.item(&config.project.frontmatter.entity_type),
                    &config.project.frontmatter.entity_type,
                    file,
                    frontmatter,
                    &mut graph.diagnostics,
                );
                state = managed_string(
                    frontmatter.item(&config.project.frontmatter.state),
                    &config.project.frontmatter.state,
                    file,
                    frontmatter,
                    &mut graph.diagnostics,
                );
                properties = managed_properties(
                    frontmatter.item(&config.project.frontmatter.properties),
                    &config.project.frontmatter.properties,
                    file,
                    frontmatter,
                    &mut graph.diagnostics,
                );

                if entity_id.is_some() != entity_type.is_some() {
                    graph.diagnostics.push(GraphDiagnostic {
                        kind: DiagnosticKind::InvalidManagedField {
                            field: format!(
                                "{}/{}",
                                config.project.frontmatter.id,
                                config.project.frontmatter.entity_type
                            ),
                            expected: "both entity ID and entity type",
                        },
                        location: GraphLocation {
                            path: file.path.clone(),
                            span: frontmatter.span.clone(),
                        },
                    });
                }
            }

            let has_complete_entity = entity_id.is_some() && entity_type.is_some();
            graph.documents.push(DocumentNode {
                path: file.path.clone(),
                entity: has_complete_entity.then(|| entity_id.clone()).flatten(),
                content_hash: file.content_hash,
            });

            if let (Some(id), Some(entity_type)) = (entity_id.clone(), entity_type) {
                let location = file
                    .document
                    .frontmatter
                    .as_ref()
                    .and_then(|frontmatter| {
                        frontmatter.item_span(&file.content, &config.project.frontmatter.id)
                    })
                    .map_or_else(
                        || GraphLocation {
                            path: file.path.clone(),
                            span: SourceSpan::from_offsets(&file.content, 0..file.content.len()),
                        },
                        |span| GraphLocation {
                            path: file.path.clone(),
                            span,
                        },
                    );
                graph.entities.push(EntityNode {
                    id,
                    entity_type,
                    state,
                    document: document_index,
                    properties,
                    location,
                });
            }

            let section_base = graph.sections.len();
            let mut stack: Vec<usize> = Vec::new();
            for heading in &file.document.headings {
                while stack
                    .last()
                    .is_some_and(|parent| graph.sections[*parent].level >= heading.level)
                {
                    stack.pop();
                }
                let section = graph.sections.len();
                graph.sections.push(SectionNode {
                    id: heading.id.clone(),
                    document: document_index,
                    parent: stack.last().copied(),
                    level: heading.level,
                    heading: heading.title.clone(),
                    location: GraphLocation {
                        path: file.path.clone(),
                        span: heading.section_span.clone(),
                    },
                    content_hash: *blake3::hash(
                        file.content[heading.section_span.bytes.clone()].as_bytes(),
                    )
                    .as_bytes(),
                });
                stack.push(section);
            }

            for link in &file.document.links {
                let source = link
                    .containing_section
                    .map_or(GraphNode::Document(document_index), |local| {
                        GraphNode::Section(section_base + local)
                    });
                raw_relations.push(RawRelation {
                    source: RawSource::Node(source),
                    predicate: "links_to".to_owned(),
                    target: link.destination.clone(),
                    properties: BTreeMap::new(),
                    origin: RelationOrigin::MarkdownLink,
                    document: document_index,
                    location: GraphLocation {
                        path: file.path.clone(),
                        span: link.span.clone(),
                    },
                });
            }

            if let Some(frontmatter) = &file.document.frontmatter {
                collect_explicit_relations(
                    file,
                    frontmatter,
                    &config.project.frontmatter.relations,
                    has_complete_entity
                        .then_some(entity_id.as_deref())
                        .flatten(),
                    document_index,
                    &mut raw_relations,
                    &mut graph.diagnostics,
                );
            }
        }

        let resolver = Resolver::new(&graph);
        let relations = raw_relations
            .into_iter()
            .map(|raw| {
                let source = match raw.source {
                    RawSource::Node(node) => node,
                    RawSource::Reference(reference) => {
                        resolver.resolve(&classifier, raw.document, &reference)
                    }
                };
                let target = resolver.resolve(&classifier, raw.document, &raw.target);
                Relation {
                    source,
                    predicate: raw.predicate,
                    target,
                    properties: raw.properties,
                    origin: raw.origin,
                    location: raw.location,
                }
            })
            .collect();
        drop(resolver);
        graph.relations = relations;
        graph
    }
}

fn managed_string(
    item: Option<&Item>,
    field: &str,
    file: &crate::CorpusFile,
    frontmatter: &docgraph_markdown::Frontmatter,
    diagnostics: &mut Vec<GraphDiagnostic>,
) -> Option<String> {
    let item = item?;
    if let Some(value) = item.as_str() {
        return Some(value.to_owned());
    }
    diagnostics.push(GraphDiagnostic {
        kind: DiagnosticKind::InvalidManagedField {
            field: field.to_owned(),
            expected: "string",
        },
        location: item_location(file, frontmatter, item),
    });
    None
}

fn managed_properties(
    item: Option<&Item>,
    field: &str,
    file: &crate::CorpusFile,
    frontmatter: &docgraph_markdown::Frontmatter,
    diagnostics: &mut Vec<GraphDiagnostic>,
) -> BTreeMap<String, Value> {
    let Some(item) = item else {
        return BTreeMap::new();
    };
    let Some(table) = item.as_table() else {
        diagnostics.push(GraphDiagnostic {
            kind: DiagnosticKind::InvalidManagedField {
                field: field.to_owned(),
                expected: "table",
            },
            location: item_location(file, frontmatter, item),
        });
        return BTreeMap::new();
    };
    table
        .iter()
        .filter_map(|(key, item)| {
            item.as_value()
                .cloned()
                .map(|value| (key.to_owned(), value))
        })
        .collect()
}

fn collect_explicit_relations(
    file: &crate::CorpusFile,
    frontmatter: &docgraph_markdown::Frontmatter,
    field: &str,
    entity_id: Option<&str>,
    document: usize,
    output: &mut Vec<RawRelation>,
    diagnostics: &mut Vec<GraphDiagnostic>,
) {
    let Some(item) = frontmatter.item(field) else {
        return;
    };
    let Some(relations) = item.as_array_of_tables() else {
        diagnostics.push(GraphDiagnostic {
            kind: DiagnosticKind::InvalidManagedField {
                field: field.to_owned(),
                expected: "array of tables",
            },
            location: item_location(file, frontmatter, item),
        });
        return;
    };
    for relation in relations {
        let location = table_location(file, frontmatter, relation);
        let predicate = relation.get("type").and_then(Item::as_str);
        let target = relation.get("target").and_then(Item::as_str);
        let source = relation.get("source").and_then(Item::as_str).map_or_else(
            || entity_id.map(|id| RawSource::Node(GraphNode::Entity(id.to_owned()))),
            |source| Some(RawSource::Reference(source.to_owned())),
        );
        let (Some(predicate), Some(target), Some(source)) = (predicate, target, source) else {
            diagnostics.push(GraphDiagnostic {
                kind: DiagnosticKind::MalformedRelation {
                    reason: "relation requires string type and target plus an explicit or enclosing source"
                        .to_owned(),
                },
                location,
            });
            continue;
        };
        let properties = relation
            .iter()
            .filter(|(key, _)| !matches!(*key, "type" | "target" | "source"))
            .filter_map(|(key, item)| {
                item.as_value()
                    .cloned()
                    .map(|value| (key.to_owned(), value))
            })
            .collect();
        output.push(RawRelation {
            source,
            predicate: predicate.to_owned(),
            target: target.to_owned(),
            properties,
            origin: RelationOrigin::Explicit,
            document,
            location,
        });
    }
}

fn item_location(
    file: &crate::CorpusFile,
    frontmatter: &docgraph_markdown::Frontmatter,
    item: &Item,
) -> GraphLocation {
    GraphLocation {
        path: file.path.clone(),
        span: item
            .span()
            .map(|span| frontmatter.source_span(&file.content, span))
            .unwrap_or_else(|| frontmatter.span.clone()),
    }
}

fn table_location(
    file: &crate::CorpusFile,
    frontmatter: &docgraph_markdown::Frontmatter,
    table: &Table,
) -> GraphLocation {
    GraphLocation {
        path: file.path.clone(),
        span: table
            .span()
            .map(|span| frontmatter.source_span(&file.content, span))
            .unwrap_or_else(|| frontmatter.span.clone()),
    }
}

struct Resolver<'a> {
    graph: &'a GraphIndex,
    documents: HashMap<PathBuf, usize>,
    entities: HashMap<&'a str, Vec<usize>>,
    sections: HashMap<(usize, &'a str), Vec<usize>>,
}

impl<'a> Resolver<'a> {
    fn new(graph: &'a GraphIndex) -> Self {
        let documents = graph
            .documents
            .iter()
            .enumerate()
            .map(|(index, document)| (document.path.clone(), index))
            .collect();
        let mut entities: HashMap<&str, Vec<usize>> = HashMap::new();
        for entity in &graph.entities {
            entities
                .entry(&entity.id)
                .or_default()
                .push(entity.document);
        }
        let mut sections: HashMap<(usize, &str), Vec<usize>> = HashMap::new();
        for (index, section) in graph.sections.iter().enumerate() {
            if let Some(id) = &section.id {
                sections
                    .entry((section.document, id.as_str()))
                    .or_default()
                    .push(index);
            }
        }
        Self {
            graph,
            documents,
            entities,
            sections,
        }
    }

    fn resolve(
        &self,
        classifier: &ReferenceClassifier,
        source_document: usize,
        raw: &str,
    ) -> GraphNode {
        match classifier.classify(raw) {
            ReferenceTarget::CurrentDocumentSection(id) => self.section(source_document, &id, raw),
            ReferenceTarget::RelativeDocument { path, section } => {
                let source = &self.graph.documents[source_document].path;
                let Some(path) = normalize_relative(source, &path) else {
                    return GraphNode::Unresolved(raw.to_owned());
                };
                let Some(document) = self.documents.get(&path).copied() else {
                    return GraphNode::Unresolved(raw.to_owned());
                };
                section.map_or(GraphNode::Document(document), |id| {
                    self.section(document, &id, raw)
                })
            }
            ReferenceTarget::CanonicalEntity { id, section } => {
                let Some(documents) = self.entities.get(id.as_str()) else {
                    return GraphNode::Unresolved(raw.to_owned());
                };
                if documents.len() != 1 {
                    return GraphNode::Unresolved(raw.to_owned());
                }
                section.map_or(GraphNode::Entity(id), |section| {
                    self.section(documents[0], &section, raw)
                })
            }
            ReferenceTarget::ExternalUri(uri) => GraphNode::ExternalUri(uri),
            ReferenceTarget::Unresolved(_) => GraphNode::Unresolved(raw.to_owned()),
        }
    }

    fn section(&self, document: usize, id: &StableSectionId, raw: &str) -> GraphNode {
        self.sections
            .get(&(document, id.as_str()))
            .filter(|matches| matches.len() == 1)
            .map_or_else(
                || GraphNode::Unresolved(raw.to_owned()),
                |matches| GraphNode::Section(matches[0]),
            )
    }
}

fn normalize_relative(source: &Path, raw: &str) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    let joined = source.parent()?.join(raw);
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(part) => normalized.push(part),
            Component::Prefix(_) | Component::RootDir => return None,
        }
    }
    Some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CanonicalCorpus, Repository, RepositoryConfig};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "docgraph-graph-test-{}-{sequence}",
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
                "[entity.task]\ndescription = \"Task\"\n[entity.task.property.title]\ntype = \"string\"\nrequired = true\n[entity.adr]\ndescription = \"Decision\"\n",
            )
            .unwrap();
            fs::write(
                root.join(".docgraph/relations.toml"),
                "[relation.blocked_by]\ndescription = \"Blocked by\"\nsource = [\"task\"]\ntarget = [\"adr\"]\n",
            )
            .unwrap();
            fs::write(
                root.join("docs/task.md"),
                "+++\nid = \"task:1\"\ntype = \"task\"\nstate = \"open\"\n[properties]\ntitle = \"Ship it\"\n[[relations]]\ntype = \"blocked_by\"\ntarget = \"adr:2\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# Task\nSee [decision](./adr.md#s-7K3M9Q2W).\n",
            )
            .unwrap();
            fs::write(
                root.join("docs/adr.md"),
                "+++\nid = \"adr:2\"\ntype = \"adr\"\n+++\n<a id=\"s-7K3M9Q2W\"></a>\n# Decision\n",
            )
            .unwrap();
            Self(root)
        }

        fn build(&self) -> GraphIndex {
            let repository = Repository::discover(&self.0).unwrap();
            let config = RepositoryConfig::load(&repository).unwrap();
            let corpus = CanonicalCorpus::load(&repository, &config).unwrap();
            GraphIndex::build(&corpus, &config)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn builds_entities_sections_and_resolved_relations() {
        let fixture = Fixture::new();

        let graph = fixture.build();

        assert!(graph.diagnostics.is_empty());
        assert_eq!(graph.documents.len(), 2);
        assert_eq!(graph.entities.len(), 2);
        assert_eq!(graph.sections.len(), 2);
        let task = graph
            .entities
            .iter()
            .find(|entity| entity.id == "task:1")
            .unwrap();
        assert_eq!(task.state.as_deref(), Some("open"));
        assert_eq!(task.properties["title"].as_str(), Some("Ship it"));

        let explicit = graph
            .relations
            .iter()
            .find(|relation| relation.origin == RelationOrigin::Explicit)
            .unwrap();
        assert_eq!(explicit.source, GraphNode::Entity("task:1".to_owned()));
        assert_eq!(explicit.target, GraphNode::Entity("adr:2".to_owned()));

        let link = graph
            .relations
            .iter()
            .find(|relation| relation.origin == RelationOrigin::MarkdownLink)
            .unwrap();
        assert!(matches!(link.source, GraphNode::Section(_)));
        assert!(matches!(link.target, GraphNode::Section(_)));
        assert_eq!(link.location.span.start_line, 13);
    }

    #[test]
    fn preserves_unresolved_references_for_validation() {
        let fixture = Fixture::new();
        fs::write(
            fixture.0.join("docs/adr.md"),
            "+++\nid = \"adr:2\"\ntype = \"adr\"\n+++\n# Missing stable ID\n",
        )
        .unwrap();

        let graph = fixture.build();
        let link = graph
            .relations
            .iter()
            .find(|relation| relation.origin == RelationOrigin::MarkdownLink)
            .unwrap();

        assert_eq!(
            link.target,
            GraphNode::Unresolved("./adr.md#s-7K3M9Q2W".to_owned())
        );
    }
}
