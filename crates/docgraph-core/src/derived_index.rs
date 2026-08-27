use crate::{CanonicalCorpus, GraphIndex, GraphNode, RelationOrigin, RepositoryFingerprint};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use std::path::Path;

const INDEX_SCHEMA: &str = r#"
PRAGMA user_version = 1;

CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

CREATE TABLE documents (
    document_key INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    entity_id TEXT,
    content_hash BLOB NOT NULL
) STRICT;

CREATE TABLE entities (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    state TEXT,
    document_key INTEGER NOT NULL REFERENCES documents(document_key),
    path TEXT NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL,
    start_line INTEGER NOT NULL,
    start_column INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    end_column INTEGER NOT NULL
) STRICT;

CREATE TABLE entity_properties (
    entity_id TEXT NOT NULL REFERENCES entities(id),
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (entity_id, name)
) STRICT;

CREATE TABLE sections (
    section_key INTEGER PRIMARY KEY,
    stable_id TEXT,
    canonical_id TEXT NOT NULL UNIQUE,
    document_key INTEGER NOT NULL REFERENCES documents(document_key),
    parent_key INTEGER REFERENCES sections(section_key),
    level INTEGER NOT NULL,
    heading TEXT NOT NULL,
    path TEXT NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL,
    start_line INTEGER NOT NULL,
    start_column INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    end_column INTEGER NOT NULL,
    content_hash BLOB NOT NULL
) STRICT;

CREATE TABLE relations (
    relation_key INTEGER PRIMARY KEY,
    source_kind TEXT NOT NULL,
    source TEXT NOT NULL,
    predicate TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target TEXT NOT NULL,
    origin TEXT NOT NULL,
    path TEXT NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL,
    start_line INTEGER NOT NULL,
    start_column INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    end_column INTEGER NOT NULL
) STRICT;

CREATE INDEX relations_source ON relations(source, predicate);
CREATE INDEX relations_target ON relations(target, predicate);

CREATE TABLE relation_properties (
    relation_key INTEGER NOT NULL REFERENCES relations(relation_key),
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (relation_key, name)
) STRICT;

CREATE VIRTUAL TABLE search_entries USING fts5(
    node UNINDEXED,
    content,
    tokenize = 'unicode61'
);
"#;

#[derive(Clone, Debug, PartialEq)]
pub struct DerivedSearchHit {
    pub node: String,
    pub score: f64,
    pub snippet: String,
}

pub(crate) fn build(
    path: &Path,
    fingerprint: RepositoryFingerprint,
    corpus: &CanonicalCorpus,
    graph: &GraphIndex,
) -> rusqlite::Result<()> {
    let mut connection = Connection::open(path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = DELETE;")?;
    let transaction = connection.transaction()?;
    transaction.execute_batch(INDEX_SCHEMA)?;
    transaction.execute(
        "INSERT INTO metadata(key, value) VALUES ('fingerprint', ?1)",
        [fingerprint.to_string()],
    )?;
    populate(&transaction, corpus, graph)?;
    transaction.commit()
}

pub(crate) fn recorded_fingerprint(path: &Path) -> rusqlite::Result<Option<RepositoryFingerprint>> {
    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    connection
        .query_row(
            "SELECT value FROM metadata WHERE key = 'fingerprint'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map(|value| value.and_then(|value| RepositoryFingerprint::from_hex(&value)))
}

pub(crate) fn search(
    path: &Path,
    query: &str,
    limit: usize,
) -> rusqlite::Result<Vec<DerivedSearchHit>> {
    let Some(query) = fts_query(query) else {
        return Ok(Vec::new());
    };
    if limit == 0 {
        return Ok(Vec::new());
    }
    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare(
        "SELECT node, -bm25(search_entries), snippet(search_entries, 1, '', '', '…', 24)
         FROM search_entries
         WHERE search_entries MATCH ?1
         ORDER BY bm25(search_entries), node
         LIMIT ?2",
    )?;
    let rows = statement.query_map(params![query, limit as i64], |row| {
        Ok(DerivedSearchHit {
            node: row.get(0)?,
            score: row.get(1)?,
            snippet: row.get(2)?,
        })
    })?;
    rows.collect()
}

fn populate(
    transaction: &Transaction<'_>,
    corpus: &CanonicalCorpus,
    graph: &GraphIndex,
) -> rusqlite::Result<()> {
    for (document_key, document) in graph.documents.iter().enumerate() {
        transaction.execute(
            "INSERT INTO documents(document_key, path, entity_id, content_hash) VALUES (?1, ?2, ?3, ?4)",
            params![
                document_key as i64,
                portable_path(&document.path),
                document.entity,
                &document.content_hash[..],
            ],
        )?;
        let file = corpus
            .files
            .iter()
            .find(|file| file.path == document.path)
            .expect("graph documents originate in the canonical corpus");
        let document_node = document
            .entity
            .clone()
            .unwrap_or_else(|| portable_path(&document.path));
        transaction.execute(
            "INSERT INTO search_entries(node, content) VALUES (?1, ?2)",
            params![document_node, file.content],
        )?;
    }

    for entity in &graph.entities {
        let span = &entity.location.span;
        transaction.execute(
            "INSERT INTO entities(
                id, entity_type, state, document_key, path,
                start_byte, end_byte, start_line, start_column, end_line, end_column
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                entity.id,
                entity.entity_type,
                entity.state,
                entity.document as i64,
                portable_path(&entity.location.path),
                span.bytes.start as i64,
                span.bytes.end as i64,
                span.start_line as i64,
                span.start_column as i64,
                span.end_line as i64,
                span.end_column as i64,
            ],
        )?;
        for (name, value) in &entity.properties {
            transaction.execute(
                "INSERT INTO entity_properties(entity_id, name, value) VALUES (?1, ?2, ?3)",
                params![entity.id, name, value.to_string()],
            )?;
        }
    }

    for (section_key, section) in graph.sections.iter().enumerate() {
        let canonical_id = node_identity(graph, &GraphNode::Section(section_key)).1;
        let span = &section.location.span;
        transaction.execute(
            "INSERT INTO sections(
                section_key, stable_id, canonical_id, document_key, parent_key, level, heading, path,
                start_byte, end_byte, start_line, start_column, end_line, end_column, content_hash
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                section_key as i64,
                section.id.as_ref().map(|id| id.as_str()),
                canonical_id,
                section.document as i64,
                section.parent.map(|parent| parent as i64),
                i64::from(section.level),
                section.heading,
                portable_path(&section.location.path),
                span.bytes.start as i64,
                span.bytes.end as i64,
                span.start_line as i64,
                span.start_column as i64,
                span.end_line as i64,
                span.end_column as i64,
                &section.content_hash[..],
            ],
        )?;
        let document = &graph.documents[section.document];
        let file = corpus
            .files
            .iter()
            .find(|file| file.path == document.path)
            .expect("graph sections originate in the canonical corpus");
        transaction.execute(
            "INSERT INTO search_entries(node, content) VALUES (?1, ?2)",
            params![canonical_id, &file.content[span.bytes.clone()]],
        )?;
    }

    for (relation_key, relation) in graph.relations.iter().enumerate() {
        let (source_kind, source) = node_identity(graph, &relation.source);
        let (target_kind, target) = node_identity(graph, &relation.target);
        let span = &relation.location.span;
        transaction.execute(
            "INSERT INTO relations(
                relation_key, source_kind, source, predicate, target_kind, target, origin, path,
                start_byte, end_byte, start_line, start_column, end_line, end_column
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                relation_key as i64,
                source_kind,
                source,
                relation.predicate,
                target_kind,
                target,
                match relation.origin {
                    RelationOrigin::Explicit => "explicit",
                    RelationOrigin::MarkdownLink => "markdown_link",
                },
                portable_path(&relation.location.path),
                span.bytes.start as i64,
                span.bytes.end as i64,
                span.start_line as i64,
                span.start_column as i64,
                span.end_line as i64,
                span.end_column as i64,
            ],
        )?;
        for (name, value) in &relation.properties {
            transaction.execute(
                "INSERT INTO relation_properties(relation_key, name, value) VALUES (?1, ?2, ?3)",
                params![relation_key as i64, name, value.to_string()],
            )?;
        }
    }
    Ok(())
}

fn node_identity(graph: &GraphIndex, node: &GraphNode) -> (&'static str, String) {
    match node {
        GraphNode::Document(index) => ("document", portable_path(&graph.documents[*index].path)),
        GraphNode::Entity(id) => ("entity", id.clone()),
        GraphNode::Section(index) => {
            let section = &graph.sections[*index];
            let document = &graph.documents[section.document];
            let base = document
                .entity
                .clone()
                .unwrap_or_else(|| portable_path(&document.path));
            let suffix = section
                .id
                .as_ref()
                .map_or_else(|| format!("<section:{index}>"), |id| id.as_str().to_owned());
            ("section", format!("{base}#{suffix}"))
        }
        GraphNode::ExternalUri(uri) => ("external", uri.clone()),
        GraphNode::Unresolved(reference) => ("unresolved", reference.clone()),
    }
}

fn portable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn fts_query(query: &str) -> Option<String> {
    let terms: Vec<_> = query
        .split(|character: char| !character.is_alphanumeric())
        .filter(|term| !term.is_empty())
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" OR "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CorpusFile, DocumentNode, EntityNode, GraphLocation, Relation, SectionNode};
    use docgraph_markdown::{ParsedDocument, SourceSpan, StableSectionId};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn persists_graph_locations_properties_and_search_content() {
        let content = "<a id=\"s-83JRT4K2P6\"></a>\n# Retry policy\nExponential backoff protects the service.\n".to_owned();
        let path = PathBuf::from("docs/retry.md");
        let span = SourceSpan::from_offsets(&content, 0..content.len());
        let hash = *blake3::hash(content.as_bytes()).as_bytes();
        let fingerprint = RepositoryFingerprint::from_hex(&"1".repeat(64)).unwrap();
        let corpus = CanonicalCorpus {
            files: vec![CorpusFile {
                path: path.clone(),
                content: content.clone(),
                content_hash: hash,
                document: ParsedDocument::parse(&content).unwrap(),
            }],
            fingerprint,
        };
        let location = GraphLocation {
            path: path.clone(),
            span: span.clone(),
        };
        let mut properties = BTreeMap::new();
        properties.insert("title".to_owned(), toml_edit::Value::from("Retry policy"));
        let graph = GraphIndex {
            documents: vec![DocumentNode {
                path,
                entity: Some("spec:retry".to_owned()),
                content_hash: hash,
            }],
            entities: vec![EntityNode {
                id: "spec:retry".to_owned(),
                entity_type: "spec".to_owned(),
                state: Some("active".to_owned()),
                document: 0,
                properties,
                location: location.clone(),
            }],
            sections: vec![SectionNode {
                id: StableSectionId::parse("s-83JRT4K2P6"),
                document: 0,
                parent: None,
                level: 1,
                heading: "Retry policy".to_owned(),
                location: location.clone(),
                content_hash: hash,
            }],
            relations: vec![Relation {
                source: GraphNode::Entity("spec:retry".to_owned()),
                predicate: "depends_on".to_owned(),
                target: GraphNode::ExternalUri("spec:clock".to_owned()),
                properties: BTreeMap::new(),
                origin: RelationOrigin::Explicit,
                location,
            }],
            diagnostics: Vec::new(),
        };
        let database = std::env::temp_dir().join(format!(
            "docgraph-derived-index-test-{}.sqlite",
            std::process::id()
        ));
        let _ = fs::remove_file(&database);

        build(&database, fingerprint, &corpus, &graph).unwrap();

        let connection = Connection::open(&database).unwrap();
        let entity_count: i64 = connection
            .query_row("SELECT count(*) FROM entities", [], |row| row.get(0))
            .unwrap();
        let relation_location: (String, i64) = connection
            .query_row(
                "SELECT path, start_line FROM relations WHERE predicate = 'depends_on'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        drop(connection);
        let hits = search(&database, "exponential backoff", 5).unwrap();

        assert_eq!(recorded_fingerprint(&database).unwrap(), Some(fingerprint));
        assert_eq!(entity_count, 1);
        assert_eq!(relation_location, ("docs/retry.md".to_owned(), 1));
        assert_eq!(hits[0].node, "spec:retry");
        assert!(hits[0].snippet.contains("Exponential backoff"));

        fs::remove_file(database).unwrap();
    }
}
