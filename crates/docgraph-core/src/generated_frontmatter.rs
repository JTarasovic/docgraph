use crate::{CanonicalCorpus, GraphIndex, GraphNode, RelationOrigin, RepositoryConfig};
use docgraph_markdown::frame_content;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use toml_edit::{DocumentMut, Item};

const GENERATED: &str = "docgraph_generated";
const SCHEMA_VERSION: i64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneratedBlockStatus {
    Current,
    Missing,
    Stale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneratedBlockError {
    MissingFrontmatter,
    MalformedTable,
}

impl fmt::Display for GeneratedBlockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFrontmatter => {
                formatter.write_str("entity document has no TOML frontmatter")
            }
            Self::MalformedTable => {
                formatter.write_str("generated frontmatter table is malformed or unsupported")
            }
        }
    }
}

impl Error for GeneratedBlockError {}

pub fn check_generated_frontmatter(
    corpus: &CanonicalCorpus,
    graph: &GraphIndex,
    config: &RepositoryConfig,
    document: usize,
) -> Result<GeneratedBlockStatus, GeneratedBlockError> {
    let file = corpus
        .files
        .iter()
        .find(|file| file.path == graph.documents[document].path)
        .expect("graph documents originate in the corpus");
    let frontmatter = file
        .document
        .frontmatter
        .as_ref()
        .ok_or(GeneratedBlockError::MissingFrontmatter)?;
    let newline = if file.content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let expected = projection(graph, config, document, newline);
    let Some(existing) = frontmatter.item(GENERATED) else {
        return Ok(GeneratedBlockStatus::Missing);
    };
    validate_generated_item(existing)?;
    Ok(
        if normalize_newlines(&render_generated_item(existing)) == normalize_newlines(&expected) {
            GeneratedBlockStatus::Current
        } else {
            GeneratedBlockStatus::Stale
        },
    )
}

pub fn sync_generated_frontmatter(
    source: &str,
    graph: &GraphIndex,
    config: &RepositoryConfig,
    document: usize,
) -> Result<String, GeneratedBlockError> {
    let parsed = docgraph_markdown::ParsedDocument::parse(source)
        .map_err(|_| GeneratedBlockError::MissingFrontmatter)?;
    let frontmatter = parsed
        .frontmatter
        .as_ref()
        .ok_or(GeneratedBlockError::MissingFrontmatter)?;
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let expected = projection(graph, config, document, newline);
    let replacement = if let Some(existing) = frontmatter.item(GENERATED) {
        validate_generated_item(existing)?;
        let mut document = frontmatter.to_mut();
        document.remove(GENERATED);
        let mut expected = projection_item(&expected);
        expected
            .as_table_mut()
            .expect("generated projection is a table")
            .decor_mut()
            .set_prefix(newline);
        document[GENERATED] = expected;
        document.set_trailing("");
        document.to_string()
    } else {
        let content = &source[frontmatter.content_span.bytes.clone()];
        let mut replacement = content.to_owned();
        if !replacement.is_empty() && !replacement.ends_with('\n') {
            replacement.push('\n');
        }
        if !replacement.is_empty() && !replacement.ends_with("\n\n") {
            replacement.push('\n');
        }
        replacement.push_str(&expected);
        replacement
    };
    let replacement = frame_content(&replacement, newline);
    let mut output = source.to_owned();
    output.replace_range(frontmatter.content_span.bytes.clone(), &replacement);
    Ok(output)
}

fn projection(
    graph: &GraphIndex,
    config: &RepositoryConfig,
    document: usize,
    newline: &str,
) -> String {
    let mut incoming = BTreeSet::new();
    let mut inverses = BTreeSet::new();
    let mut backlinks = BTreeSet::new();
    for relation in &graph.relations {
        if target_document(graph, &relation.target) != Some(document) {
            continue;
        }
        let Some(source) = node_identity(graph, &relation.source) else {
            continue;
        };
        let Some(target) = node_identity(graph, &relation.target) else {
            continue;
        };
        match relation.origin {
            RelationOrigin::Explicit => {
                incoming.insert((source.clone(), relation.predicate.clone(), target.clone()));
                if let Some(inverse) = config
                    .relations
                    .get(&relation.predicate)
                    .and_then(|relation| relation.inverse.as_deref())
                {
                    inverses.insert((target, inverse.to_owned(), source));
                }
            }
            RelationOrigin::MarkdownLink => {
                backlinks.insert((source, target));
            }
        }
    }

    let mut output =
        format!("[docgraph_generated]{newline}schema_version = {SCHEMA_VERSION}{newline}");
    for (source, predicate, target) in incoming {
        output.push_str(&format!(
            "{newline}[[docgraph_generated.incoming]]{newline}source = {}{newline}predicate = {}{newline}target = {}{newline}",
            toml_string(&source),
            toml_string(&predicate),
            toml_string(&target)
        ));
    }
    for (source, inverse, target) in inverses {
        output.push_str(&format!(
            "{newline}[[docgraph_generated.inverses]]{newline}source = {}{newline}type = {}{newline}target = {}{newline}",
            toml_string(&source),
            toml_string(&inverse),
            toml_string(&target)
        ));
    }
    for (source, target) in backlinks {
        output.push_str(&format!(
            "{newline}[[docgraph_generated.backlinks]]{newline}source = {}{newline}target = {}{newline}",
            toml_string(&source),
            toml_string(&target)
        ));
    }
    output
}

fn validate_generated_item(item: &Item) -> Result<(), GeneratedBlockError> {
    let table = item.as_table().ok_or(GeneratedBlockError::MalformedTable)?;
    if table.get("schema_version").and_then(Item::as_integer) != Some(SCHEMA_VERSION) {
        return Err(GeneratedBlockError::MalformedTable);
    }
    Ok(())
}

fn projection_item(projection: &str) -> Item {
    let mut item = projection
        .parse::<DocumentMut>()
        .expect("generated projection is valid TOML")
        .remove(GENERATED)
        .expect("generated projection contains its reserved table");
    clear_positions(&mut item);
    item
}

fn clear_positions(item: &mut Item) {
    match item {
        Item::Table(table) => {
            table.set_position(None);
            for (_, child) in table.iter_mut() {
                clear_positions(child);
            }
        }
        Item::ArrayOfTables(tables) => {
            for table in tables.iter_mut() {
                table.set_position(None);
                for (_, child) in table.iter_mut() {
                    clear_positions(child);
                }
            }
        }
        Item::Value(_) | Item::None => {}
    }
}

fn render_generated_item(item: &Item) -> String {
    let mut item = item.clone();
    clear_positions(&mut item);
    item.as_table_mut()
        .expect("generated item was validated as a table")
        .decor_mut()
        .clear();
    let mut document = DocumentMut::new();
    document[GENERATED] = item;
    document.set_trailing("");
    document.to_string()
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n")
}

fn target_document(graph: &GraphIndex, node: &GraphNode) -> Option<usize> {
    match node {
        GraphNode::Document(document) => Some(*document),
        GraphNode::Entity(id) => graph
            .entities
            .iter()
            .find(|entity| entity.id == *id)
            .map(|entity| entity.document),
        GraphNode::Section(section) => graph.sections.get(*section).map(|section| section.document),
        GraphNode::ExternalUri(_) | GraphNode::Unresolved(_) => None,
    }
}

fn node_identity(graph: &GraphIndex, node: &GraphNode) -> Option<String> {
    match node {
        GraphNode::Document(document) => Some(portable_path(&graph.documents.get(*document)?.path)),
        GraphNode::Entity(id) | GraphNode::ExternalUri(id) => Some(id.clone()),
        GraphNode::Section(section) => {
            let section = graph.sections.get(*section)?;
            let id = section.id.as_ref()?;
            let document = &graph.documents[section.document];
            Some(document.entity.as_ref().map_or_else(
                || format!("{}#{}", portable_path(&document.path), id.as_str()),
                |entity| format!("{entity}#{}", id.as_str()),
            ))
        }
        GraphNode::Unresolved(_) => None,
    }
}

fn portable_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn toml_string(value: &str) -> String {
    toml_edit::Value::from(value).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DocumentNode, GraphIndex};
    use std::path::PathBuf;

    #[test]
    fn sync_is_idempotent_and_preserves_authored_frontmatter() {
        let source = "+++\nid = \"task:1\"\ntype = \"task\"\nowner = \"me\"\n\n[[relations]]\ntype = \"implements\"\ntarget = \"spec:1\"\n+++\n# Task\n";
        let graph = GraphIndex {
            documents: vec![DocumentNode {
                path: PathBuf::from("docs/task.md"),
                entity: Some("task:1".to_owned()),
                content_hash: [0; 32],
            }],
            entities: Vec::new(),
            sections: Vec::new(),
            relations: vec![crate::Relation {
                source: GraphNode::ExternalUri("https://example.com/source".to_owned()),
                predicate: "supports".to_owned(),
                target: GraphNode::Document(0),
                properties: Default::default(),
                origin: RelationOrigin::Explicit,
                location: crate::GraphLocation {
                    path: PathBuf::from("docs/task.md"),
                    span: docgraph_markdown::SourceSpan::from_offsets("", 0..0),
                },
            }],
            diagnostics: Vec::new(),
        };
        let config = RepositoryConfig {
            project: crate::ProjectConfig {
                name: "test".to_owned(),
                documents: crate::DocumentsConfig {
                    root: "docs".into(),
                    include: vec![],
                    exclude: vec![],
                },
                frontmatter: crate::FrontmatterConfig::default(),
                agent_instructions: crate::AgentInstructionsConfig::default(),
                validation: crate::ValidationConfig::default(),
            },
            entities: Default::default(),
            relations: Default::default(),
            workflows: Default::default(),
            queries: Default::default(),
            commands: Default::default(),
            logic: None,
        };
        let once = sync_generated_frontmatter(source, &graph, &config, 0).unwrap();
        let twice = sync_generated_frontmatter(&once, &graph, &config, 0).unwrap();
        let expected = projection(&graph, &config, 0, "\n");
        assert_eq!(render_generated_item(&projection_item(&expected)), expected);
        assert_eq!(once, twice);
        assert!(once.starts_with("+++\n\nid = \"task:1\""));
        assert!(once.contains("predicate = \"supports\"\ntarget = \"docs/task.md\"\n\n+++\n"));
        assert!(once.contains("owner = \"me\""));
        assert!(once.contains("[docgraph_generated]\nschema_version = 1\n"));
        assert!(!once.contains("# docgraph:generated"));
    }

    #[test]
    fn refuses_a_non_table_generated_value() {
        let source = "docgraph_generated = true\n";
        let document = source.parse::<DocumentMut>().unwrap();
        assert_eq!(
            validate_generated_item(&document[GENERATED]),
            Err(GeneratedBlockError::MalformedTable)
        );
    }

    #[test]
    fn refuses_an_unsupported_generated_schema() {
        let source = "[docgraph_generated]\nschema_version = 2\n";
        let document = source.parse::<DocumentMut>().unwrap();
        assert_eq!(
            validate_generated_item(&document[GENERATED]),
            Err(GeneratedBlockError::MalformedTable)
        );
    }

    #[test]
    fn requires_a_generated_schema_version() {
        let source = "[docgraph_generated]\n";
        let document = source.parse::<DocumentMut>().unwrap();
        assert_eq!(
            validate_generated_item(&document[GENERATED]),
            Err(GeneratedBlockError::MalformedTable)
        );
    }
}
