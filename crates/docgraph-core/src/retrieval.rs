use crate::{CanonicalCorpus, GraphIndex, GraphNode, Relation, RelationOrigin};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Debug)]
pub struct Neighbor<'a> {
    pub node: &'a GraphNode,
    pub relation: &'a Relation,
    pub outgoing: bool,
}

pub struct GraphTraversal<'a> {
    graph: &'a GraphIndex,
}

impl<'a> GraphTraversal<'a> {
    pub fn new(graph: &'a GraphIndex) -> Self {
        Self { graph }
    }

    pub fn entity(&self, id: &str) -> Option<&'a crate::EntityNode> {
        self.graph.entities.iter().find(|entity| entity.id == id)
    }

    pub fn neighbors(&self, node: &GraphNode, origin: Option<RelationOrigin>) -> Vec<Neighbor<'a>> {
        self.graph
            .relations
            .iter()
            .filter(|relation| origin.is_none_or(|origin| relation.origin == origin))
            .filter_map(|relation| {
                if &relation.source == node {
                    Some(Neighbor {
                        node: &relation.target,
                        relation,
                        outgoing: true,
                    })
                } else if &relation.target == node {
                    Some(Neighbor {
                        node: &relation.source,
                        relation,
                        outgoing: false,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn shortest_path(
        &self,
        source: &GraphNode,
        target: &GraphNode,
        origin: Option<RelationOrigin>,
    ) -> Option<Vec<GraphNode>> {
        if source == target {
            return Some(vec![source.clone()]);
        }
        let mut pending = VecDeque::from([source.clone()]);
        let mut previous: HashMap<GraphNode, GraphNode> = HashMap::new();
        let mut visited = HashSet::from([source.clone()]);
        while let Some(node) = pending.pop_front() {
            for neighbor in self.neighbors(&node, origin) {
                if visited.insert(neighbor.node.clone()) {
                    previous.insert(neighbor.node.clone(), node.clone());
                    if neighbor.node == target {
                        let mut path = vec![target.clone()];
                        let mut cursor = target;
                        while let Some(parent) = previous.get(cursor) {
                            path.push(parent.clone());
                            cursor = parent;
                        }
                        path.reverse();
                        return Some(path);
                    }
                    pending.push_back(neighbor.node.clone());
                }
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchHit {
    pub node: GraphNode,
    pub score: f64,
    pub snippet: String,
}

#[derive(Clone, Debug, Default)]
pub struct SearchIndex {
    entries: Vec<SearchEntry>,
    document_frequency: HashMap<String, usize>,
    average_length: f64,
}

#[derive(Clone, Debug)]
struct SearchEntry {
    node: GraphNode,
    text: String,
    terms: HashMap<String, usize>,
    length: usize,
}

impl SearchIndex {
    pub fn build(corpus: &CanonicalCorpus, graph: &GraphIndex) -> Self {
        let mut entries = Vec::new();
        for (document_index, document) in graph.documents.iter().enumerate() {
            let Some(file) = corpus.files.iter().find(|file| file.path == document.path) else {
                continue;
            };
            entries.push(SearchEntry::new(
                document
                    .entity
                    .as_ref()
                    .map_or(GraphNode::Document(document_index), |id| {
                        GraphNode::Entity(id.clone())
                    }),
                file.content.clone(),
            ));
            for (section_index, section) in graph.sections.iter().enumerate() {
                if section.document == document_index {
                    entries.push(SearchEntry::new(
                        GraphNode::Section(section_index),
                        file.content[section.location.span.bytes.clone()].to_owned(),
                    ));
                }
            }
        }
        let mut document_frequency = HashMap::new();
        for entry in &entries {
            for term in entry.terms.keys() {
                *document_frequency.entry(term.clone()).or_insert(0) += 1;
            }
        }
        let average_length = if entries.is_empty() {
            0.0
        } else {
            entries.iter().map(|entry| entry.length).sum::<usize>() as f64 / entries.len() as f64
        };
        Self {
            entries,
            document_frequency,
            average_length,
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let query_terms: HashSet<_> = tokenize(query).collect();
        if query_terms.is_empty() || self.entries.is_empty() || limit == 0 {
            return Vec::new();
        }
        let count = self.entries.len() as f64;
        let mut hits: Vec<_> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let score = query_terms.iter().fold(0.0, |score, term| {
                    let frequency = entry.terms.get(term).copied().unwrap_or_default() as f64;
                    if frequency == 0.0 {
                        return score;
                    }
                    let documents = self
                        .document_frequency
                        .get(term)
                        .copied()
                        .unwrap_or_default() as f64;
                    let inverse_frequency =
                        ((count - documents + 0.5) / (documents + 0.5) + 1.0).ln();
                    let normalized = frequency * 2.2
                        / (frequency
                            + 1.2
                                * (0.25
                                    + 0.75 * entry.length as f64 / self.average_length.max(1.0)));
                    score + inverse_frequency * normalized
                });
                (score > 0.0).then(|| SearchHit {
                    node: entry.node.clone(),
                    score,
                    snippet: snippet(&entry.text, &query_terms),
                })
            })
            .collect();
        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| format!("{:?}", left.node).cmp(&format!("{:?}", right.node)))
        });
        hits.truncate(limit);
        hits
    }
}

impl SearchEntry {
    fn new(node: GraphNode, text: String) -> Self {
        let mut terms = HashMap::new();
        let mut length = 0;
        for term in tokenize(&text) {
            *terms.entry(term).or_insert(0) += 1;
            length += 1;
        }
        Self {
            node,
            text,
            terms,
            length,
        }
    }
}

fn tokenize(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|term| !term.is_empty())
        .map(str::to_lowercase)
}

fn snippet(text: &str, terms: &HashSet<String>) -> String {
    let line = text
        .lines()
        .find(|line| tokenize(line).any(|term| terms.contains(&term)))
        .unwrap_or_default()
        .trim();
    const LIMIT: usize = 200;
    if line.chars().count() <= LIMIT {
        line.to_owned()
    } else {
        format!("{}…", line.chars().take(LIMIT).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DocumentNode, GraphLocation, SectionNode};
    use docgraph_markdown::{ParsedDocument, SourceSpan, StableSectionId};
    use std::path::PathBuf;

    fn fixture() -> (CanonicalCorpus, GraphIndex) {
        let content = "<a id=\"s-83JRT4K2P6\"></a>\n# Retry policy\nExponential backoff protects the service.\n".to_owned();
        let span = SourceSpan::from_offsets(&content, 0..content.len());
        let hash = *blake3::hash(content.as_bytes()).as_bytes();
        let corpus = CanonicalCorpus {
            files: vec![crate::CorpusFile {
                path: PathBuf::from("docs/retry.md"),
                document: ParsedDocument::parse(&content).unwrap(),
                content: content.clone(),
                content_hash: hash,
            }],
            fingerprint: crate::RepositoryFingerprint::from_hex(&"1".repeat(64)).unwrap(),
        };
        let graph = GraphIndex {
            documents: vec![DocumentNode {
                path: PathBuf::from("docs/retry.md"),
                entity: Some("spec:retry".to_owned()),
                content_hash: hash,
            }],
            entities: Vec::new(),
            sections: vec![SectionNode {
                id: StableSectionId::parse("s-83JRT4K2P6"),
                document: 0,
                parent: None,
                level: 1,
                heading: "Retry policy".to_owned(),
                location: GraphLocation {
                    path: PathBuf::from("docs/retry.md"),
                    span,
                },
                content_hash: hash,
            }],
            relations: Vec::new(),
            diagnostics: Vec::new(),
        };
        (corpus, graph)
    }

    #[test]
    fn full_text_search_ranks_matching_graph_nodes() {
        let (corpus, graph) = fixture();
        let index = SearchIndex::build(&corpus, &graph);
        let hits = index.search("exponential backoff", 5);

        assert!(!hits.is_empty());
        assert!(hits[0].score > 0.0);
        assert!(hits[0].snippet.contains("Exponential backoff"));
    }

    #[test]
    fn traversal_can_exclude_informational_edges() {
        let (_, mut graph) = fixture();
        graph.relations.push(Relation {
            source: GraphNode::Entity("spec:retry".to_owned()),
            predicate: "depends_on".to_owned(),
            target: GraphNode::Entity("spec:clock".to_owned()),
            properties: Default::default(),
            origin: RelationOrigin::Explicit,
            location: graph.sections[0].location.clone(),
        });
        let traversal = GraphTraversal::new(&graph);
        let path = traversal.shortest_path(
            &GraphNode::Entity("spec:retry".to_owned()),
            &GraphNode::Entity("spec:clock".to_owned()),
            Some(RelationOrigin::Explicit),
        );

        assert_eq!(path.unwrap().len(), 2);
    }
}
