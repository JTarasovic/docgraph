use crate::{
    CanonicalCorpus, CorpusFile, DerivedState, GraphIndex, Repository, RepositoryConfig,
    RepositoryFingerprint, Validator, sync_generated_frontmatter,
};
use docgraph_markdown::{ParsedDocument, StableSectionId, normalize_sections_with_reserved_random};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};

#[derive(Clone, Debug)]
pub enum MutationRequest {
    Transition {
        entity: String,
        target_state: String,
    },
    AddRelation {
        source: String,
        predicate: String,
        target: String,
        properties: BTreeMap<String, toml_edit::Value>,
    },
    RemoveRelation {
        source: String,
        predicate: String,
        target: String,
    },
    Normalize,
    SyncFrontmatter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileChange {
    pub path: PathBuf,
    pub original: String,
    pub intended: String,
    pub original_hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutationPlan {
    pub fingerprint: RepositoryFingerprint,
    pub changes: Vec<FileChange>,
}

impl MutationPlan {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

pub struct MutationService {
    repository: Repository,
    config: RepositoryConfig,
    state: DerivedState,
}

impl MutationService {
    pub fn open(start: impl AsRef<Path>) -> Result<Self, MutationError> {
        let repository = Repository::discover(start)
            .map_err(|error| MutationError::Repository(error.to_string()))?;
        let config = RepositoryConfig::load(&repository)
            .map_err(|error| MutationError::Configuration(error.to_string()))?;
        let state = DerivedState::discover(&repository)
            .map_err(|error| MutationError::State(error.to_string()))?;
        Ok(Self {
            repository,
            config,
            state,
        })
    }

    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    pub fn config(&self) -> &RepositoryConfig {
        &self.config
    }

    pub fn plan(&self, request: &MutationRequest) -> Result<MutationPlan, MutationError> {
        let corpus = CanonicalCorpus::load(&self.repository, &self.config)
            .map_err(|error| MutationError::Corpus(error.to_string()))?;
        self.plan_against(request, &corpus)
    }

    pub fn apply(
        &self,
        request: &MutationRequest,
        dry_run: bool,
    ) -> Result<MutationPlan, MutationError> {
        if dry_run {
            return self.plan(request);
        }
        fs::create_dir_all(&self.state.paths.directory)
            .map_err(|source| MutationError::io(&self.state.paths.directory, source))?;
        let _lock = MutationLock::acquire(&self.state.paths.mutation_lock)?;
        self.recover_if_needed()?;

        // Re-plan under the lock. This is the bounded retry for canonical input
        // changes that occurred between inspection and mutation.
        let current = CanonicalCorpus::load(&self.repository, &self.config)
            .map_err(|error| MutationError::Corpus(error.to_string()))?;
        let plan = self.plan_against(request, &current)?;
        if plan.is_empty() {
            return Ok(plan);
        }
        for change in &plan.changes {
            let absolute = self.repository.root().join(&change.path);
            let current = fs::read_to_string(&absolute)
                .map_err(|source| MutationError::io(&absolute, source))?;
            if *blake3::hash(current.as_bytes()).as_bytes() != change.original_hash {
                return Err(MutationError::ConcurrentEdit(change.path.clone()));
            }
        }

        let journal = Journal::from_plan(&plan);
        let journal_source = toml_edit::ser::to_string(&journal)
            .map_err(|error| MutationError::Journal(error.to_string()))?;
        fs::write(&self.state.paths.recovery_journal, journal_source)
            .map_err(|source| MutationError::io(&self.state.paths.recovery_journal, source))?;
        let before_write = CanonicalCorpus::load(&self.repository, &self.config)
            .map_err(|error| MutationError::Corpus(error.to_string()))?;
        if before_write.fingerprint != plan.fingerprint {
            fs::remove_file(&self.state.paths.recovery_journal)
                .map_err(|source| MutationError::io(&self.state.paths.recovery_journal, source))?;
            return Err(MutationError::CanonicalInputsChanged);
        }
        for change in &plan.changes {
            let absolute = self.repository.root().join(&change.path);
            let current = fs::read_to_string(&absolute)
                .map_err(|source| MutationError::io(&absolute, source))?;
            if *blake3::hash(current.as_bytes()).as_bytes() != change.original_hash {
                return Err(MutationError::ConcurrentEdit(change.path.clone()));
            }
            replace_file(&absolute, &change.intended)?;
        }
        fs::remove_file(&self.state.paths.recovery_journal)
            .map_err(|source| MutationError::io(&self.state.paths.recovery_journal, source))?;

        let refreshed = CanonicalCorpus::load(&self.repository, &self.config)
            .map_err(|error| MutationError::Corpus(error.to_string()))?;
        fs::write(&self.state.paths.index, b"docgraph-derived-index-v1\n")
            .map_err(|source| MutationError::io(&self.state.paths.index, source))?;
        self.state
            .record(refreshed.fingerprint)
            .map_err(|error| MutationError::State(error.to_string()))?;
        Ok(plan)
    }

    fn plan_against(
        &self,
        request: &MutationRequest,
        corpus: &CanonicalCorpus,
    ) -> Result<MutationPlan, MutationError> {
        let graph = GraphIndex::build(corpus, &self.config);
        let mut contents: BTreeMap<PathBuf, String> = corpus
            .files
            .iter()
            .map(|file| (file.path.clone(), file.content.clone()))
            .collect();
        match request {
            MutationRequest::Transition {
                entity,
                target_state,
            } => {
                let node = unique_entity(&graph, entity)?;
                let entity_config =
                    self.config.entities.get(&node.entity_type).ok_or_else(|| {
                        MutationError::InvalidRequest(format!("entity {entity:?} has unknown type"))
                    })?;
                let workflow_name = entity_config.workflow.as_deref().ok_or_else(|| {
                    MutationError::InvalidRequest(format!("entity {entity:?} has no workflow"))
                })?;
                let workflow = self.config.workflows.get(workflow_name).ok_or_else(|| {
                    MutationError::InvalidRequest(format!(
                        "workflow {workflow_name:?} does not exist"
                    ))
                })?;
                let current = node.state.as_deref().unwrap_or(&workflow.initial);
                let state = workflow.states.get(current).ok_or_else(|| {
                    MutationError::InvalidRequest(format!(
                        "entity {entity:?} has invalid state {current:?}"
                    ))
                })?;
                if !state
                    .transitions
                    .iter()
                    .any(|target| target == target_state)
                {
                    return Err(MutationError::InvalidTransition {
                        entity: entity.clone(),
                        from: current.to_owned(),
                        to: target_state.clone(),
                    });
                }
                let path = &graph.documents[node.document].path;
                let source = contents.get(path).expect("corpus content exists");
                let edited = edit_document(source, |document| {
                    document[&self.config.project.frontmatter.state] = value(target_state.clone());
                    Ok(())
                })?;
                contents.insert(path.clone(), edited);
            }
            MutationRequest::AddRelation {
                source,
                predicate,
                target,
                properties,
            } => {
                let node = unique_entity(&graph, source)?;
                let path = &graph.documents[node.document].path;
                let input = contents.get(path).expect("corpus content exists");
                let edited = edit_document(input, |document| {
                    let generated_position = document
                        .get("docgraph_generated")
                        .and_then(Item::as_table)
                        .and_then(Table::position);
                    let relations =
                        relations_mut(document, &self.config.project.frontmatter.relations)?;
                    let mut relation = Table::new();
                    relation.set_position(generated_position.map(|position| position - 1));
                    relation.insert("type", value(predicate.clone()));
                    relation.insert("target", value(target.clone()));
                    for (key, property) in properties {
                        relation.insert(key, Item::Value(property.clone()));
                    }
                    relations.push(relation);
                    Ok(())
                })?;
                contents.insert(path.clone(), edited);
            }
            MutationRequest::RemoveRelation {
                source,
                predicate,
                target,
            } => {
                let node = unique_entity(&graph, source)?;
                let path = &graph.documents[node.document].path;
                let input = contents.get(path).expect("corpus content exists");
                let edited = edit_document(input, |document| {
                    let relations =
                        relations_mut(document, &self.config.project.frontmatter.relations)?;
                    let matches: Vec<_> = relations
                        .iter()
                        .enumerate()
                        .filter(|(_, relation)| {
                            relation.get("type").and_then(Item::as_str) == Some(predicate)
                                && relation.get("target").and_then(Item::as_str) == Some(target)
                        })
                        .map(|(index, _)| index)
                        .collect();
                    if matches.len() != 1 {
                        return Err(MutationError::InvalidRequest(format!(
                            "expected one {predicate:?} relation from {source:?} to {target:?}, found {}",
                            matches.len()
                        )));
                    }
                    relations.remove(matches[0]);
                    Ok(())
                })?;
                contents.insert(path.clone(), edited);
            }
            MutationRequest::Normalize => {
                let mut reserved: BTreeSet<StableSectionId> = graph
                    .sections
                    .iter()
                    .filter_map(|section| section.id.clone())
                    .collect();
                for file in &corpus.files {
                    let normalized = normalize_sections_with_reserved_random(
                        contents.get(&file.path).expect("corpus content exists"),
                        reserved.clone(),
                    )
                    .map_err(|error| MutationError::InvalidRequest(error.to_string()))?;
                    reserved.extend(
                        normalized
                            .inserted
                            .iter()
                            .map(|insertion| insertion.id.clone()),
                    );
                    contents.insert(file.path.clone(), normalized.content);
                }
            }
            MutationRequest::SyncFrontmatter => {}
        }

        let mut projections_converged = false;
        for _ in 0..3 {
            let snapshot = candidate_corpus(corpus, &contents)?;
            let snapshot_graph = GraphIndex::build(&snapshot, &self.config);
            let mut updates = Vec::new();
            for (document, node) in snapshot_graph.documents.iter().enumerate() {
                if node.entity.is_none() {
                    continue;
                }
                let source = contents.get(&node.path).expect("corpus content exists");
                let synced =
                    sync_generated_frontmatter(source, &snapshot_graph, &self.config, document)?;
                if synced != *source {
                    updates.push((node.path.clone(), synced));
                }
            }
            if updates.is_empty() {
                projections_converged = true;
                break;
            }
            contents.extend(updates);
        }
        if !projections_converged {
            return Err(MutationError::InvalidRequest(
                "generated frontmatter did not converge".to_owned(),
            ));
        }

        let candidate = candidate_corpus(corpus, &contents)?;
        let candidate_graph = GraphIndex::build(&candidate, &self.config);
        let report = Validator::validate_corpus(
            &self.repository,
            &self.config,
            &candidate,
            &candidate_graph,
        );
        if !report.is_valid() {
            return Err(MutationError::ProspectiveValidation(
                report
                    .errors()
                    .map(|diagnostic| {
                        format!(
                            "{}: {}: {}",
                            diagnostic.location.path.display(),
                            diagnostic.code,
                            diagnostic.message
                        )
                    })
                    .collect(),
            ));
        }
        let changes = corpus
            .files
            .iter()
            .filter_map(|file| {
                let intended = contents.get(&file.path).expect("corpus content exists");
                (intended != &file.content).then(|| FileChange {
                    path: file.path.clone(),
                    original: file.content.clone(),
                    intended: intended.clone(),
                    original_hash: file.content_hash,
                })
            })
            .collect();
        Ok(MutationPlan {
            fingerprint: corpus.fingerprint,
            changes,
        })
    }

    fn recover_if_needed(&self) -> Result<(), MutationError> {
        let source = match fs::read_to_string(&self.state.paths.recovery_journal) {
            Ok(source) => source,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(MutationError::io(
                    &self.state.paths.recovery_journal,
                    source,
                ));
            }
        };
        let journal: Journal = toml_edit::de::from_str(&source)
            .map_err(|error| MutationError::Journal(error.to_string()))?;
        let mut unknown = Vec::new();
        for file in &journal.file {
            let absolute = self.repository.root().join(&file.path);
            let current = fs::read_to_string(&absolute)
                .map_err(|source| MutationError::io(&absolute, source))?;
            if current != file.original && current != file.intended {
                unknown.push(file.path.clone());
            }
        }
        if !unknown.is_empty() {
            return Err(MutationError::RecoveryConflict(unknown));
        }
        let corpus = CanonicalCorpus::load(&self.repository, &self.config)
            .map_err(|error| MutationError::Corpus(error.to_string()))?;
        let mut contents: BTreeMap<_, _> = corpus
            .files
            .iter()
            .map(|file| (file.path.clone(), file.content.clone()))
            .collect();
        for file in &journal.file {
            contents.insert(file.path.clone(), file.intended.clone());
        }
        let candidate = candidate_corpus(&corpus, &contents)?;
        let graph = GraphIndex::build(&candidate, &self.config);
        let report = Validator::validate_corpus(&self.repository, &self.config, &candidate, &graph);
        if !report.is_valid() {
            return Err(MutationError::ProspectiveValidation(
                report
                    .errors()
                    .map(|diagnostic| diagnostic.message.clone())
                    .collect(),
            ));
        }
        for file in &journal.file {
            let absolute = self.repository.root().join(&file.path);
            if fs::read_to_string(&absolute)
                .map_err(|source| MutationError::io(&absolute, source))?
                == file.original
            {
                replace_file(&absolute, &file.intended)?;
            }
        }
        fs::remove_file(&self.state.paths.recovery_journal)
            .map_err(|source| MutationError::io(&self.state.paths.recovery_journal, source))?;
        Ok(())
    }
}

fn unique_entity<'a>(
    graph: &'a GraphIndex,
    id: &str,
) -> Result<&'a crate::EntityNode, MutationError> {
    let matches: Vec<_> = graph
        .entities
        .iter()
        .filter(|entity| entity.id == id)
        .collect();
    match matches.as_slice() {
        [entity] => Ok(entity),
        [] => Err(MutationError::InvalidRequest(format!(
            "entity {id:?} does not exist"
        ))),
        _ => Err(MutationError::InvalidRequest(format!(
            "entity {id:?} is duplicated"
        ))),
    }
}

fn edit_document(
    source: &str,
    edit: impl FnOnce(&mut DocumentMut) -> Result<(), MutationError>,
) -> Result<String, MutationError> {
    let parsed = ParsedDocument::parse(source)
        .map_err(|error| MutationError::InvalidRequest(error.to_string()))?;
    let frontmatter = parsed.frontmatter.as_ref().ok_or_else(|| {
        MutationError::InvalidRequest("managed mutation requires TOML frontmatter".to_owned())
    })?;
    let mut document = frontmatter.to_mut();
    edit(&mut document)?;
    let mut output = source.to_owned();
    output.replace_range(
        frontmatter.content_span.bytes.clone(),
        &document.to_string(),
    );
    Ok(output)
}

fn relations_mut<'a>(
    document: &'a mut DocumentMut,
    field: &str,
) -> Result<&'a mut ArrayOfTables, MutationError> {
    if document.get(field).is_none() {
        document.insert(field, Item::ArrayOfTables(ArrayOfTables::new()));
    }
    document
        .get_mut(field)
        .and_then(Item::as_array_of_tables_mut)
        .ok_or_else(|| {
            MutationError::InvalidRequest(format!(
                "managed field {field:?} must be an array of tables"
            ))
        })
}

fn candidate_corpus(
    original: &CanonicalCorpus,
    contents: &BTreeMap<PathBuf, String>,
) -> Result<CanonicalCorpus, MutationError> {
    let files = original
        .files
        .iter()
        .map(|file| {
            let content = contents
                .get(&file.path)
                .expect("every corpus path has content")
                .clone();
            let document = ParsedDocument::parse(&content).map_err(|error| {
                MutationError::InvalidRequest(format!("{}: {error}", file.path.display()))
            })?;
            Ok(CorpusFile {
                path: file.path.clone(),
                content_hash: *blake3::hash(content.as_bytes()).as_bytes(),
                content,
                document,
            })
        })
        .collect::<Result<Vec<_>, MutationError>>()?;
    Ok(CanonicalCorpus {
        files,
        fingerprint: original.fingerprint,
    })
}

fn replace_file(path: &Path, intended: &str) -> Result<(), MutationError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("document");
    let temporary =
        path.with_file_name(format!(".{file_name}.docgraph-{}.tmp", std::process::id()));
    fs::write(&temporary, intended).map_err(|source| MutationError::io(&temporary, source))?;
    if let Err(source) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(MutationError::io(path, source));
    }
    Ok(())
}

struct MutationLock(PathBuf);

impl MutationLock {
    fn acquire(path: &Path) -> Result<Self, MutationError> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|source| {
                if source.kind() == io::ErrorKind::AlreadyExists {
                    MutationError::Locked(path.to_path_buf())
                } else {
                    MutationError::io(path, source)
                }
            })?;
        Ok(Self(path.to_path_buf()))
    }
}

impl Drop for MutationLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Journal {
    fingerprint: String,
    file: Vec<JournalFile>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JournalFile {
    path: PathBuf,
    original: String,
    intended: String,
}

impl Journal {
    fn from_plan(plan: &MutationPlan) -> Self {
        Self {
            fingerprint: plan.fingerprint.to_string(),
            file: plan
                .changes
                .iter()
                .map(|change| JournalFile {
                    path: change.path.clone(),
                    original: change.original.clone(),
                    intended: change.intended.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub enum MutationError {
    Repository(String),
    Configuration(String),
    Corpus(String),
    State(String),
    InvalidRequest(String),
    InvalidTransition {
        entity: String,
        from: String,
        to: String,
    },
    ProspectiveValidation(Vec<String>),
    ConcurrentEdit(PathBuf),
    CanonicalInputsChanged,
    Locked(PathBuf),
    RecoveryConflict(Vec<PathBuf>),
    Journal(String),
    Generated(crate::GeneratedBlockError),
    Io {
        path: PathBuf,
        source: io::Error,
    },
}

impl MutationError {
    fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

impl From<crate::GeneratedBlockError> for MutationError {
    fn from(error: crate::GeneratedBlockError) -> Self {
        Self::Generated(error)
    }
}

impl fmt::Display for MutationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(message)
            | Self::Configuration(message)
            | Self::Corpus(message)
            | Self::State(message)
            | Self::InvalidRequest(message)
            | Self::Journal(message) => formatter.write_str(message),
            Self::InvalidTransition { entity, from, to } => write!(
                formatter,
                "entity {entity:?} cannot transition from {from:?} to {to:?}"
            ),
            Self::ProspectiveValidation(errors) => write!(
                formatter,
                "prospective repository is invalid: {}",
                errors.join("; ")
            ),
            Self::ConcurrentEdit(path) => {
                write!(formatter, "{} changed during mutation", path.display())
            }
            Self::CanonicalInputsChanged => {
                formatter.write_str("canonical inputs changed during mutation; retry the operation")
            }
            Self::Locked(path) => write!(formatter, "another mutation holds {}", path.display()),
            Self::RecoveryConflict(paths) => write!(
                formatter,
                "recovery requires manual resolution for {paths:?}"
            ),
            Self::Generated(error) => error.fmt(formatter),
            Self::Io { path, source } => {
                write!(formatter, "cannot update {}: {source}", path.display())
            }
        }
    }
}

impl Error for MutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Generated(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "docgraph-mutation-test-{}-{sequence}",
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
                "[workflow.task]\ninitial = \"open\"\n[workflow.task.states.open]\ndescription = \"Open\"\ntransitions = [\"done\"]\n[workflow.task.states.done]\ndescription = \"Done\"\n",
            )
            .unwrap();
            fs::write(
                root.join(".docgraph/relations.toml"),
                "[relation.blocks]\ndescription = \"Blocks\"\nsource = [\"task\"]\ntarget = [\"task\"]\ninverse = \"blocked_by\"\n[relation.blocked_by]\ndescription = \"Blocked by\"\nsource = [\"task\"]\ntarget = [\"task\"]\ninverse = \"blocks\"\n",
            )
            .unwrap();
            fs::write(
                root.join("docs/one.md"),
                "+++\nid = \"task:1\"\ntype = \"task\"\nstate = \"open\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# One\n",
            )
            .unwrap();
            fs::write(
                root.join("docs/two.md"),
                "+++\nid = \"task:2\"\ntype = \"task\"\nstate = \"open\"\n+++\n<a id=\"s-7K3M9Q2W\"></a>\n# Two\n",
            )
            .unwrap();
            Self(root)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn dry_run_is_read_only_and_apply_transitions_atomically() {
        let fixture = Fixture::new();
        let service = MutationService::open(&fixture.0).unwrap();
        let request = MutationRequest::Transition {
            entity: "task:1".to_owned(),
            target_state: "done".to_owned(),
        };
        let before = fs::read_to_string(fixture.0.join("docs/one.md")).unwrap();

        let preview = service.apply(&request, true).unwrap();
        assert!(!preview.is_empty());
        assert_eq!(
            fs::read_to_string(fixture.0.join("docs/one.md")).unwrap(),
            before
        );

        service.apply(&request, false).unwrap();
        let after = fs::read_to_string(fixture.0.join("docs/one.md")).unwrap();
        assert!(after.contains("state = \"done\""));
        assert!(after.contains("# docgraph:generated:v1:begin"));
        assert!(!service.state.paths.recovery_journal.exists());
        assert!(service.state.paths.index.exists());
    }

    #[test]
    fn relation_mutation_refreshes_incoming_and_inverse_projections() {
        let fixture = Fixture::new();
        let service = MutationService::open(&fixture.0).unwrap();
        service
            .apply(
                &MutationRequest::AddRelation {
                    source: "task:1".to_owned(),
                    predicate: "blocks".to_owned(),
                    target: "task:2".to_owned(),
                    properties: BTreeMap::new(),
                },
                false,
            )
            .unwrap();

        let source = fs::read_to_string(fixture.0.join("docs/one.md")).unwrap();
        let target = fs::read_to_string(fixture.0.join("docs/two.md")).unwrap();
        assert!(source.contains("type = \"blocks\""));
        assert!(target.contains("predicate = \"blocks\""));
        assert!(target.contains("type = \"blocked_by\""));
        assert!(target.contains("target = \"task:1\""));

        service
            .apply(
                &MutationRequest::RemoveRelation {
                    source: "task:1".to_owned(),
                    predicate: "blocks".to_owned(),
                    target: "task:2".to_owned(),
                },
                false,
            )
            .unwrap();
        service
            .apply(
                &MutationRequest::AddRelation {
                    source: "task:1".to_owned(),
                    predicate: "blocks".to_owned(),
                    target: "task:2".to_owned(),
                    properties: BTreeMap::new(),
                },
                false,
            )
            .unwrap();
        let target = fs::read_to_string(fixture.0.join("docs/two.md")).unwrap();
        assert!(target.contains("predicate = \"blocks\""));
    }

    #[test]
    fn illegal_transition_never_writes() {
        let fixture = Fixture::new();
        let service = MutationService::open(&fixture.0).unwrap();
        let before = fs::read_to_string(fixture.0.join("docs/one.md")).unwrap();
        let error = service
            .apply(
                &MutationRequest::Transition {
                    entity: "task:1".to_owned(),
                    target_state: "missing".to_owned(),
                },
                false,
            )
            .unwrap_err();

        assert!(matches!(error, MutationError::InvalidTransition { .. }));
        assert_eq!(
            fs::read_to_string(fixture.0.join("docs/one.md")).unwrap(),
            before
        );
    }
}
