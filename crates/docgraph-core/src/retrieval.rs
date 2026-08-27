use crate::{GraphIndex, GraphNode, Relation, RelationOrigin};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DocumentNode, GraphLocation, SectionNode};
    use docgraph_markdown::{SourceSpan, StableSectionId};
    use std::path::PathBuf;

    fn fixture() -> GraphIndex {
        let content = "<a id=\"s-83JRT4K2P6\"></a>\n# Retry policy\nExponential backoff protects the service.\n".to_owned();
        let span = SourceSpan::from_offsets(&content, 0..content.len());
        let hash = *blake3::hash(content.as_bytes()).as_bytes();
        GraphIndex {
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
        }
    }

    #[test]
    fn traversal_can_exclude_informational_edges() {
        let mut graph = fixture();
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
