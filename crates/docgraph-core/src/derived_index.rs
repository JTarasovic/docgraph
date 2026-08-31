use crate::{
    CanonicalCorpus, DerivedExternalEntity, EmbeddingConfig, EmbeddingError, EmbeddingFallback,
    EmbeddingProvider, GraphIndex, GraphNode, RelationOrigin, RepositoryFingerprint,
    SemanticSearchHit, SemanticSearchMode, SemanticSearchResult,
};
use docgraph_markdown::searchable_markdown;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

const INDEX_SCHEMA: &str = r#"
PRAGMA user_version = 3;

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
    content_hash UNINDEXED,
    tokenize = 'unicode61'
);

CREATE TABLE vector_entries (
    node TEXT NOT NULL,
    content TEXT NOT NULL,
    content_hash BLOB NOT NULL,
    provider_key TEXT NOT NULL,
    vector BLOB NOT NULL,
    PRIMARY KEY (node, provider_key)
) STRICT;

CREATE INDEX vector_entries_reuse ON vector_entries(content_hash, provider_key);
"#;

#[derive(Clone, Debug, PartialEq)]
pub struct DerivedSearchHit {
    pub node: String,
    pub score: f64,
    pub snippet: String,
}

#[cfg(test)]
pub(crate) fn build(
    path: &Path,
    fingerprint: RepositoryFingerprint,
    corpus: &CanonicalCorpus,
    graph: &GraphIndex,
) -> rusqlite::Result<()> {
    build_with_external(path, fingerprint, corpus, graph, &[])
}

pub(crate) fn build_with_external(
    path: &Path,
    fingerprint: RepositoryFingerprint,
    corpus: &CanonicalCorpus,
    graph: &GraphIndex,
    external: &[DerivedExternalEntity],
) -> rusqlite::Result<()> {
    initialize_sqlite_vec()?;
    let mut connection = Connection::open(path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = DELETE;")?;
    let transaction = connection.transaction()?;
    transaction.execute_batch(INDEX_SCHEMA)?;
    transaction.execute(
        "INSERT INTO metadata(key, value) VALUES ('fingerprint', ?1)",
        [fingerprint.to_string()],
    )?;
    transaction.execute(
        "INSERT INTO metadata(key, value) VALUES ('external_fingerprint', ?1)",
        [external_fingerprint(external)],
    )?;
    populate(&transaction, corpus, graph, external)?;
    transaction.commit()
}

pub(crate) fn recorded_external_fingerprint(path: &Path) -> rusqlite::Result<Option<String>> {
    initialize_sqlite_vec()?;
    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    metadata(&connection, "external_fingerprint")
}

pub(crate) fn external_fingerprint(external: &[DerivedExternalEntity]) -> String {
    let mut records: Vec<_> = external.iter().collect();
    records.sort_by(|left, right| left.record.identity.cmp(&right.record.identity));
    let mut hasher = blake3::Hasher::new();
    for record in records {
        hasher.update(record.record.identity.as_bytes());
        hasher.update(&[0]);
        hasher.update(
            serde_json::to_string(&record.record)
                .expect("external records serialize")
                .as_bytes(),
        );
        hasher.update(&[0xff]);
    }
    hasher.finalize().to_hex().to_string()
}

pub(crate) fn recorded_fingerprint(path: &Path) -> rusqlite::Result<Option<RepositoryFingerprint>> {
    initialize_sqlite_vec()?;
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
    initialize_sqlite_vec()?;
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

pub(crate) fn index_vectors(
    path: &Path,
    previous: Option<&Path>,
    config: &EmbeddingConfig,
    provider: &dyn EmbeddingProvider,
) -> Result<(), EmbeddingError> {
    initialize_sqlite_vec().map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    let mut connection =
        Connection::open(path).map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    let provider_key = config.identity();
    let reusable = previous.map_or_else(HashMap::new, |path| reusable_vectors(path, &provider_key));
    let chunks = {
        let mut statement = connection
            .prepare("SELECT node, content, content_hash FROM search_entries ORDER BY node")
            .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| EmbeddingError::Protocol(error.to_string()))?
    };
    let mut missing = Vec::new();
    let transaction = connection
        .transaction()
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    for (node, content, content_hash) in &chunks {
        if let Some(vector) = reusable.get(content_hash) {
            insert_vector(
                &transaction,
                node,
                content,
                content_hash,
                &provider_key,
                vector,
            )?;
        } else {
            missing.push((node, content, content_hash));
        }
    }
    if !missing.is_empty() {
        for batch in missing.chunks(config.batch_size) {
            let texts: Vec<_> = batch
                .iter()
                .map(|(_, content, _)| (*content).clone())
                .collect();
            let vectors = provider.embed(&texts)?;
            validate_vectors(&vectors, texts.len(), config.dimensions)?;
            for ((node, content, content_hash), vector) in batch.iter().zip(vectors) {
                insert_vector(
                    &transaction,
                    node,
                    content,
                    content_hash,
                    &provider_key,
                    &encode_vector(&vector),
                )?;
            }
        }
    }
    transaction
        .execute(
            "INSERT OR REPLACE INTO metadata(key, value) VALUES ('vector_provider_key', ?1)",
            [&provider_key],
        )
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    transaction
        .execute(
            "INSERT OR REPLACE INTO metadata(key, value) VALUES ('vector_status', 'ready')",
            [],
        )
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))
}

pub(crate) fn record_vector_failure(path: &Path, error: &EmbeddingError) -> rusqlite::Result<()> {
    initialize_sqlite_vec()?;
    let connection = Connection::open(path)?;
    connection.execute(
        "INSERT OR REPLACE INTO metadata(key, value) VALUES ('vector_status', 'unavailable')",
        [],
    )?;
    connection.execute(
        "INSERT OR REPLACE INTO metadata(key, value) VALUES ('vector_error', ?1)",
        [error.to_string()],
    )?;
    Ok(())
}

pub(crate) fn semantic_search(
    path: &Path,
    query: &str,
    limit: usize,
    config: &EmbeddingConfig,
    provider: &dyn EmbeddingProvider,
) -> Result<SemanticSearchResult, EmbeddingError> {
    initialize_sqlite_vec().map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    if limit == 0 {
        return Ok(SemanticSearchResult {
            mode: SemanticSearchMode::Vector,
            reason: None,
            hits: Vec::new(),
        });
    }
    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    let status = metadata(&connection, "vector_status")
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    if status.as_deref() != Some("ready") {
        let reason = metadata(&connection, "vector_error")
            .map_err(|error| EmbeddingError::Protocol(error.to_string()))?
            .unwrap_or_else(|| "vector index is unavailable".to_owned());
        return fallback(path, query, limit, config.fallback, reason);
    }
    let query_vector = match provider.embed(&[query.to_owned()]) {
        Ok(vectors) => {
            validate_vectors(&vectors, 1, config.dimensions)?;
            vectors
                .into_iter()
                .next()
                .expect("one vector was validated")
        }
        Err(error) => return fallback(path, query, limit, config.fallback, error.to_string()),
    };
    let query_vector = encode_vector(&query_vector);
    let mut statement = connection
        .prepare(
            "SELECT node, content, vec_distance_cosine(vector, vec_f32(?2)) AS distance
             FROM vector_entries
             WHERE provider_key = ?1
             ORDER BY distance, node
             LIMIT ?3",
        )
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    let rows = statement
        .query_map(
            params![config.identity(), query_vector, limit as i64],
            |row| {
                Ok(SemanticSearchHit {
                    node: row.get(0)?,
                    snippet: compact_snippet(&row.get::<_, String>(1)?),
                    score: 1.0 - row.get::<_, f64>(2)?,
                })
            },
        )
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    let hits = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    Ok(SemanticSearchResult {
        mode: SemanticSearchMode::Vector,
        reason: None,
        hits,
    })
}

fn fallback(
    path: &Path,
    query: &str,
    limit: usize,
    policy: EmbeddingFallback,
    reason: String,
) -> Result<SemanticSearchResult, EmbeddingError> {
    if policy == EmbeddingFallback::Error {
        return Err(EmbeddingError::Unavailable(reason));
    }
    let hits = search(path, query, limit)
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?
        .into_iter()
        .map(|hit| SemanticSearchHit {
            node: hit.node,
            score: hit.score,
            snippet: hit.snippet,
        })
        .collect();
    Ok(SemanticSearchResult {
        mode: SemanticSearchMode::FullTextFallback,
        reason: Some(reason),
        hits,
    })
}

fn reusable_vectors(path: &Path, provider_key: &str) -> HashMap<Vec<u8>, Vec<u8>> {
    if initialize_sqlite_vec().is_err() {
        return HashMap::new();
    }
    let Ok(connection) =
        Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    else {
        return HashMap::new();
    };
    let Ok(mut statement) = connection.prepare(
        "SELECT content_hash, vector FROM vector_entries WHERE provider_key = ?1 ORDER BY node",
    ) else {
        return HashMap::new();
    };
    let Ok(rows) = statement.query_map([provider_key], |row| Ok((row.get(0)?, row.get(1)?))) else {
        return HashMap::new();
    };
    rows.filter_map(Result::ok).collect()
}

fn insert_vector(
    transaction: &Transaction<'_>,
    node: &str,
    content: &str,
    content_hash: &[u8],
    provider_key: &str,
    vector: &[u8],
) -> Result<(), EmbeddingError> {
    transaction
        .execute(
            "INSERT INTO vector_entries(node, content, content_hash, provider_key, vector) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![node, content, content_hash, provider_key, vector],
        )
        .map_err(|error| EmbeddingError::Protocol(error.to_string()))?;
    Ok(())
}

fn metadata(connection: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    connection
        .query_row("SELECT value FROM metadata WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()
}

fn encode_vector(vector: &[f32]) -> Vec<u8> {
    vector
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn validate_vectors(
    vectors: &[Vec<f32>],
    expected_count: usize,
    dimensions: usize,
) -> Result<(), EmbeddingError> {
    if vectors.len() != expected_count {
        return Err(EmbeddingError::Protocol(format!(
            "provider returned {} vectors for {expected_count} texts",
            vectors.len()
        )));
    }
    if let Some(vector) = vectors
        .iter()
        .find(|vector| vector.len() != dimensions || vector.iter().any(|value| !value.is_finite()))
    {
        return Err(EmbeddingError::Protocol(format!(
            "provider returned an invalid {}-dimension vector; expected {dimensions} finite values",
            vector.len()
        )));
    }
    Ok(())
}

fn initialize_sqlite_vec() -> rusqlite::Result<()> {
    type ExtensionEntry = unsafe extern "C" fn(
        *mut rusqlite::ffi::sqlite3,
        *mut *mut std::ffi::c_char,
        *const rusqlite::ffi::sqlite3_api_routines,
    ) -> std::ffi::c_int;
    static RESULT: OnceLock<i32> = OnceLock::new();
    let result = *RESULT.get_or_init(|| unsafe {
        rusqlite::ffi::sqlite3_auto_extension(Some(
            std::mem::transmute::<*const (), ExtensionEntry>(
                sqlite_vec::sqlite3_vec_init as *const (),
            ),
        ))
    });
    if result == rusqlite::ffi::SQLITE_OK {
        Ok(())
    } else {
        Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(result),
            Some("cannot register sqlite-vec".to_owned()),
        ))
    }
}

fn compact_snippet(content: &str) -> String {
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut characters = compact.chars();
    let snippet: String = characters.by_ref().take(160).collect();
    if characters.next().is_some() {
        format!("{snippet}…")
    } else {
        snippet
    }
}

fn populate(
    transaction: &Transaction<'_>,
    corpus: &CanonicalCorpus,
    graph: &GraphIndex,
    external: &[DerivedExternalEntity],
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
        let search_content = file.document.searchable_text(&file.content);
        let search_hash = blake3::hash(search_content.as_bytes());
        transaction.execute(
            "INSERT INTO search_entries(node, content, content_hash) VALUES (?1, ?2, ?3)",
            params![document_node, search_content, search_hash.as_bytes()],
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
        let search_content = searchable_markdown(&file.content[span.bytes.clone()]);
        let search_hash = blake3::hash(search_content.as_bytes());
        transaction.execute(
            "INSERT INTO search_entries(node, content, content_hash) VALUES (?1, ?2, ?3)",
            params![canonical_id, search_content, search_hash.as_bytes()],
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
    for entity in external {
        let record = &entity.record;
        let attributes = record
            .attributes
            .iter()
            .map(|(name, value)| format!("{name}: {value}"))
            .collect::<Vec<_>>()
            .join("\n");
        let content = format!(
            "{}\n{}\nstate: {}\nauthor: {}\n{}",
            record.title,
            record.body,
            record.state,
            record.author.as_deref().unwrap_or_default(),
            attributes,
        );
        let content_hash = blake3::hash(content.as_bytes());
        transaction.execute(
            "INSERT INTO search_entries(node, content, content_hash) VALUES (?1, ?2, ?3)",
            params![record.identity, content, content_hash.as_bytes()],
        )?;
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
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn persists_graph_locations_properties_and_search_content() {
        let content = "+++\nid = \"spec:structured-only-token\"\ntype = \"spec\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# Retry policy\nExponential backoff protects the service.\n".to_owned();
        let path = PathBuf::from("docs/retry.md");
        let span = SourceSpan::from_offsets(&content, 0..content.len());
        let hash = *blake3::hash(content.as_bytes()).as_bytes();
        let fingerprint = RepositoryFingerprint::from_hex(&"1".repeat(64)).unwrap();
        let parsed = ParsedDocument::parse(&content).unwrap();
        let section_span = parsed.headings[0].section_span.clone();
        let corpus = CanonicalCorpus {
            files: vec![CorpusFile {
                path: path.clone(),
                content: content.clone(),
                content_hash: hash,
                document: parsed,
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
                path: path.clone(),
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
                location: GraphLocation {
                    path: path.clone(),
                    span: section_span.clone(),
                },
                content_hash: *blake3::hash(content[section_span.bytes.clone()].as_bytes())
                    .as_bytes(),
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
        let structured_hits = search(&database, "structured-only-token", 5).unwrap();

        assert_eq!(recorded_fingerprint(&database).unwrap(), Some(fingerprint));
        assert_eq!(entity_count, 1);
        assert_eq!(relation_location, ("docs/retry.md".to_owned(), 1));
        assert_eq!(hits[0].node, "spec:retry");
        assert!(hits[0].snippet.contains("Exponential backoff"));
        assert!(structured_hits.is_empty());

        let external_database = database.with_extension("external.sqlite");
        let external = DerivedExternalEntity {
            record: crate::ExternalEntityRecord {
                identity: "github:issue:github.test/owner/repo:7".to_owned(),
                provider: "github".to_owned(),
                remote_kind: "issue".to_owned(),
                title: "External retry defect".to_owned(),
                body: "Remote-only backpressure token".to_owned(),
                state: "open".to_owned(),
                author: Some("octo".to_owned()),
                created_at: None,
                updated_at: None,
                url: "https://github.test/owner/repo/issues/7".to_owned(),
                attributes: BTreeMap::new(),
            },
            fetched_at: 100,
            freshness: crate::ExternalFreshness::Fresh,
            provider_version: None,
        };
        build_with_external(
            &external_database,
            fingerprint,
            &corpus,
            &graph,
            std::slice::from_ref(&external),
        )
        .unwrap();
        let external_hits = search(&external_database, "backpressure", 5).unwrap();
        assert_eq!(external_hits[0].node, external.record.identity);
        assert_eq!(
            recorded_external_fingerprint(&external_database).unwrap(),
            Some(external_fingerprint(&[external]))
        );

        fs::remove_file(database).unwrap();
        fs::remove_file(external_database).unwrap();
    }

    struct RecordingProvider {
        batches: RefCell<Vec<Vec<String>>>,
    }

    impl RecordingProvider {
        fn new() -> Self {
            Self {
                batches: RefCell::new(Vec::new()),
            }
        }
    }

    impl EmbeddingProvider for RecordingProvider {
        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            self.batches.borrow_mut().push(texts.to_vec());
            Ok(texts
                .iter()
                .map(|text| {
                    if text.to_ascii_lowercase().contains("backoff") {
                        vec![1.0, 0.0]
                    } else {
                        vec![0.0, 1.0]
                    }
                })
                .collect())
        }
    }

    #[test]
    fn reuses_unchanged_vectors_and_returns_ranked_semantic_hits() {
        let content = "+++\nid = \"spec:retry\"\ntype = \"spec\"\nstate = \"open\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# Retry policy\nExponential backoff protects the service.\n".to_owned();
        let path = PathBuf::from("docs/retry.md");
        let parsed = ParsedDocument::parse(&content).unwrap();
        let span = parsed.headings[0].section_span.clone();
        let hash = *blake3::hash(content.as_bytes()).as_bytes();
        let section_hash = *blake3::hash(content[span.bytes.clone()].as_bytes()).as_bytes();
        let fingerprint = RepositoryFingerprint::from_hex(&"2".repeat(64)).unwrap();
        let corpus = CanonicalCorpus {
            files: vec![CorpusFile {
                path: path.clone(),
                content: content.clone(),
                content_hash: hash,
                document: parsed,
            }],
            fingerprint,
        };
        let graph = GraphIndex {
            documents: vec![DocumentNode {
                path: path.clone(),
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
                location: GraphLocation { path, span },
                content_hash: section_hash,
            }],
            relations: Vec::new(),
            diagnostics: Vec::new(),
        };
        let config = EmbeddingConfig {
            provider: "test".to_owned(),
            model: "two-dimensional".to_owned(),
            dimensions: 2,
            command: vec!["unused".to_owned()],
            batch_size: 32,
            timeout_seconds: 30,
            fallback: EmbeddingFallback::FullText,
        };
        let directory = std::env::temp_dir();
        let first = directory.join(format!(
            "docgraph-vector-first-{}.sqlite",
            std::process::id()
        ));
        let second = directory.join(format!(
            "docgraph-vector-second-{}.sqlite",
            std::process::id()
        ));
        let _ = fs::remove_file(&first);
        let _ = fs::remove_file(&second);
        let provider = RecordingProvider::new();

        build(&first, fingerprint, &corpus, &graph).unwrap();
        index_vectors(&first, None, &config, &provider).unwrap();
        assert_eq!(provider.batches.borrow().len(), 1);
        assert!(
            provider.batches.borrow()[0]
                .iter()
                .all(|text| !text.contains("state =") && !text.contains("s-83JRT4K2P6"))
        );

        provider.batches.borrow_mut().clear();
        let changed_content = content.replace("state = \"open\"", "state = \"done\"");
        let changed_hash = *blake3::hash(changed_content.as_bytes()).as_bytes();
        let changed_fingerprint = RepositoryFingerprint::from_hex(&"3".repeat(64)).unwrap();
        let mut changed_corpus = corpus.clone();
        changed_corpus.fingerprint = changed_fingerprint;
        changed_corpus.files[0].content = changed_content.clone();
        changed_corpus.files[0].content_hash = changed_hash;
        changed_corpus.files[0].document = ParsedDocument::parse(&changed_content).unwrap();
        let mut changed_graph = graph.clone();
        changed_graph.documents[0].content_hash = changed_hash;

        build(
            &second,
            changed_fingerprint,
            &changed_corpus,
            &changed_graph,
        )
        .unwrap();
        index_vectors(&second, Some(&first), &config, &provider).unwrap();
        assert!(provider.batches.borrow().is_empty());

        let result = semantic_search(&second, "backoff", 1, &config, &provider).unwrap();
        assert_eq!(result.mode, SemanticSearchMode::Vector);
        assert_eq!(result.hits[0].node, "spec:retry");
        assert_eq!(result.hits[0].score, 1.0);

        record_vector_failure(
            &second,
            &EmbeddingError::Unavailable("provider is offline".to_owned()),
        )
        .unwrap();
        let fallback = semantic_search(&second, "backoff", 1, &config, &provider).unwrap();
        assert_eq!(fallback.mode, SemanticSearchMode::FullTextFallback);
        assert!(fallback.reason.unwrap().contains("provider is offline"));
        assert_eq!(fallback.hits[0].node, "spec:retry");

        fs::remove_file(first).unwrap();
        fs::remove_file(second).unwrap();
    }
}
