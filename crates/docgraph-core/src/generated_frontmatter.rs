use crate::{CanonicalCorpus, GraphIndex, GraphNode, RelationOrigin, RepositoryConfig};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

const BEGIN: &str = "# docgraph:generated:v1:begin";
const END: &str = "# docgraph:generated:end";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneratedBlockStatus {
    Current,
    Missing,
    Stale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneratedBlockError {
    MissingFrontmatter,
    MalformedMarkers,
}

impl fmt::Display for GeneratedBlockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFrontmatter => {
                formatter.write_str("entity document has no TOML frontmatter")
            }
            Self::MalformedMarkers => {
                formatter.write_str("generated frontmatter markers are malformed or ambiguous")
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
    let content = &file.content[frontmatter.content_span.bytes.clone()];
    let newline = if file.content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let expected = projection(graph, config, document, newline);
    let Some(region) = region(content)? else {
        return Ok(GeneratedBlockStatus::Missing);
    };
    Ok(if content[region] == expected {
        GeneratedBlockStatus::Current
    } else {
        GeneratedBlockStatus::Stale
    })
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
    let content = &source[frontmatter.content_span.bytes.clone()];
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let expected = projection(graph, config, document, newline);
    let replacement = if let Some(region) = region(content)? {
        let mut replacement = content.to_owned();
        replacement.replace_range(region, &expected);
        replacement
    } else {
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
        match relation.origin {
            RelationOrigin::Explicit => {
                incoming.insert((source.clone(), relation.predicate.clone()));
                if let Some(inverse) = config
                    .relations
                    .get(&relation.predicate)
                    .and_then(|relation| relation.inverse.as_deref())
                {
                    inverses.insert((inverse.to_owned(), source));
                }
            }
            RelationOrigin::MarkdownLink => {
                backlinks.insert(source);
            }
        }
    }

    let mut output = format!("{BEGIN}{newline}[docgraph_generated]{newline}");
    for (source, predicate) in incoming {
        output.push_str(&format!(
            "{newline}[[docgraph_generated.incoming]]{newline}source = {}{newline}predicate = {}{newline}",
            toml_string(&source),
            toml_string(&predicate)
        ));
    }
    for (inverse, target) in inverses {
        output.push_str(&format!(
            "{newline}[[docgraph_generated.inverses]]{newline}type = {}{newline}target = {}{newline}",
            toml_string(&inverse),
            toml_string(&target)
        ));
    }
    for source in backlinks {
        output.push_str(&format!(
            "{newline}[[docgraph_generated.backlinks]]{newline}source = {}{newline}",
            toml_string(&source)
        ));
    }
    output.push_str(END);
    output.push('\n');
    output
}

fn region(content: &str) -> Result<Option<std::ops::Range<usize>>, GeneratedBlockError> {
    let mut begin = Vec::new();
    let mut end = Vec::new();
    let mut offset = 0;
    for line in content.split_inclusive('\n') {
        let logical = line.strip_suffix('\n').unwrap_or(line);
        let logical = logical.strip_suffix('\r').unwrap_or(logical);
        if logical == BEGIN {
            begin.push(offset);
        } else if logical == END {
            end.push(offset + line.len());
        } else if logical.contains("docgraph:generated") {
            return Err(GeneratedBlockError::MalformedMarkers);
        }
        offset += line.len();
    }
    match (begin.as_slice(), end.as_slice()) {
        ([], []) => Ok(None),
        ([start], [finish]) if start < finish => Ok(Some(*start..*finish)),
        _ => Err(GeneratedBlockError::MalformedMarkers),
    }
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
        GraphNode::Document(document) => {
            Some(graph.documents.get(*document)?.path.display().to_string())
        }
        GraphNode::Entity(id) | GraphNode::ExternalUri(id) => Some(id.clone()),
        GraphNode::Section(section) => {
            let section = graph.sections.get(*section)?;
            let id = section.id.as_ref()?;
            let document = &graph.documents[section.document];
            Some(document.entity.as_ref().map_or_else(
                || format!("{}#{}", document.path.display(), id.as_str()),
                |entity| format!("{entity}#{}", id.as_str()),
            ))
        }
        GraphNode::Unresolved(_) => None,
    }
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
        let source = "+++\nid = \"task:1\"\ntype = \"task\"\nowner = \"me\"\n+++\n# Task\n";
        let graph = GraphIndex {
            documents: vec![DocumentNode {
                path: PathBuf::from("docs/task.md"),
                entity: Some("task:1".to_owned()),
                content_hash: [0; 32],
            }],
            entities: Vec::new(),
            sections: Vec::new(),
            relations: Vec::new(),
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
            logic: None,
        };
        let once = sync_generated_frontmatter(source, &graph, &config, 0).unwrap();
        let twice = sync_generated_frontmatter(&once, &graph, &config, 0).unwrap();
        assert_eq!(once, twice);
        assert!(once.contains("owner = \"me\""));
        assert!(once.contains(BEGIN));
    }

    #[test]
    fn refuses_ambiguous_markers() {
        let content = format!("{BEGIN}\n{BEGIN}\n{END}\n");
        assert_eq!(region(&content), Err(GeneratedBlockError::MalformedMarkers));
    }
}
