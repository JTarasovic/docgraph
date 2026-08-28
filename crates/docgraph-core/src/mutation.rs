use crate::{
    CanonicalCorpus, CorpusFile, DerivedState, GraphIndex, GraphNode, RelationOrigin, Repository,
    RepositoryConfig, RepositoryFingerprint, Validator, state::StateLock,
    sync_generated_frontmatter,
};
use docgraph_markdown::{
    ParsedDocument, StableSectionId, frame_content, normalize_sections_with_reserved_random,
};
use ignore::overrides::OverrideBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};

#[derive(Clone, Debug)]
pub struct Adoption {
    pub path: PathBuf,
    pub id: String,
    pub entity_type: String,
    pub properties: BTreeMap<String, toml_edit::Value>,
}

#[derive(Clone, Debug)]
pub enum MutationRequest {
    CreateDocument {
        path: PathBuf,
        id: String,
        entity_type: String,
        title: String,
        properties: BTreeMap<String, toml_edit::Value>,
    },
    MoveDocument {
        entity: String,
        path: PathBuf,
    },
    DeleteDocument {
        entity: String,
    },
    Adopt {
        path: PathBuf,
        id: String,
        entity_type: String,
        properties: BTreeMap<String, toml_edit::Value>,
    },
    AdoptBatch {
        documents: Vec<Adoption>,
    },
    Transition {
        entity: String,
        target_state: String,
    },
    InitializeWorkflow {
        entity_type: String,
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
    SetEntityProperty {
        entity: String,
        property: String,
        value: toml_edit::Value,
    },
    RemoveEntityProperty {
        entity: String,
        property: String,
    },
    Normalize,
    SyncFrontmatter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileChange {
    pub path: PathBuf,
    pub original: Option<String>,
    pub intended: Option<String>,
    pub original_hash: Option<[u8; 32]>,
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
        let _lock = acquire_state_lock(&self.state.paths.mutation_lock)?;
        self.recover_if_needed()?;

        // Re-plan under the lock. This is the bounded retry for canonical input
        // changes that occurred between inspection and mutation.
        let current = CanonicalCorpus::load(&self.repository, &self.config)
            .map_err(|error| MutationError::Corpus(error.to_string()))?;
        let plan = self.plan_against(request, &current)?;
        if plan.is_empty() {
            let graph = GraphIndex::build(&current, &self.config);
            self.state
                .ensure_fresh(&current, &graph)
                .map_err(|error| MutationError::State(error.to_string()))?;
            return Ok(plan);
        }
        for change in &plan.changes {
            let absolute = self.repository.root().join(&change.path);
            let current = read_optional_file(&absolute)?;
            if current
                .as_ref()
                .map(|content| *blake3::hash(content.as_bytes()).as_bytes())
                != change.original_hash
            {
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
            let current = read_optional_file(&absolute)?;
            if current
                .as_ref()
                .map(|content| *blake3::hash(content.as_bytes()).as_bytes())
                != change.original_hash
            {
                return Err(MutationError::ConcurrentEdit(change.path.clone()));
            }
            write_file_state(&absolute, change.intended.as_deref())?;
        }
        fs::remove_file(&self.state.paths.recovery_journal)
            .map_err(|source| MutationError::io(&self.state.paths.recovery_journal, source))?;

        let refreshed = CanonicalCorpus::load(&self.repository, &self.config)
            .map_err(|error| MutationError::Corpus(error.to_string()))?;
        let refreshed_graph = GraphIndex::build(&refreshed, &self.config);
        self.state
            .refresh(&refreshed, &refreshed_graph)
            .map_err(|error| MutationError::State(error.to_string()))?;
        Ok(plan)
    }

    pub fn recover_pending(&self) -> Result<(), MutationError> {
        fs::create_dir_all(&self.state.paths.directory)
            .map_err(|source| MutationError::io(&self.state.paths.directory, source))?;
        let _lock = acquire_state_lock(&self.state.paths.mutation_lock)?;
        self.recover_if_needed()
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
            MutationRequest::CreateDocument {
                path,
                id,
                entity_type,
                title,
                properties,
            } => {
                create_document(
                    &self.repository,
                    &self.config,
                    &graph,
                    &mut contents,
                    DocumentCreation {
                        path,
                        id,
                        entity_type,
                        title,
                        properties,
                    },
                )?;
            }
            MutationRequest::MoveDocument { entity, path } => {
                let node = unique_entity(&graph, entity)?;
                let source = graph.documents[node.document].path.clone();
                let target = managed_document_path(&self.repository, &self.config, path)?;
                if source == target {
                    return Err(MutationError::InvalidRequest(format!(
                        "entity {entity:?} is already at {}",
                        target.display()
                    )));
                }
                if contents.contains_key(&target) || self.repository.root().join(&target).exists() {
                    return Err(MutationError::InvalidRequest(format!(
                        "document {:?} already exists",
                        target.display()
                    )));
                }
                rewrite_relative_markdown_links(&graph, corpus, &mut contents, &source, &target)?;
                let content = contents.remove(&source).expect("entity document exists");
                contents.insert(target, content);
            }
            MutationRequest::DeleteDocument { entity } => {
                let node = unique_entity(&graph, entity)?;
                reject_inbound_references(&graph, node.document, entity)?;
                let path = graph.documents[node.document].path.clone();
                contents.remove(&path);
            }
            MutationRequest::Adopt {
                path,
                id,
                entity_type,
                properties,
            } => {
                adopt_documents(
                    &self.repository,
                    &self.config,
                    corpus,
                    &graph,
                    &mut contents,
                    std::slice::from_ref(&Adoption {
                        path: path.clone(),
                        id: id.clone(),
                        entity_type: entity_type.clone(),
                        properties: properties.clone(),
                    }),
                )?;
            }
            MutationRequest::AdoptBatch { documents } => {
                if documents.is_empty() {
                    return Err(MutationError::InvalidRequest(
                        "adoption batch is empty".to_owned(),
                    ));
                }
                adopt_documents(
                    &self.repository,
                    &self.config,
                    corpus,
                    &graph,
                    &mut contents,
                    documents,
                )?;
            }
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
            MutationRequest::InitializeWorkflow { entity_type } => {
                let entity_config = self.config.entities.get(entity_type).ok_or_else(|| {
                    MutationError::InvalidRequest(format!("unknown entity type {entity_type:?}"))
                })?;
                let workflow_name = entity_config.workflow.as_deref().ok_or_else(|| {
                    MutationError::InvalidRequest(format!(
                        "entity type {entity_type:?} has no workflow"
                    ))
                })?;
                let workflow = self.config.workflows.get(workflow_name).ok_or_else(|| {
                    MutationError::InvalidRequest(format!(
                        "workflow {workflow_name:?} does not exist"
                    ))
                })?;
                for node in graph
                    .entities
                    .iter()
                    .filter(|node| node.entity_type == *entity_type && node.state.is_none())
                {
                    let path = &graph.documents[node.document].path;
                    let source = contents.get(path).expect("corpus content exists");
                    let edited = edit_document(source, |document| {
                        document[&self.config.project.frontmatter.state] =
                            value(workflow.initial.clone());
                        Ok(())
                    })?;
                    contents.insert(path.clone(), edited);
                }
            }
            MutationRequest::AddRelation {
                source,
                predicate,
                target,
                properties,
            } => {
                let relation_source = relation_mutation_source(&graph, source)?;
                let path = &graph.documents[relation_source.document].path;
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
                    if relation_source.explicit {
                        relation.insert("source", value(source.clone()));
                    }
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
                let relation_source = relation_mutation_source(&graph, source)?;
                let path = &graph.documents[relation_source.document].path;
                let input = contents.get(path).expect("corpus content exists");
                let edited = edit_document(input, |document| {
                    let relations =
                        relations_mut(document, &self.config.project.frontmatter.relations)?;
                    let matches: Vec<_> = relations
                        .iter()
                        .enumerate()
                        .filter(|(_, relation)| {
                            let authored_source = relation.get("source").and_then(Item::as_str);
                            let source_matches = authored_source == Some(source)
                                || (!relation_source.explicit && authored_source.is_none());
                            source_matches
                                && relation.get("type").and_then(Item::as_str) == Some(predicate)
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
            MutationRequest::SetEntityProperty {
                entity,
                property,
                value,
            } => {
                let node = unique_entity(&graph, entity)?;
                let path = &graph.documents[node.document].path;
                let input = contents.get(path).expect("corpus content exists");
                let edited = edit_document(input, |document| {
                    properties_mut(document, &self.config.project.frontmatter.properties)?
                        .insert(property, Item::Value(value.clone()));
                    Ok(())
                })?;
                contents.insert(path.clone(), edited);
            }
            MutationRequest::RemoveEntityProperty { entity, property } => {
                let node = unique_entity(&graph, entity)?;
                let path = &graph.documents[node.document].path;
                let input = contents.get(path).expect("corpus content exists");
                let edited = edit_document(input, |document| {
                    let properties =
                        properties_mut(document, &self.config.project.frontmatter.properties)?;
                    if properties.remove(property).is_none() {
                        return Err(MutationError::InvalidRequest(format!(
                            "entity {entity:?} has no property {property:?}"
                        )));
                    }
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
        if matches!(request, MutationRequest::MoveDocument { .. })
            && explicit_relation_meaning(&graph) != explicit_relation_meaning(&candidate_graph)
        {
            return Err(MutationError::InvalidRequest(
                "move would change a managed relation; replace path-relative managed references with canonical entity or stable-section references first".to_owned(),
            ));
        }
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
        let originals: BTreeMap<_, _> = corpus
            .files
            .iter()
            .map(|file| (file.path.clone(), file))
            .collect();
        let paths: BTreeSet<_> = originals.keys().chain(contents.keys()).cloned().collect();
        let changes = paths
            .into_iter()
            .filter_map(|path| {
                let original = originals.get(&path).map(|file| file.content.clone());
                let intended = contents.get(&path).cloned();
                (original != intended).then(|| FileChange {
                    original_hash: originals.get(&path).map(|file| file.content_hash),
                    path,
                    original,
                    intended,
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
            let current = read_optional_file(&absolute)?;
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
            if let Some(intended) = &file.intended {
                contents.insert(file.path.clone(), intended.clone());
            } else {
                contents.remove(&file.path);
            }
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
            if read_optional_file(&absolute)? == file.original {
                write_file_state(&absolute, file.intended.as_deref())?;
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

struct RelationMutationSource {
    document: usize,
    explicit: bool,
}

fn relation_mutation_source(
    graph: &GraphIndex,
    reference: &str,
) -> Result<RelationMutationSource, MutationError> {
    let entities: Vec<_> = graph
        .entities
        .iter()
        .filter(|entity| entity.id == reference)
        .collect();
    if let [entity] = entities.as_slice() {
        return Ok(RelationMutationSource {
            document: entity.document,
            explicit: false,
        });
    }
    if entities.len() > 1 {
        return Err(MutationError::InvalidRequest(format!(
            "entity {reference:?} is duplicated"
        )));
    }
    let sections: Vec<_> = graph
        .sections
        .iter()
        .enumerate()
        .filter(|(index, _)| stable_section_reference(graph, *index).as_deref() == Some(reference))
        .collect();
    match sections.as_slice() {
        [(_, section)] => Ok(RelationMutationSource {
            document: section.document,
            explicit: true,
        }),
        [] => Err(MutationError::InvalidRequest(format!(
            "entity or stable section {reference:?} does not exist"
        ))),
        _ => Err(MutationError::InvalidRequest(format!(
            "stable section {reference:?} is duplicated"
        ))),
    }
}

fn stable_section_reference(graph: &GraphIndex, index: usize) -> Option<String> {
    let section = graph.sections.get(index)?;
    let id = section.id.as_ref()?;
    let document = graph.documents.get(section.document)?;
    Some(document.entity.as_ref().map_or_else(
        || format!("{}#{}", document.path.display(), id.as_str()),
        |entity| format!("{entity}#{}", id.as_str()),
    ))
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
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let rendered = frame_content(&document.to_string(), newline);
    let mut output = source.to_owned();
    output.replace_range(frontmatter.content_span.bytes.clone(), &rendered);
    Ok(output)
}

fn repository_relative_path(root: &Path, requested: &Path) -> Result<PathBuf, MutationError> {
    let relative = if requested.is_absolute() {
        requested.strip_prefix(root).map_err(|_| {
            MutationError::InvalidRequest(format!(
                "document {:?} is outside the repository",
                requested.display()
            ))
        })?
    } else {
        requested
    };
    let mut normalized = PathBuf::new();
    for component in relative.components() {
        match component {
            std::path::Component::Normal(component) => normalized.push(component),
            std::path::Component::CurDir => {}
            _ => {
                return Err(MutationError::InvalidRequest(format!(
                    "document path {:?} must stay within the repository",
                    requested.display()
                )));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(MutationError::InvalidRequest(
            "document path must not be empty".to_owned(),
        ));
    }
    Ok(normalized)
}

fn managed_document_path(
    repository: &Repository,
    config: &RepositoryConfig,
    requested: &Path,
) -> Result<PathBuf, MutationError> {
    let path = repository_relative_path(repository.root(), requested)?;
    let relative = path
        .strip_prefix(&config.project.documents.root)
        .map_err(|_| {
            MutationError::InvalidRequest(format!(
                "document {:?} is outside configured root {:?}",
                path.display(),
                config.project.documents.root.display()
            ))
        })?;
    let docs_root = repository.root().join(&config.project.documents.root);
    let mut overrides = OverrideBuilder::new(&docs_root);
    for include in &config.project.documents.include {
        overrides.add(include).map_err(|error| {
            MutationError::Configuration(format!("invalid document include {include:?}: {error}"))
        })?;
    }
    for exclude in &config.project.documents.exclude {
        overrides.add(&format!("!{exclude}")).map_err(|error| {
            MutationError::Configuration(format!("invalid document exclude {exclude:?}: {error}"))
        })?;
    }
    let overrides = overrides
        .build()
        .map_err(|error| MutationError::Configuration(error.to_string()))?;
    if config.project.documents.include.is_empty()
        || !overrides.matched(relative, false).is_whitelist()
    {
        return Err(MutationError::InvalidRequest(format!(
            "document {:?} is outside the configured corpus",
            path.display()
        )));
    }
    Ok(path)
}

struct DocumentCreation<'a> {
    path: &'a Path,
    id: &'a str,
    entity_type: &'a str,
    title: &'a str,
    properties: &'a BTreeMap<String, toml_edit::Value>,
}

fn create_document(
    repository: &Repository,
    config: &RepositoryConfig,
    graph: &GraphIndex,
    contents: &mut BTreeMap<PathBuf, String>,
    creation: DocumentCreation<'_>,
) -> Result<(), MutationError> {
    let DocumentCreation {
        path: requested,
        id,
        entity_type,
        title,
        properties,
    } = creation;
    if title.trim().is_empty() || title.contains('\r') || title.contains('\n') {
        return Err(MutationError::InvalidRequest(
            "document title must be a non-empty single line".to_owned(),
        ));
    }
    if graph.entities.iter().any(|entity| entity.id == id) {
        return Err(MutationError::InvalidRequest(format!(
            "entity {id:?} already exists"
        )));
    }
    let path = managed_document_path(repository, config, requested)?;
    if contents.contains_key(&path) || repository.root().join(&path).exists() {
        return Err(MutationError::InvalidRequest(format!(
            "document {:?} already exists",
            path.display()
        )));
    }
    let entity = config.entities.get(entity_type).ok_or_else(|| {
        MutationError::InvalidRequest(format!("unknown entity type {entity_type:?}"))
    })?;
    let workflow = entity
        .workflow
        .as_deref()
        .map(|name| {
            config.workflows.get(name).ok_or_else(|| {
                MutationError::InvalidRequest(format!(
                    "entity type {entity_type:?} references unknown workflow {name:?}"
                ))
            })
        })
        .transpose()?;
    let source = format!("# {}\n", title.trim());
    let adopted = adopt_document(
        &source,
        config,
        id,
        entity_type,
        workflow.map(|workflow| workflow.initial.as_str()),
        properties,
    )?;
    let reserved: BTreeSet<StableSectionId> = graph
        .sections
        .iter()
        .filter_map(|section| section.id.clone())
        .collect();
    let normalized = normalize_sections_with_reserved_random(&adopted, reserved)
        .map_err(|error| MutationError::InvalidRequest(error.to_string()))?;
    contents.insert(path, normalized.content);
    Ok(())
}

fn reject_inbound_references(
    graph: &GraphIndex,
    document: usize,
    entity: &str,
) -> Result<(), MutationError> {
    let inbound: Vec<_> = graph
        .relations
        .iter()
        .filter(|relation| graph_node_document(graph, &relation.target) == Some(document))
        .filter(|relation| graph_node_document(graph, &relation.source) != Some(document))
        .map(|relation| {
            format!(
                "{}:{} ({} {})",
                relation.location.path.display(),
                relation.location.span.start_line,
                match relation.origin {
                    RelationOrigin::Explicit => "managed",
                    RelationOrigin::MarkdownLink => "Markdown",
                },
                relation.predicate
            )
        })
        .collect();
    if inbound.is_empty() {
        Ok(())
    } else {
        Err(MutationError::InvalidRequest(format!(
            "cannot delete entity {entity:?}; inbound references remain at {}",
            inbound.join(", ")
        )))
    }
}

fn graph_node_document(graph: &GraphIndex, node: &GraphNode) -> Option<usize> {
    match node {
        GraphNode::Document(document) => Some(*document),
        GraphNode::Entity(id) => graph
            .entities
            .iter()
            .find(|entity| &entity.id == id)
            .map(|entity| entity.document),
        GraphNode::Section(section) => graph.sections.get(*section).map(|node| node.document),
        GraphNode::ExternalUri(_) | GraphNode::Unresolved(_) => None,
    }
}

fn rewrite_relative_markdown_links(
    graph: &GraphIndex,
    corpus: &CanonicalCorpus,
    contents: &mut BTreeMap<PathBuf, String>,
    source_path: &Path,
    target_path: &Path,
) -> Result<(), MutationError> {
    let moved_document = graph
        .documents
        .iter()
        .position(|document| document.path == source_path)
        .expect("moved document exists");
    let mut edits: BTreeMap<PathBuf, Vec<(std::ops::Range<usize>, String, String)>> =
        BTreeMap::new();
    for file in &corpus.files {
        for link in &file.document.links {
            let (base, fragment) = link
                .destination
                .split_once('#')
                .map_or((link.destination.as_str(), None), |(base, fragment)| {
                    (base, Some(fragment))
                });
            if !(base.starts_with("./") || base.starts_with("../")) {
                continue;
            }
            let relation = graph.relations.iter().find(|relation| {
                relation.origin == RelationOrigin::MarkdownLink
                    && relation.location.path == file.path
                    && relation.location.span == link.span
            });
            let target_document =
                relation.and_then(|relation| graph_node_document(graph, &relation.target));
            let source_moves = file.path == source_path;
            let target_moves = target_document == Some(moved_document);
            if !source_moves && !target_moves {
                continue;
            }
            let target_document = target_document.ok_or_else(|| {
                MutationError::InvalidRequest(format!(
                    "cannot safely move {}; relative link at {}:{} is unresolved",
                    source_path.display(),
                    file.path.display(),
                    link.span.start_line
                ))
            })?;
            let new_source = if source_moves {
                target_path
            } else {
                file.path.as_path()
            };
            let old_target = &graph.documents[target_document].path;
            let new_target = if target_moves {
                target_path
            } else {
                old_target.as_path()
            };
            let mut destination = relative_link_destination(new_source, new_target)?;
            if let Some(fragment) = fragment {
                destination.push('#');
                destination.push_str(fragment);
            }
            edits.entry(file.path.clone()).or_default().push((
                link.span.bytes.clone(),
                link.destination.clone(),
                destination,
            ));
        }
    }
    for (path, mut path_edits) in edits {
        let content = contents.get_mut(&path).expect("link source exists");
        path_edits.sort_by_key(|(span, _, _)| std::cmp::Reverse(span.start));
        for (span, old, new) in path_edits {
            let rendered = &content[span.clone()];
            let Some(offset) = rendered.rfind(&old) else {
                return Err(MutationError::InvalidRequest(format!(
                    "cannot safely rewrite path-relative link at {}",
                    path.display()
                )));
            };
            let start = span.start + offset;
            content.replace_range(start..start + old.len(), &new);
        }
    }
    Ok(())
}

fn relative_link_destination(source: &Path, target: &Path) -> Result<String, MutationError> {
    let source_parent = source.parent().unwrap_or_else(|| Path::new(""));
    let source_parts: Vec<_> = source_parent.components().collect();
    let target_parts: Vec<_> = target.components().collect();
    let common = source_parts
        .iter()
        .zip(&target_parts)
        .take_while(|(left, right)| left == right)
        .count();
    let mut parts = vec!["..".to_owned(); source_parts.len() - common];
    parts.extend(
        target_parts[common..]
            .iter()
            .map(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .ok_or_else(|| {
                        MutationError::InvalidRequest(
                            "document paths must be valid UTF-8".to_owned(),
                        )
                    })
                    .map(str::to_owned)
            })
            .collect::<Result<Vec<_>, _>>()?,
    );
    let relative = parts.join("/");
    Ok(if relative.starts_with("../") {
        relative
    } else {
        format!("./{relative}")
    })
}

fn explicit_relation_meaning(graph: &GraphIndex) -> BTreeSet<String> {
    graph
        .relations
        .iter()
        .filter(|relation| relation.origin == RelationOrigin::Explicit)
        .map(|relation| {
            format!(
                "{}\0{}\0{}\0{:?}",
                semantic_node_key(graph, &relation.source),
                relation.predicate,
                semantic_node_key(graph, &relation.target),
                relation.properties
            )
        })
        .collect()
}

fn semantic_node_key(graph: &GraphIndex, node: &GraphNode) -> String {
    match node {
        GraphNode::Document(document) => graph.documents[*document].entity.as_ref().map_or_else(
            || format!("document:{}", graph.documents[*document].path.display()),
            |entity| format!("entity:{entity}"),
        ),
        GraphNode::Entity(entity) => format!("entity:{entity}"),
        GraphNode::Section(section) => {
            let section = &graph.sections[*section];
            section.id.as_ref().map_or_else(
                || format!("section:{}:{}", section.document, section.heading),
                |id| format!("section:{id}"),
            )
        }
        GraphNode::ExternalUri(uri) => format!("external:{uri}"),
        GraphNode::Unresolved(reference) => format!("unresolved:{reference}"),
    }
}

fn adopt_documents(
    repository: &Repository,
    config: &RepositoryConfig,
    corpus: &CanonicalCorpus,
    graph: &GraphIndex,
    contents: &mut BTreeMap<PathBuf, String>,
    documents: &[Adoption],
) -> Result<(), MutationError> {
    let mut ids: BTreeSet<String> = graph
        .entities
        .iter()
        .map(|entity| entity.id.clone())
        .collect();
    let mut paths = BTreeSet::new();
    let mut reserved: BTreeSet<StableSectionId> = graph
        .sections
        .iter()
        .filter_map(|section| section.id.clone())
        .collect();

    for adoption in documents {
        let path = repository_relative_path(repository.root(), &adoption.path)?;
        if !paths.insert(path.clone()) {
            return Err(MutationError::InvalidRequest(format!(
                "document {:?} appears more than once in the adoption batch",
                path.display()
            )));
        }
        if !ids.insert(adoption.id.clone()) {
            return Err(MutationError::InvalidRequest(format!(
                "entity {:?} already exists or appears more than once in the adoption batch",
                adoption.id
            )));
        }
        let file = corpus
            .files
            .iter()
            .find(|file| file.path == path)
            .ok_or_else(|| {
                MutationError::InvalidRequest(format!(
                    "document {:?} is outside the configured corpus or does not exist",
                    path.display()
                ))
            })?;
        let entity_config = config.entities.get(&adoption.entity_type).ok_or_else(|| {
            MutationError::InvalidRequest(format!("unknown entity type {:?}", adoption.entity_type))
        })?;
        let workflow = entity_config
            .workflow
            .as_deref()
            .map(|name| {
                config.workflows.get(name).ok_or_else(|| {
                    MutationError::InvalidRequest(format!(
                        "entity type {:?} references unknown workflow {name:?}",
                        adoption.entity_type
                    ))
                })
            })
            .transpose()?;
        let source = contents.get(&file.path).expect("corpus content exists");
        let adopted = adopt_document(
            source,
            config,
            &adoption.id,
            &adoption.entity_type,
            workflow.map(|workflow| workflow.initial.as_str()),
            &adoption.properties,
        )?;
        let normalized = normalize_sections_with_reserved_random(&adopted, reserved.clone())
            .map_err(|error| MutationError::InvalidRequest(error.to_string()))?;
        reserved.extend(
            normalized
                .inserted
                .iter()
                .map(|insertion| insertion.id.clone()),
        );
        contents.insert(file.path.clone(), normalized.content);
    }
    Ok(())
}

fn adopt_document(
    source: &str,
    config: &RepositoryConfig,
    id: &str,
    entity_type: &str,
    initial_state: Option<&str>,
    properties: &BTreeMap<String, toml_edit::Value>,
) -> Result<String, MutationError> {
    let parsed = ParsedDocument::parse(source)
        .map_err(|error| MutationError::InvalidRequest(error.to_string()))?;
    let managed_fields = [
        config.project.frontmatter.id.as_str(),
        config.project.frontmatter.entity_type.as_str(),
        config.project.frontmatter.state.as_str(),
        config.project.frontmatter.relations.as_str(),
        config.project.frontmatter.properties.as_str(),
        "docgraph_generated",
    ];
    let mut document = parsed
        .frontmatter
        .as_ref()
        .map_or_else(DocumentMut::new, |frontmatter| frontmatter.to_mut());
    if let Some(field) = managed_fields
        .iter()
        .find(|field| document.get(field).is_some())
    {
        return Err(MutationError::InvalidRequest(format!(
            "document already contains managed frontmatter field {field:?}"
        )));
    }
    document[&config.project.frontmatter.id] = value(id);
    document[&config.project.frontmatter.entity_type] = value(entity_type);
    if let Some(state) = initial_state {
        document[&config.project.frontmatter.state] = value(state);
    }
    if !properties.is_empty() {
        let mut table = Table::new();
        for (name, property) in properties {
            table.insert(name, Item::Value(property.clone()));
        }
        document.insert(&config.project.frontmatter.properties, Item::Table(table));
    }

    if let Some(frontmatter) = &parsed.frontmatter {
        let newline = if source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let rendered = frame_content(&document.to_string(), newline);
        let mut output = source.to_owned();
        output.replace_range(frontmatter.content_span.bytes.clone(), &rendered);
        Ok(output)
    } else {
        let newline = if source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let frontmatter = document.to_string().replace('\n', newline);
        let frontmatter = frame_content(&frontmatter, newline);
        Ok(format!("+++{newline}{frontmatter}+++{newline}{source}"))
    }
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

fn properties_mut<'a>(
    document: &'a mut DocumentMut,
    field: &str,
) -> Result<&'a mut Table, MutationError> {
    if document.get(field).is_none() {
        let generated_position = document
            .get("docgraph_generated")
            .and_then(Item::as_table)
            .and_then(Table::position);
        let mut properties = Table::new();
        properties.set_position(generated_position.map(|position| position - 1));
        document.insert(field, Item::Table(properties));
    }
    document
        .get_mut(field)
        .and_then(Item::as_table_mut)
        .ok_or_else(|| {
            MutationError::InvalidRequest(format!("managed field {field:?} must be a table"))
        })
}

fn candidate_corpus(
    original: &CanonicalCorpus,
    contents: &BTreeMap<PathBuf, String>,
) -> Result<CanonicalCorpus, MutationError> {
    let files = contents
        .iter()
        .map(|(path, content)| {
            let document = ParsedDocument::parse(content).map_err(|error| {
                MutationError::InvalidRequest(format!("{}: {error}", path.display()))
            })?;
            Ok(CorpusFile {
                path: path.clone(),
                content_hash: *blake3::hash(content.as_bytes()).as_bytes(),
                content: content.clone(),
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| MutationError::io(parent, source))?;
    }
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

fn read_optional_file(path: &Path) -> Result<Option<String>, MutationError> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(MutationError::io(path, source)),
    }
}

fn write_file_state(path: &Path, intended: Option<&str>) -> Result<(), MutationError> {
    if let Some(intended) = intended {
        replace_file(path, intended)
    } else {
        fs::remove_file(path).map_err(|source| MutationError::io(path, source))
    }
}

fn acquire_state_lock(path: &Path) -> Result<StateLock, MutationError> {
    StateLock::acquire(path).map_err(|source| {
        if source.kind() == io::ErrorKind::WouldBlock {
            MutationError::Locked(path.to_path_buf())
        } else {
            MutationError::io(path, source)
        }
    })
}

#[derive(Debug, Deserialize, Serialize)]
struct Journal {
    fingerprint: String,
    file: Vec<JournalFile>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JournalFile {
    path: PathBuf,
    original: Option<String>,
    intended: Option<String>,
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
                "[relation.blocks]\ndescription = \"Blocks\"\nsource = [\"task\"]\ntarget = [\"task\"]\ninverse = \"blocked_by\"\n[relation.blocked_by]\ndescription = \"Blocked by\"\nsource = [\"task\"]\ntarget = [\"task\"]\ninverse = \"blocks\"\n[relation.cites]\ndescription = \"Cites\"\nsource = [\"section\"]\ntarget = [\"section\"]\ninverse = \"cited_by\"\n[relation.cited_by]\ndescription = \"Cited by\"\nsource = [\"section\"]\ntarget = [\"section\"]\ninverse = \"cites\"\n",
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
        assert!(after.contains("[docgraph_generated]\nschema_version = 1"));
        assert!(!service.state.paths.recovery_journal.exists());
        assert!(service.state.paths.index.exists());
    }

    #[test]
    fn adopt_preserves_prose_and_initializes_managed_frontmatter() {
        let fixture = Fixture::new();
        let path = fixture.0.join("docs/three.md");
        let original = "# Three\n\nExisting prose.\n";
        fs::write(&path, original).unwrap();
        let service = MutationService::open(&fixture.0).unwrap();
        let request = MutationRequest::Adopt {
            path: PathBuf::from("docs/three.md"),
            id: "task:3".to_owned(),
            entity_type: "task".to_owned(),
            properties: BTreeMap::new(),
        };

        let preview = service.apply(&request, true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        let adopted = preview
            .changes
            .iter()
            .find(|change| change.path == Path::new("docs/three.md"))
            .unwrap();
        assert!(
            adopted
                .intended
                .as_deref()
                .unwrap()
                .contains("id = \"task:3\"")
        );
        assert!(
            adopted
                .intended
                .as_deref()
                .unwrap()
                .contains("state = \"open\"")
        );

        service.apply(&request, false).unwrap();
        let adopted = fs::read_to_string(&path).unwrap();
        assert!(adopted.contains("# Three\n\nExisting prose.\n"));
        assert!(adopted.contains("<a id=\"s-"));
        assert!(adopted.contains("type = \"task\""));
        assert!(adopted.contains("[docgraph_generated]\nschema_version = 1"));
    }

    #[test]
    fn document_lifecycle_creates_moves_and_deletes_recoverably() {
        let fixture = Fixture::new();
        let service = MutationService::open(&fixture.0).unwrap();
        let create = MutationRequest::CreateDocument {
            path: PathBuf::from("docs/three.md"),
            id: "task:3".to_owned(),
            entity_type: "task".to_owned(),
            title: "Three".to_owned(),
            properties: BTreeMap::new(),
        };

        let preview = service.apply(&create, true).unwrap();
        let created = preview
            .changes
            .iter()
            .find(|change| change.path == Path::new("docs/three.md"))
            .unwrap();
        assert!(created.original.is_none());
        assert!(
            created
                .intended
                .as_deref()
                .unwrap()
                .contains("id = \"task:3\"")
        );
        assert!(!fixture.0.join("docs/three.md").exists());
        service.apply(&create, false).unwrap();
        assert!(fixture.0.join("docs/three.md").exists());
        let created_path = fixture.0.join("docs/three.md");
        let created = format!(
            "{}\nSee [one](./one.md).\n",
            fs::read_to_string(&created_path).unwrap()
        );
        fs::write(&created_path, created).unwrap();
        let one_path = fixture.0.join("docs/one.md");
        let one = format!(
            "{}\nSee [three](./three.md).\n",
            fs::read_to_string(&one_path).unwrap()
        );
        fs::write(&one_path, one).unwrap();

        let move_document = MutationRequest::MoveDocument {
            entity: "task:3".to_owned(),
            path: PathBuf::from("docs/archive/three.md"),
        };
        let preview = service.apply(&move_document, true).unwrap();
        assert_eq!(preview.changes.len(), 3);
        assert!(preview.changes.iter().any(|change| {
            change.path == Path::new("docs/three.md") && change.intended.is_none()
        }));
        assert!(preview.changes.iter().any(|change| {
            change.path == Path::new("docs/archive/three.md") && change.original.is_none()
        }));
        service.apply(&move_document, false).unwrap();
        assert!(!fixture.0.join("docs/three.md").exists());
        assert!(fixture.0.join("docs/archive/three.md").exists());
        assert!(
            fs::read_to_string(fixture.0.join("docs/archive/three.md"))
                .unwrap()
                .contains("[one](../one.md)")
        );
        let one = fs::read_to_string(&one_path).unwrap();
        assert!(one.contains("[three](./archive/three.md)"));
        fs::write(
            &one_path,
            one.replace("\nSee [three](./archive/three.md).\n", ""),
        )
        .unwrap();

        let delete = MutationRequest::DeleteDocument {
            entity: "task:3".to_owned(),
        };
        let preview = service.apply(&delete, true).unwrap();
        assert!(preview.changes.iter().any(|change| {
            change.path == Path::new("docs/archive/three.md") && change.intended.is_none()
        }));
        service.apply(&delete, false).unwrap();
        assert!(!fixture.0.join("docs/archive/three.md").exists());
        assert!(!service.state.paths.recovery_journal.exists());
    }

    #[test]
    fn recovery_rolls_forward_interrupted_create_and_delete() {
        let fixture = Fixture::new();
        let service = MutationService::open(&fixture.0).unwrap();
        fs::create_dir_all(&service.state.paths.directory).unwrap();
        let create = MutationRequest::CreateDocument {
            path: PathBuf::from("docs/three.md"),
            id: "task:3".to_owned(),
            entity_type: "task".to_owned(),
            title: "Three".to_owned(),
            properties: BTreeMap::new(),
        };
        let plan = service.plan(&create).unwrap();
        fs::write(
            &service.state.paths.recovery_journal,
            toml_edit::ser::to_string(&Journal::from_plan(&plan)).unwrap(),
        )
        .unwrap();
        let created = plan
            .changes
            .iter()
            .find(|change| change.path == Path::new("docs/three.md"))
            .unwrap();
        write_file_state(&fixture.0.join(&created.path), created.intended.as_deref()).unwrap();

        service.recover_pending().unwrap();
        assert!(fixture.0.join("docs/three.md").exists());
        assert!(!service.state.paths.recovery_journal.exists());

        let delete = MutationRequest::DeleteDocument {
            entity: "task:3".to_owned(),
        };
        let plan = service.plan(&delete).unwrap();
        fs::write(
            &service.state.paths.recovery_journal,
            toml_edit::ser::to_string(&Journal::from_plan(&plan)).unwrap(),
        )
        .unwrap();
        fs::remove_file(fixture.0.join("docs/three.md")).unwrap();

        service.recover_pending().unwrap();
        assert!(!fixture.0.join("docs/three.md").exists());
        assert!(!service.state.paths.recovery_journal.exists());
    }

    #[test]
    fn deletion_reports_inbound_references() {
        let fixture = Fixture::new();
        let one = fixture.0.join("docs/one.md");
        let source = fs::read_to_string(&one).unwrap().replace(
            "state = \"open\"\n",
            "state = \"open\"\n\n[[relations]]\ntype = \"blocks\"\ntarget = \"task:2\"\n",
        );
        fs::write(one, source).unwrap();
        let service = MutationService::open(&fixture.0).unwrap();

        let error = service
            .plan(&MutationRequest::DeleteDocument {
                entity: "task:2".to_owned(),
            })
            .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("inbound references remain"));
        assert!(message.contains("one.md"));
        assert!(message.contains("blocks"));
    }

    #[test]
    fn workflow_initialize_materializes_all_missing_initial_states_together() {
        let fixture = Fixture::new();
        for name in ["one.md", "two.md"] {
            let path = fixture.0.join("docs").join(name);
            let source = fs::read_to_string(&path).unwrap();
            fs::write(&path, source.replace("state = \"open\"\n", "")).unwrap();
        }
        let service = MutationService::open(&fixture.0).unwrap();
        let request = MutationRequest::InitializeWorkflow {
            entity_type: "task".to_owned(),
        };

        let preview = service.apply(&request, true).unwrap();
        assert_eq!(preview.changes.len(), 2);
        assert!(preview.changes.iter().all(|change| {
            change
                .intended
                .as_deref()
                .is_some_and(|content| content.contains("state = \"open\""))
        }));
        assert!(
            !fs::read_to_string(fixture.0.join("docs/one.md"))
                .unwrap()
                .contains("state = \"open\"")
        );

        service.apply(&request, false).unwrap();
        for name in ["one.md", "two.md"] {
            assert!(
                fs::read_to_string(fixture.0.join("docs").join(name))
                    .unwrap()
                    .contains("state = \"open\"")
            );
        }
        assert!(service.plan(&request).unwrap().is_empty());
    }

    #[test]
    fn batch_adoption_normalizes_and_validates_the_complete_candidate() {
        let fixture = Fixture::new();
        fs::write(fixture.0.join("docs/three.md"), "# Three\n").unwrap();
        fs::write(fixture.0.join("docs/four.md"), "# Four\n").unwrap();
        let service = MutationService::open(&fixture.0).unwrap();

        let single = service.plan(&MutationRequest::Adopt {
            path: PathBuf::from("docs/three.md"),
            id: "task:3".to_owned(),
            entity_type: "task".to_owned(),
            properties: BTreeMap::new(),
        });
        assert!(matches!(
            single,
            Err(MutationError::ProspectiveValidation(_))
        ));

        let request = MutationRequest::AdoptBatch {
            documents: vec![
                Adoption {
                    path: PathBuf::from("docs/three.md"),
                    id: "task:3".to_owned(),
                    entity_type: "task".to_owned(),
                    properties: BTreeMap::new(),
                },
                Adoption {
                    path: PathBuf::from("docs/four.md"),
                    id: "task:4".to_owned(),
                    entity_type: "task".to_owned(),
                    properties: BTreeMap::new(),
                },
            ],
        };

        let plan = service.apply(&request, false).unwrap();
        assert_eq!(plan.changes.len(), 4);
        for name in ["three.md", "four.md"] {
            let adopted = fs::read_to_string(fixture.0.join("docs").join(name)).unwrap();
            assert!(adopted.contains("type = \"task\""));
            assert!(adopted.contains("<a id=\"s-"));
        }
    }

    #[test]
    fn adopt_preserves_unmanaged_frontmatter_and_rejects_managed_collisions() {
        let fixture = Fixture::new();
        let path = fixture.0.join("docs/three.md");
        fs::write(&path, "+++\ntitle = \"Three\"\n+++\n# Three\n").unwrap();
        let service = MutationService::open(&fixture.0).unwrap();
        service
            .apply(
                &MutationRequest::Adopt {
                    path: PathBuf::from("docs/three.md"),
                    id: "task:3".to_owned(),
                    entity_type: "task".to_owned(),
                    properties: BTreeMap::new(),
                },
                false,
            )
            .unwrap();
        let adopted = fs::read_to_string(&path).unwrap();
        assert!(adopted.contains("title = \"Three\""));

        let error = service
            .apply(
                &MutationRequest::Adopt {
                    path: PathBuf::from("docs/three.md"),
                    id: "task:4".to_owned(),
                    entity_type: "task".to_owned(),
                    properties: BTreeMap::new(),
                },
                false,
            )
            .unwrap_err();
        assert!(error.to_string().contains("managed frontmatter field"));
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
    fn section_relation_mutation_preserves_both_exact_endpoints() {
        let fixture = Fixture::new();
        let second_path = fixture.0.join("docs/two.md");
        let mut second = fs::read_to_string(&second_path).unwrap();
        second.push_str("\n<a id=\"s-9H4K2M7Q8R\"></a>\n## Two details\n");
        fs::write(&second_path, second).unwrap();
        let service = MutationService::open(&fixture.0).unwrap();
        let source = "task:1#s-83JRT4K2P6";
        let first_target = "task:2#s-7K3M9Q2W";
        let second_target = "task:2#s-9H4K2M7Q8R";

        for target in [first_target, second_target] {
            service
                .apply(
                    &MutationRequest::AddRelation {
                        source: source.to_owned(),
                        predicate: "cites".to_owned(),
                        target: target.to_owned(),
                        properties: BTreeMap::new(),
                    },
                    false,
                )
                .unwrap();
        }

        let authored = fs::read_to_string(fixture.0.join("docs/one.md")).unwrap();
        let projected = fs::read_to_string(&second_path).unwrap();
        assert_eq!(
            authored.matches(&format!("source = \"{source}\"")).count(),
            2
        );
        assert!(authored.contains(&format!("target = \"{first_target}\"")));
        assert!(authored.contains(&format!("target = \"{second_target}\"")));
        for target in [first_target, second_target] {
            assert!(projected.contains(&format!(
                "source = \"{source}\"\npredicate = \"cites\"\ntarget = \"{target}\""
            )));
            assert!(projected.contains(&format!(
                "source = \"{target}\"\ntype = \"cited_by\"\ntarget = \"{source}\""
            )));
        }

        service
            .apply(
                &MutationRequest::RemoveRelation {
                    source: source.to_owned(),
                    predicate: "cites".to_owned(),
                    target: first_target.to_owned(),
                },
                false,
            )
            .unwrap();

        let authored = fs::read_to_string(fixture.0.join("docs/one.md")).unwrap();
        let projected = fs::read_to_string(second_path).unwrap();
        assert!(!authored.contains(&format!("target = \"{first_target}\"")));
        assert!(authored.contains(&format!("target = \"{second_target}\"")));
        assert!(!projected.contains(&format!(
            "predicate = \"cites\"\ntarget = \"{first_target}\""
        )));
        assert!(projected.contains(&format!(
            "predicate = \"cites\"\ntarget = \"{second_target}\""
        )));
    }

    #[test]
    fn recovery_rolls_forward_an_interrupted_multi_file_mutation() {
        let fixture = Fixture::new();
        let service = MutationService::open(&fixture.0).unwrap();
        let request = MutationRequest::AddRelation {
            source: "task:1".to_owned(),
            predicate: "blocks".to_owned(),
            target: "task:2".to_owned(),
            properties: BTreeMap::new(),
        };
        let plan = service.plan(&request).unwrap();
        assert!(plan.changes.len() >= 2);
        fs::create_dir_all(&service.state.paths.directory).unwrap();
        fs::write(
            &service.state.paths.recovery_journal,
            toml_edit::ser::to_string(&Journal::from_plan(&plan)).unwrap(),
        )
        .unwrap();
        let first = &plan.changes[0];
        fs::write(
            fixture.0.join(&first.path),
            first.intended.as_deref().unwrap(),
        )
        .unwrap();

        service.recover_pending().unwrap();

        for change in &plan.changes {
            assert_eq!(
                fs::read_to_string(fixture.0.join(&change.path)).unwrap(),
                change.intended.as_deref().unwrap()
            );
        }
        assert!(!service.state.paths.recovery_journal.exists());
    }

    #[test]
    fn recovery_refuses_unknown_file_state_without_overwriting_it() {
        let fixture = Fixture::new();
        let service = MutationService::open(&fixture.0).unwrap();
        let plan = service
            .plan(&MutationRequest::AddRelation {
                source: "task:1".to_owned(),
                predicate: "blocks".to_owned(),
                target: "task:2".to_owned(),
                properties: BTreeMap::new(),
            })
            .unwrap();
        fs::create_dir_all(&service.state.paths.directory).unwrap();
        fs::write(
            &service.state.paths.recovery_journal,
            toml_edit::ser::to_string(&Journal::from_plan(&plan)).unwrap(),
        )
        .unwrap();
        let conflict = &plan.changes[0];
        let manual = format!(
            "{}\nManual concurrent edit.\n",
            conflict.original.as_deref().unwrap()
        );
        fs::write(fixture.0.join(&conflict.path), &manual).unwrap();

        let error = service.recover_pending().unwrap_err();

        assert!(matches!(error, MutationError::RecoveryConflict(_)));
        assert_eq!(
            fs::read_to_string(fixture.0.join(&conflict.path)).unwrap(),
            manual
        );
        assert!(service.state.paths.recovery_journal.exists());
    }

    #[test]
    fn advisory_lock_ignores_stale_lock_files_and_rejects_live_owners() {
        let fixture = Fixture::new();
        let service = MutationService::open(&fixture.0).unwrap();
        fs::create_dir_all(&service.state.paths.directory).unwrap();
        fs::write(&service.state.paths.mutation_lock, "stale owner").unwrap();

        service.recover_pending().unwrap();
        let _owner = StateLock::acquire(&service.state.paths.mutation_lock).unwrap();
        let error = service.recover_pending().unwrap_err();

        assert!(matches!(error, MutationError::Locked(_)));
    }

    #[test]
    fn failed_index_refresh_leaves_canonical_changes_for_the_next_rebuild() {
        let fixture = Fixture::new();
        let service = MutationService::open(&fixture.0).unwrap();
        fs::create_dir_all(&service.state.paths.directory).unwrap();
        fs::create_dir_all(&service.state.paths.index).unwrap();
        let request = MutationRequest::Transition {
            entity: "task:1".to_owned(),
            target_state: "done".to_owned(),
        };

        assert!(service.apply(&request, false).is_err());
        assert!(
            fs::read_to_string(fixture.0.join("docs/one.md"))
                .unwrap()
                .contains("state = \"done\"")
        );
        assert!(!service.state.paths.recovery_journal.exists());

        fs::remove_dir(&service.state.paths.index).unwrap();
        let corpus = CanonicalCorpus::load(&service.repository, &service.config).unwrap();
        let graph = GraphIndex::build(&corpus, &service.config);
        service.state.ensure_fresh(&corpus, &graph).unwrap();
        assert_eq!(
            service.state.status(corpus.fingerprint).unwrap(),
            crate::IndexStatus::Fresh
        );
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
