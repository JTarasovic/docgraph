use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use docgraph_core::{
    Adoption, AgentInstructionsConfig, CanonicalCorpus, CommandConfig, CommandEmbeddingProvider,
    CommandOperation, DerivedExternalEntity, DerivedState, DiagnosticSeverity, DocumentsConfig,
    ExternalEntityCache, ExternalEntityService, ExternalEntityView, ExternalIdentity,
    FrontmatterConfig, GeneratedBlockStatus, GeneratedFrontmatterIndex, GithubExternalEntitySource,
    GraphIndex, GraphNode, GraphTraversal, InstructionService, InstructionStatus,
    ManagedChangeValidator, MutationPlan, MutationRequest, MutationService, PORTABLE_SKILL_PATH,
    PortableSkillService, PortableSkillStatus, ProjectConfig, PropertyConfig, PropertyType,
    QueryValueType, RelationOrigin, Repository, RepositoryConfig, SCHEMA_VERSION, ScalarValue,
    SemanticChange, SemanticChangeReviewer, SemanticSearchHit, SemanticSearchMode,
    SemanticSearchResult, SemanticSection, TraversalDirection, ValidationConfig, Validator,
};
use docgraph_logic::{QueryEngine, QueryValue};
use serde::Deserialize;
use serde_json::{Value as JsonValue, json};
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};
use std::time::SystemTime;
use toml_edit::Value;

#[derive(Parser)]
#[command(
    name = "docgraph",
    version,
    about = "Repository-native document graphs"
)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize or verify docgraph repository support.
    Init {
        /// Project name for a new configuration. Defaults to the Git worktree name.
        #[arg(long)]
        name: Option<String>,
        /// Document corpus root for a new configuration.
        #[arg(long, value_name = "PATH")]
        documents: Option<PathBuf>,
        /// Agent-instruction target for a new configuration. May be repeated.
        #[arg(long = "instruction-target", value_name = "PATH")]
        instruction_targets: Vec<PathBuf>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Create, move, or delete managed documents.
    Document {
        #[command(subcommand)]
        action: DocumentAction,
    },
    /// Split, merge, or delete stable sections.
    Section {
        #[command(subcommand)]
        action: SectionAction,
    },
    /// Adopt an existing document into the managed graph.
    Adopt {
        path: Option<PathBuf>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long = "type")]
        entity_type: Option<String>,
        #[arg(long = "property")]
        properties: Vec<String>,
        /// Adopt all documents declared in a TOML manifest.
        #[arg(long)]
        batch: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Describe the configured repository model.
    Describe {
        /// Emit every configured definition and project setting in one document.
        #[arg(long)]
        all: bool,
        #[arg(value_enum)]
        kind: Option<DescribeKind>,
        name: Option<String>,
    },
    /// Retrieve an entity or stable section and its direct graph context.
    Get { reference: String },
    /// Full-text search indexed documents and sections.
    Search {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Search by semantic similarity, with an explicit configured fallback.
    SemanticSearch {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Apply a configured workflow transition.
    Transition {
        entity: String,
        state: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Manage configured workflows.
    Workflow {
        #[command(subcommand)]
        action: WorkflowAction,
    },
    /// Set or remove a typed entity property.
    Property {
        #[command(subcommand)]
        action: PropertyAction,
    },
    /// Add an explicit managed relation.
    Relate {
        source: String,
        relation: String,
        target: String,
        #[arg(long = "property")]
        properties: Vec<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove an explicit managed relation.
    Unrelate {
        source: String,
        relation: String,
        target: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// List adjacent nodes and edge origins.
    Neighbors {
        reference: String,
        /// Include informational Markdown links.
        #[arg(long)]
        all: bool,
    },
    /// List direct incoming relations.
    Incoming {
        reference: String,
        /// Include informational Markdown links.
        #[arg(long)]
        all: bool,
    },
    /// List direct outgoing relations.
    Outgoing {
        reference: String,
        /// Include informational Markdown links.
        #[arg(long)]
        all: bool,
    },
    /// Traverse the graph to a bounded arbitrary depth.
    Traverse {
        reference: String,
        #[arg(long, value_enum, default_value_t = TraversalDirectionArg::Both)]
        direction: TraversalDirectionArg,
        #[arg(long, default_value_t = 1)]
        depth: usize,
        /// Include informational Markdown links.
        #[arg(long)]
        all: bool,
    },
    /// Assemble node details and relations around a graph reference.
    Context {
        reference: String,
        #[arg(long, default_value_t = 1)]
        depth: usize,
        /// Include informational Markdown links.
        #[arg(long)]
        all: bool,
    },
    /// Find the shortest graph path between canonical entities or stable sections.
    Path {
        source: String,
        target: String,
        /// Include informational Markdown links.
        #[arg(long)]
        all: bool,
    },
    /// Add stable IDs to headings that lack them.
    Normalize {
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate configuration, logic, and the complete canonical corpus.
    Validate {
        /// Reject managed changes not equivalent to supported operations since REF.
        #[arg(long, value_name = "REF")]
        changes: Option<String>,
    },
    /// Review graph-level changes from a Git state to the current worktree.
    Review {
        #[arg(value_name = "REF")]
        reference: String,
    },
    /// Execute a typed named query.
    Query {
        name: String,
        #[arg(long = "arg")]
        arguments: Vec<String>,
    },
    /// Manage generated agent instruction blocks.
    Instructions {
        #[command(subcommand)]
        action: InstructionAction,
    },
    /// Manage generated frontmatter projections.
    Frontmatter {
        #[command(subcommand)]
        action: FrontmatterAction,
    },
    #[command(external_subcommand)]
    Custom(Vec<String>),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DescribeKind {
    Type,
    Relation,
    Workflow,
    Query,
    Command,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TraversalDirectionArg {
    Incoming,
    Outgoing,
    Both,
}

impl From<TraversalDirectionArg> for TraversalDirection {
    fn from(direction: TraversalDirectionArg) -> Self {
        match direction {
            TraversalDirectionArg::Incoming => Self::Incoming,
            TraversalDirectionArg::Outgoing => Self::Outgoing,
            TraversalDirectionArg::Both => Self::Both,
        }
    }
}

#[derive(Subcommand)]
enum InstructionAction {
    Sync {
        #[arg(long)]
        dry_run: bool,
    },
    Check,
}

#[derive(Subcommand)]
enum FrontmatterAction {
    Sync {
        #[arg(long)]
        dry_run: bool,
    },
    Check,
}

#[derive(Subcommand)]
enum DocumentAction {
    /// Create a new managed document.
    Create {
        path: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long = "type")]
        entity_type: String,
        #[arg(long)]
        title: String,
        #[arg(long = "property")]
        properties: Vec<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Move a managed entity's document.
    Move {
        entity: String,
        path: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Delete a managed entity's document when no inbound references remain.
    Delete {
        entity: String,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum SectionAction {
    /// Split a section by inserting a same-level heading at an exact source line.
    Split {
        section: String,
        #[arg(long)]
        at_line: usize,
        #[arg(long)]
        title: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Merge a section into the immediately preceding sibling.
    Merge {
        section: String,
        #[arg(long)]
        into: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Delete a section and its descendants when no durable references remain.
    Delete {
        section: String,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum WorkflowAction {
    /// Materialize the initial state for every uninitialized entity of a type.
    Initialize {
        entity_type: String,
        #[arg(long)]
        dry_run: bool,
    },
}

struct Context {
    repository: Repository,
    config: RepositoryConfig,
    corpus: CanonicalCorpus,
    graph: GraphIndex,
}

impl Context {
    fn load() -> Result<Self, CliError> {
        let service = MutationService::open(".").map_err(CliError::boxed)?;
        service.recover_pending().map_err(CliError::boxed)?;
        let repository = service.repository().clone();
        let config = service.config().clone();
        let corpus = CanonicalCorpus::load(&repository, &config).map_err(CliError::boxed)?;
        let graph = GraphIndex::build(&corpus, &config);
        Ok(Self {
            repository,
            config,
            corpus,
            graph,
        })
    }

    fn ensure_derived(&self) -> Result<DerivedState, CliError> {
        let external = self.external_records(true)?;
        self.ensure_derived_with_external(&external)
    }

    fn ensure_derived_with_external(
        &self,
        external: &[DerivedExternalEntity],
    ) -> Result<DerivedState, CliError> {
        let state = DerivedState::discover(&self.repository).map_err(CliError::boxed)?;
        if let Some(config) = &self.config.project.embeddings {
            let provider = CommandEmbeddingProvider::new(config);
            state
                .ensure_fresh_with_external(
                    &self.corpus,
                    &self.graph,
                    external,
                    Some((config, &provider)),
                )
                .map_err(CliError::boxed)?;
        } else {
            state
                .ensure_fresh_with_external(&self.corpus, &self.graph, external, None)
                .map_err(CliError::boxed)?;
        }
        Ok(state)
    }

    fn external_service(&self) -> Result<ExternalEntityService, CliError> {
        let state = DerivedState::discover(&self.repository).map_err(CliError::boxed)?;
        Ok(ExternalEntityService::new(
            ExternalEntityCache::new(state.paths.external_cache),
            self.config.project.external_entities.cache_ttl_seconds,
        ))
    }

    fn external_records(&self, refresh: bool) -> Result<Vec<DerivedExternalEntity>, CliError> {
        let service = self.external_service()?;
        if refresh {
            for reference in &self.config.project.references {
                let Some(source_config) =
                    self.config
                        .project
                        .external_entities
                        .source
                        .iter()
                        .find(|source| {
                            source.provider == reference.provider
                                && source.host.eq_ignore_ascii_case(&reference.host)
                        })
                else {
                    continue;
                };
                if source_config.provider == "github"
                    && let Ok(source) = GithubExternalEntitySource::new(source_config)
                {
                    let _ = service.refresh_repository(reference, &source, SystemTime::now());
                }
            }
        }
        service.cached(SystemTime::now()).map_err(CliError::boxed)
    }

    fn resolve_external(
        &self,
        identity: &str,
        refresh: bool,
    ) -> Result<ExternalEntityView, CliError> {
        let service = self.external_service()?;
        let parsed = ExternalIdentity::parse(identity);
        let source = if refresh {
            parsed.as_ref().and_then(|identity| {
                self.config
                    .project
                    .external_entities
                    .source
                    .iter()
                    .find(|source| {
                        source.provider == identity.provider
                            && source.host.eq_ignore_ascii_case(&identity.host)
                    })
                    .and_then(|source| GithubExternalEntitySource::new(source).ok())
            })
        } else {
            None
        };
        Ok(service.resolve(
            identity,
            source
                .as_ref()
                .map(|source| source as &dyn docgraph_core::ExternalEntitySource),
            SystemTime::now(),
        ))
    }
}

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().collect();
    if arguments.len() == 2 && matches!(arguments[1].to_str(), Some("--help" | "-h" | "help")) {
        return match print_root_help() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        };
    }
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if !error.silent {
                eprintln!("error: {error}");
            }
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Command::Init {
            name,
            documents,
            instruction_targets,
            dry_run,
        } => initialize(
            name.as_deref(),
            documents.as_deref(),
            &instruction_targets,
            dry_run,
            cli.json,
        ),
        Command::Document { action } => match action {
            DocumentAction::Create {
                path,
                id,
                entity_type,
                title,
                properties,
                dry_run,
            } => {
                let service = MutationService::open(".").map_err(CliError::boxed)?;
                let entity = service.config().entities.get(&entity_type).ok_or_else(|| {
                    CliError::message(format!("unknown entity type {entity_type:?}"))
                })?;
                let properties = parse_properties(&properties, &entity.property, "entity")?;
                let plan = service
                    .apply(
                        &MutationRequest::CreateDocument {
                            path,
                            id,
                            entity_type,
                            title,
                            properties,
                        },
                        dry_run,
                    )
                    .map_err(CliError::boxed)?;
                print_plan(&plan, dry_run, cli.json)
            }
            DocumentAction::Move {
                entity,
                path,
                dry_run,
            } => mutate(
                MutationRequest::MoveDocument { entity, path },
                dry_run,
                cli.json,
            ),
            DocumentAction::Delete { entity, dry_run } => mutate(
                MutationRequest::DeleteDocument { entity },
                dry_run,
                cli.json,
            ),
        },
        Command::Section { action } => match action {
            SectionAction::Split {
                section,
                at_line,
                title,
                dry_run,
            } => mutate(
                MutationRequest::SplitSection {
                    section,
                    at_line,
                    title,
                },
                dry_run,
                cli.json,
            ),
            SectionAction::Merge {
                section,
                into,
                dry_run,
            } => mutate(
                MutationRequest::MergeSection { section, into },
                dry_run,
                cli.json,
            ),
            SectionAction::Delete { section, dry_run } => mutate(
                MutationRequest::DeleteSection { section },
                dry_run,
                cli.json,
            ),
        },
        Command::Adopt {
            path,
            id,
            entity_type,
            properties,
            batch,
            dry_run,
        } => {
            let service = MutationService::open(".").map_err(CliError::boxed)?;
            let request = if let Some(batch) = batch {
                if path.is_some() || id.is_some() || entity_type.is_some() || !properties.is_empty()
                {
                    return Err(CliError::message(
                        "--batch cannot be combined with a path, --id, --type, or --property",
                    ));
                }
                MutationRequest::AdoptBatch {
                    documents: read_adoption_manifest(&batch, service.config())?,
                }
            } else {
                let path =
                    path.ok_or_else(|| CliError::message("adopt requires a path or --batch"))?;
                let id = id.ok_or_else(|| CliError::message("adopt requires --id"))?;
                let entity_type =
                    entity_type.ok_or_else(|| CliError::message("adopt requires --type"))?;
                let entity = service.config().entities.get(&entity_type).ok_or_else(|| {
                    CliError::message(format!("unknown entity type {entity_type:?}"))
                })?;
                let properties = parse_properties(&properties, &entity.property, "entity")?;
                MutationRequest::Adopt {
                    path,
                    id,
                    entity_type,
                    properties,
                }
            };
            let plan = service.apply(&request, dry_run).map_err(CliError::boxed)?;
            print_plan(&plan, dry_run, cli.json)
        }
        Command::Describe { all, kind, name } => {
            let context = Context::load()?;
            describe(&context.config, all, kind, name.as_deref(), cli.json)
        }
        Command::Get { reference } => {
            let context = Context::load()?;
            get(&context, &reference, cli.json)
        }
        Command::Search { query, limit } => {
            let context = Context::load()?;
            let hits = context
                .ensure_derived()?
                .search(&query, limit)
                .map_err(CliError::boxed)?;
            if cli.json {
                let rows: Vec<_> = hits
                    .iter()
                    .map(|hit| {
                        json!({
                            "node": hit.node,
                            "score": hit.score,
                            "snippet": hit.snippet,
                            "external": external_hit_metadata(&context, &hit.node),
                        })
                    })
                    .collect();
                print_json(json!({ "query": query, "rows": rows }))
            } else {
                for hit in hits {
                    println!(
                        "{:.3}\t{}\t{}\t{}",
                        hit.score,
                        hit.node,
                        external_hit_label(&context, &hit.node),
                        hit.snippet
                    );
                }
                Ok(())
            }
        }
        Command::SemanticSearch { query, limit } => {
            let context = Context::load()?;
            let state = context.ensure_derived()?;
            let result = if let Some(config) = &context.config.project.embeddings {
                let provider = CommandEmbeddingProvider::new(config);
                state
                    .semantic_search(&query, limit, config, &provider)
                    .map_err(CliError::boxed)?
            } else {
                SemanticSearchResult {
                    mode: SemanticSearchMode::FullTextFallback,
                    reason: Some("no embedding provider is configured".to_owned()),
                    hits: state
                        .search(&query, limit)
                        .map_err(CliError::boxed)?
                        .into_iter()
                        .map(|hit| SemanticSearchHit {
                            node: hit.node,
                            score: hit.score,
                            snippet: hit.snippet,
                        })
                        .collect(),
                }
            };
            print_semantic_search(&context, &query, &result, cli.json)
        }
        Command::Transition {
            entity,
            state,
            dry_run,
        } => mutate(
            MutationRequest::Transition {
                entity,
                target_state: state,
            },
            dry_run,
            cli.json,
        ),
        Command::Workflow { action } => match action {
            WorkflowAction::Initialize {
                entity_type,
                dry_run,
            } => mutate(
                MutationRequest::InitializeWorkflow { entity_type },
                dry_run,
                cli.json,
            ),
        },
        Command::Property { action } => property(action, cli.json),
        Command::Relate {
            source,
            relation,
            target,
            properties,
            dry_run,
        } => {
            let service = MutationService::open(".").map_err(CliError::boxed)?;
            let relation_config = service
                .config()
                .relations
                .get(&relation)
                .ok_or_else(|| CliError::message(format!("unknown relation {relation:?}")))?;
            let properties = parse_properties(&properties, &relation_config.property, "relation")?;
            let plan = service
                .apply(
                    &MutationRequest::AddRelation {
                        source,
                        predicate: relation,
                        target,
                        properties,
                    },
                    dry_run,
                )
                .map_err(CliError::boxed)?;
            print_plan(&plan, dry_run, cli.json)
        }
        Command::Unrelate {
            source,
            relation,
            target,
            dry_run,
        } => mutate(
            MutationRequest::RemoveRelation {
                source,
                predicate: relation,
                target,
            },
            dry_run,
            cli.json,
        ),
        Command::Neighbors { reference, all } => {
            let context = Context::load()?;
            direct_relations(&context, &reference, None, all, cli.json)
        }
        Command::Incoming { reference, all } => {
            let context = Context::load()?;
            direct_relations(
                &context,
                &reference,
                Some(TraversalDirection::Incoming),
                all,
                cli.json,
            )
        }
        Command::Outgoing { reference, all } => {
            let context = Context::load()?;
            direct_relations(
                &context,
                &reference,
                Some(TraversalDirection::Outgoing),
                all,
                cli.json,
            )
        }
        Command::Traverse {
            reference,
            direction,
            depth,
            all,
        } => {
            let context = Context::load()?;
            traverse(&context, &reference, direction.into(), depth, all, cli.json)
        }
        Command::Context {
            reference,
            depth,
            all,
        } => {
            let context = Context::load()?;
            let _ = context.external_records(true)?;
            expanded_context(&context, &reference, depth, all, cli.json)
        }
        Command::Path {
            source,
            target,
            all,
        } => {
            let context = Context::load()?;
            let source = resolve_graph_reference(&context.graph, &source)?;
            let target = resolve_graph_reference(&context.graph, &target)?;
            let path = GraphTraversal::new(&context.graph)
                .shortest_path(&source, &target, (!all).then_some(RelationOrigin::Explicit))
                .ok_or_else(|| CliError::message("no graph path found"))?;
            let names: Vec<_> = path
                .iter()
                .map(|node| node_name(&context.graph, node))
                .collect();
            if cli.json {
                print_json(json!({ "path": names }))
            } else {
                println!("{}", names.join(" -> "));
                Ok(())
            }
        }
        Command::Normalize { dry_run } => mutate(MutationRequest::Normalize, dry_run, cli.json),
        Command::Validate { changes } => validate(changes.as_deref(), cli.json),
        Command::Review { reference } => review(&reference, cli.json),
        Command::Query { name, arguments } => query(&name, &arguments, cli.json),
        Command::Instructions { action } => instructions(action, cli.json),
        Command::Frontmatter { action } => frontmatter(action, cli.json),
        Command::Custom(arguments) => custom_command(&arguments, cli.json),
    }
}

fn initialize(
    requested_name: Option<&str>,
    requested_documents: Option<&Path>,
    requested_targets: &[PathBuf],
    dry_run: bool,
    json_output: bool,
) -> Result<(), CliError> {
    let root = git_worktree_root()?;
    let repository = Repository::from_root(&root).map_err(CliError::boxed)?;
    let project_path = repository.project_file();
    let config_exists = project_path.is_file();

    let requested_name = match requested_name {
        Some(name) if name.trim().is_empty() => {
            return Err(CliError::message("--name cannot be empty"));
        }
        Some(name) => Some(name.trim().to_owned()),
        None => None,
    };

    let requested_documents = requested_documents
        .map(|path| normalized_relative_path(path, "document root"))
        .transpose()?;
    let requested_targets: Vec<_> = requested_targets
        .iter()
        .map(|path| normalized_relative_path(path, "instruction target"))
        .collect::<Result<_, _>>()?;
    let mut unique_targets = HashSet::new();
    if requested_targets
        .iter()
        .any(|target| !unique_targets.insert(target.clone()))
    {
        return Err(CliError::message(
            "--instruction-target cannot contain duplicates",
        ));
    }

    let mut project_change = None;
    let config = if config_exists {
        let config = RepositoryConfig::load(&repository).map_err(|error| {
            CliError::message(format!(
                "cannot adopt existing .docgraph/project.toml: {error}"
            ))
        })?;
        if let Some(name) = &requested_name
            && name != &config.project.name
        {
            return Err(CliError::message(format!(
                "existing project name is {:?}, not requested {:?}",
                config.project.name, name
            )));
        }
        if let Some(documents) = &requested_documents
            && documents != &config.project.documents.root
        {
            return Err(CliError::message(format!(
                "existing document root is {}, not requested {}",
                config.project.documents.root.display(),
                documents.display()
            )));
        }
        if !requested_targets.is_empty()
            && requested_targets != config.project.agent_instructions.targets
        {
            return Err(CliError::message(
                "existing instruction targets differ from --instruction-target values",
            ));
        }
        config
    } else {
        refuse_ambiguous_config_directory(repository.config_dir())?;
        let name = requested_name.unwrap_or_else(|| {
            root.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "docgraph-project".to_owned())
        });
        let documents = requested_documents.unwrap_or_else(|| PathBuf::from("docs"));
        let instruction_targets = if requested_targets.is_empty() {
            AgentInstructionsConfig::default().targets
        } else {
            requested_targets
        };
        let source = minimal_project_source(&name, &documents, &instruction_targets)?;
        project_change = Some(docgraph_core::FileChange {
            path: PathBuf::from(".docgraph/project.toml"),
            original: None,
            intended: Some(source),
            original_hash: None,
        });
        RepositoryConfig {
            project: ProjectConfig {
                name,
                documents: DocumentsConfig {
                    root: documents,
                    include: vec!["**/*.md".to_owned()],
                    exclude: Vec::new(),
                },
                frontmatter: FrontmatterConfig::default(),
                agent_instructions: AgentInstructionsConfig {
                    targets: instruction_targets,
                },
                validation: ValidationConfig::default(),
                references: Vec::new(),
                embeddings: None,
                external_entities: docgraph_core::ExternalEntitiesConfig::default(),
            },
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
            workflows: BTreeMap::new(),
            queries: BTreeMap::new(),
            commands: BTreeMap::new(),
            logic: None,
        }
    };

    let documents_path = repository.root().join(&config.project.documents.root);
    if documents_path.exists() && !documents_path.is_dir() {
        return Err(CliError::message(format!(
            "configured document root is not a directory: {}",
            documents_path.display()
        )));
    }
    let create_documents = !documents_path.exists();

    let skill = PortableSkillService::new(&repository).map_err(CliError::boxed)?;
    let skill_changes = skill.sync(true).map_err(CliError::boxed)?;
    let instructions = InstructionService::new(&repository, &config).map_err(CliError::boxed)?;
    let instruction_changes = instructions.sync(true).map_err(CliError::boxed)?;

    let mut changes = Vec::new();
    if let Some(change) = &project_change {
        changes.push(change.clone());
    }
    changes.extend(
        skill_changes
            .iter()
            .map(|change| docgraph_core::FileChange {
                path: change.path.clone(),
                original: change.original.clone(),
                intended: Some(change.intended.clone()),
                original_hash: None,
            }),
    );
    changes.extend(
        instruction_changes
            .iter()
            .map(|change| docgraph_core::FileChange {
                path: change.path.clone(),
                original: change.original.clone(),
                intended: Some(change.intended.clone()),
                original_hash: None,
            }),
    );

    if !dry_run {
        if let Some(change) = &project_change {
            refuse_ambiguous_config_directory(repository.config_dir())?;
            fs::create_dir_all(repository.config_dir()).map_err(CliError::boxed)?;
            let mut project = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&project_path)
                .map_err(|error| {
                    CliError::message(format!(
                        "refusing to overwrite {}: {error}",
                        project_path.display()
                    ))
                })?;
            project
                .write_all(
                    change
                        .intended
                        .as_deref()
                        .expect("new project configuration has intended content")
                        .as_bytes(),
                )
                .map_err(CliError::boxed)?;
            project.sync_all().map_err(CliError::boxed)?;
        }
        if create_documents {
            fs::create_dir_all(&documents_path).map_err(CliError::boxed)?;
        }
        skill.sync(false).map_err(CliError::boxed)?;
        instructions.sync(false).map_err(CliError::boxed)?;

        RepositoryConfig::load(&repository).map_err(CliError::boxed)?;
        if skill.check().map_err(CliError::boxed)? != PortableSkillStatus::Current
            || instructions
                .check()
                .map_err(CliError::boxed)?
                .iter()
                .any(|(_, status)| *status != InstructionStatus::Current)
        {
            return Err(CliError::message(
                "initialization completed with non-current guidance; run docgraph instructions check",
            ));
        }
    }

    print_initialization(
        repository.root(),
        config_exists,
        &changes,
        create_documents.then_some(config.project.documents.root.as_path()),
        dry_run,
        json_output,
    )
}

fn git_worktree_root() -> Result<PathBuf, CliError> {
    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|error| {
            CliError::message(format!("cannot run git to locate repository: {error}"))
        })?;
    if !output.status.success() {
        return Err(CliError::message(
            "docgraph init must run inside a Git worktree",
        ));
    }
    let root = String::from_utf8(output.stdout)
        .map_err(|_| CliError::message("Git worktree path is not valid UTF-8"))?;
    fs::canonicalize(root.trim()).map_err(CliError::boxed)
}

fn normalized_relative_path(path: &Path, label: &str) -> Result<PathBuf, CliError> {
    if path.is_absolute() {
        return Err(CliError::message(format!(
            "{label} must be repository-relative: {}",
            path.display()
        )));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CliError::message(format!(
                    "{label} cannot escape the repository: {}",
                    path.display()
                )));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(CliError::message(format!("{label} cannot be empty")));
    }
    Ok(normalized)
}

fn refuse_ambiguous_config_directory(path: &Path) -> Result<(), CliError> {
    if !path.exists() {
        return Ok(());
    }
    let entries: Vec<_> = fs::read_dir(path)
        .map_err(CliError::boxed)?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    if entries.is_empty() {
        Ok(())
    } else {
        Err(CliError::message(format!(
            "{} exists without project.toml and contains {}; move or reconcile that state before init",
            path.display(),
            entries.join(", ")
        )))
    }
}

fn minimal_project_source(
    name: &str,
    documents: &Path,
    instruction_targets: &[PathBuf],
) -> Result<String, CliError> {
    let documents = documents
        .to_str()
        .ok_or_else(|| CliError::message("document root is not valid UTF-8"))?;
    let mut targets = toml_edit::Array::new();
    for target in instruction_targets {
        targets.push(
            target
                .to_str()
                .ok_or_else(|| CliError::message("instruction target is not valid UTF-8"))?,
        );
    }
    Ok(format!(
        "schema_version = {SCHEMA_VERSION}\n\n[project]\nname = {}\n\n[documents]\nroot = {}\n\n[agent_instructions]\ntargets = {}\n",
        Value::from(name),
        Value::from(documents),
        targets
    ))
}

fn print_initialization(
    root: &Path,
    adopted: bool,
    changes: &[docgraph_core::FileChange],
    created_directory: Option<&Path>,
    dry_run: bool,
    json_output: bool,
) -> Result<(), CliError> {
    if json_output {
        return print_json(json!({
            "root": json_path(root),
            "adopted": adopted,
            "dry_run": dry_run,
            "directories": created_directory.iter().map(|path| json_path(path)).collect::<Vec<_>>(),
            "changes": changes.iter().map(|change| json!({
                "path": json_path(&change.path),
                "original": change.original,
                "intended": change.intended,
            })).collect::<Vec<_>>(),
        }));
    }
    if changes.is_empty() && created_directory.is_none() {
        println!("already initialized {}", root.display());
        return Ok(());
    }
    if dry_run {
        for change in changes {
            println!("{}", render_patch(change));
        }
        if let Some(path) = created_directory {
            println!("would create directory {}", path.display());
        }
    } else {
        for change in changes {
            let action = if change.original.is_some() {
                "updated"
            } else {
                "created"
            };
            println!("{action} {}", change.path.display());
        }
        if let Some(path) = created_directory {
            println!("created directory {}", path.display());
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdoptionManifest {
    document: Vec<ManifestDocument>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestDocument {
    path: PathBuf,
    id: String,
    #[serde(rename = "type")]
    entity_type: String,
    #[serde(default, rename = "property")]
    properties: Vec<String>,
}

fn read_adoption_manifest(
    path: &PathBuf,
    config: &RepositoryConfig,
) -> Result<Vec<Adoption>, CliError> {
    let source = std::fs::read_to_string(path).map_err(CliError::boxed)?;
    let manifest: AdoptionManifest = toml_edit::de::from_str(&source).map_err(CliError::boxed)?;
    manifest
        .document
        .into_iter()
        .map(|document| {
            let entity = config.entities.get(&document.entity_type).ok_or_else(|| {
                CliError::message(format!("unknown entity type {:?}", document.entity_type))
            })?;
            Ok(Adoption {
                path: document.path,
                id: document.id,
                entity_type: document.entity_type,
                properties: parse_properties(&document.properties, &entity.property, "entity")?,
            })
        })
        .collect()
}

fn print_root_help() -> Result<(), CliError> {
    let mut command = Cli::command();
    if let Ok(repository) = Repository::discover(".")
        && let Ok(config) = RepositoryConfig::load(&repository)
        && !config.commands.is_empty()
    {
        let mut help = String::from("Repository commands:\n");
        for (name, configured) in &config.commands {
            help.push_str(&format!(
                "  {:20} {}\n",
                name.replace('.', " "),
                configured.description
            ));
        }
        command = command.before_help(help.trim_end().to_owned());
    }
    command.print_long_help().map_err(CliError::boxed)?;
    println!();
    Ok(())
}

fn custom_command(arguments: &[String], json_output: bool) -> Result<(), CliError> {
    let repository = Repository::discover(".").map_err(CliError::boxed)?;
    let config = RepositoryConfig::load(&repository).map_err(CliError::boxed)?;
    if arguments
        .last()
        .is_some_and(|argument| matches!(argument.as_str(), "--help" | "-h"))
    {
        let path = arguments[..arguments.len() - 1].join(".");
        if let Some(command) = config.commands.get(&path) {
            return print_custom_help(&path, command, &config);
        }
        return print_custom_group_help(&path, &config);
    }
    let (name, consumed) = resolve_custom_command(&config, arguments)?;
    let command = &config.commands[&name];
    let remaining = &arguments[consumed..];
    if remaining
        .iter()
        .any(|argument| matches!(argument.as_str(), "--help" | "-h"))
    {
        return print_custom_help(&name, command, &config);
    }
    match &command.operation {
        CommandOperation::Query {
            query: name,
            entity_type,
        } => {
            let query = config.queries.get(name).ok_or_else(|| {
                CliError::message(format!(
                    "repository command references unknown query {name:?}"
                ))
            })?;
            let inputs: Vec<_> = query
                .arguments
                .iter()
                .filter(|argument| argument.mode == docgraph_core::ArgumentMode::Input)
                .collect();
            let mut raw = BTreeMap::new();
            let mut index = 0;
            if entity_type.is_some() && !inputs.is_empty() {
                let value = remaining
                    .first()
                    .filter(|value| !value.starts_with('-'))
                    .ok_or_else(|| CliError::message(format!("missing {}", inputs[0].name)))?;
                raw.insert(inputs[0].name.clone(), value.clone());
                index = 1;
            }
            while index < remaining.len() {
                let option = remaining[index].strip_prefix("--").ok_or_else(|| {
                    CliError::message(format!("expected --name, found {:?}", remaining[index]))
                })?;
                let value = remaining
                    .get(index + 1)
                    .ok_or_else(|| CliError::message(format!("missing value for --{option}")))?;
                if raw.insert(option.to_owned(), value.clone()).is_some() {
                    return Err(CliError::message(format!("duplicate --{option}")));
                }
                index += 2;
            }
            execute_query(name, raw, json_output)
        }
        CommandOperation::Transition { .. } => {
            let (entity, dry_run) = parse_entity_mutation_arguments(remaining)?;
            let CommandOperation::Transition { target_state, .. } = &command.operation else {
                unreachable!()
            };
            mutate(
                MutationRequest::Transition {
                    entity,
                    target_state: target_state.clone(),
                },
                dry_run,
                json_output,
            )
        }
        CommandOperation::AddRelation { relation, .. } => {
            let dry_run = remaining.iter().any(|argument| argument == "--dry-run");
            let positional: Vec<_> = remaining
                .iter()
                .filter(|argument| argument.as_str() != "--dry-run")
                .collect();
            if positional.len() != 2 {
                return Err(CliError::message(
                    "relation command requires SOURCE TARGET [--dry-run]",
                ));
            }
            mutate(
                MutationRequest::AddRelation {
                    source: positional[0].clone(),
                    predicate: relation.clone(),
                    target: positional[1].clone(),
                    properties: BTreeMap::new(),
                },
                dry_run,
                json_output,
            )
        }
    }
}

fn print_custom_group_help(path: &str, config: &RepositoryConfig) -> Result<(), CliError> {
    let prefix = if path.is_empty() {
        String::new()
    } else {
        format!("{path}.")
    };
    let matches: Vec<_> = config
        .commands
        .iter()
        .filter(|(name, _)| name.starts_with(&prefix))
        .collect();
    if matches.is_empty() {
        return Err(CliError::message(format!(
            "unknown repository command group {:?}",
            path.replace('.', " ")
        )));
    }
    println!("Usage: docgraph {} <COMMAND>\n", path.replace('.', " "));
    println!("Commands:");
    for (name, command) in matches {
        let suffix = &name[prefix.len()..];
        println!("  {:20} {}", suffix.replace('.', " "), command.description);
    }
    Ok(())
}

fn resolve_custom_command(
    config: &RepositoryConfig,
    arguments: &[String],
) -> Result<(String, usize), CliError> {
    for consumed in (1..=arguments.len()).rev() {
        let name = arguments[..consumed].join(".");
        if config.commands.contains_key(&name) {
            return Ok((name, consumed));
        }
    }
    Err(CliError::message(format!(
        "unknown repository command {:?}",
        arguments.first().map(String::as_str).unwrap_or_default()
    )))
}

fn print_custom_help(
    name: &str,
    command: &CommandConfig,
    config: &RepositoryConfig,
) -> Result<(), CliError> {
    println!("{}", command.description);
    match &command.operation {
        CommandOperation::Query { query, entity_type } => {
            let query = config.queries.get(query).ok_or_else(|| {
                CliError::message(format!(
                    "repository command references unknown query {query:?}"
                ))
            })?;
            print!("\nUsage: docgraph {}", name.replace('.', " "));
            for (index, argument) in query
                .arguments
                .iter()
                .filter(|argument| argument.mode == docgraph_core::ArgumentMode::Input)
                .enumerate()
            {
                if index == 0 && entity_type.is_some() {
                    print!(" <{}>", argument.name);
                } else if argument.default.is_some() {
                    print!(" [--{} <VALUE>]", argument.name);
                } else {
                    print!(" --{} <VALUE>", argument.name);
                }
            }
            println!();
        }
        CommandOperation::Transition { .. } => println!(
            "\nUsage: docgraph {} <ENTITY> [--dry-run]",
            name.replace('.', " ")
        ),
        CommandOperation::AddRelation { .. } => println!(
            "\nUsage: docgraph {} <SOURCE> <TARGET> [--dry-run]",
            name.replace('.', " ")
        ),
    }
    Ok(())
}

fn parse_entity_mutation_arguments(arguments: &[String]) -> Result<(String, bool), CliError> {
    let dry_run = arguments.iter().any(|argument| argument == "--dry-run");
    let positional: Vec<_> = arguments
        .iter()
        .filter(|argument| argument.as_str() != "--dry-run")
        .collect();
    if positional.len() != 1 {
        return Err(CliError::message(
            "transition command requires ENTITY [--dry-run]",
        ));
    }
    Ok((positional[0].clone(), dry_run))
}

fn describe(
    config: &RepositoryConfig,
    all: bool,
    kind: Option<DescribeKind>,
    name: Option<&str>,
    json_output: bool,
) -> Result<(), CliError> {
    if all && (kind.is_some() || name.is_some()) {
        return Err(CliError::message(
            "describe --all does not accept a kind or name",
        ));
    }
    if all {
        return print_json(complete_description_json(config));
    }
    let value = match (kind, name) {
        (None, None) => json!({
            "project": config.project.name,
            "documents_root": json_path(&config.project.documents.root),
            "entity_types": config.entities.keys().collect::<Vec<_>>(),
            "relations": config.relations.keys().collect::<Vec<_>>(),
            "workflows": config.workflows.keys().collect::<Vec<_>>(),
            "queries": config.queries.keys().collect::<Vec<_>>(),
            "commands": config.commands.keys().collect::<Vec<_>>(),
            "reference_providers": config.project.references.iter().map(|reference| json!({
                "provider": reference.provider,
                "host": reference.host,
                "repository": reference.repository,
                "remote": reference.remote,
            })).collect::<Vec<_>>(),
            "embeddings": config.project.embeddings.as_ref().map(|embedding| json!({
                "provider": embedding.provider,
                "model": embedding.model,
                "dimensions": embedding.dimensions,
                "batch_size": embedding.batch_size,
                "timeout_seconds": embedding.timeout_seconds,
                "fallback": match embedding.fallback {
                    docgraph_core::EmbeddingFallback::FullText => "full_text",
                    docgraph_core::EmbeddingFallback::Error => "error",
                },
            })),
            "external_entities": {
                "cache_ttl_seconds": config.project.external_entities.cache_ttl_seconds,
                "sources": config.project.external_entities.source.iter().map(|source| json!({
                    "provider": source.provider,
                    "host": source.host,
                    "api_url": source.api_url,
                    "token_env": source.token_env,
                    "token_command": source.token_command,
                    "timeout_seconds": source.timeout_seconds,
                    "capabilities": {
                        "read": source.provider == "github",
                        "search": source.provider == "github",
                        "mutate": false,
                    },
                })).collect::<Vec<_>>(),
            },
        }),
        (Some(DescribeKind::Type), Some(name)) => {
            let item = config
                .entities
                .get(name)
                .ok_or_else(|| CliError::message(format!("unknown entity type {name:?}")))?;
            json!({ "name": name, "description": item.description, "workflow": item.workflow, "properties": property_schema_json(&item.property) })
        }
        (Some(DescribeKind::Relation), Some(name)) => {
            let item = config
                .relations
                .get(name)
                .ok_or_else(|| CliError::message(format!("unknown relation {name:?}")))?;
            json!({ "name": name, "description": item.description, "source": item.source, "target": item.target, "inverse": item.inverse, "acyclic": item.acyclic, "properties": property_schema_json(&item.property) })
        }
        (Some(DescribeKind::Workflow), Some(name)) => {
            let item = config
                .workflows
                .get(name)
                .ok_or_else(|| CliError::message(format!("unknown workflow {name:?}")))?;
            let states: BTreeMap<_, _> = item.states.iter().map(|(name, state)| (name, json!({ "description": state.description, "transitions": state.transitions }))).collect();
            json!({ "name": name, "initial": item.initial, "states": states })
        }
        (Some(DescribeKind::Query), Some(name)) => {
            let item = config
                .queries
                .get(name)
                .ok_or_else(|| CliError::message(format!("unknown query {name:?}")))?;
            let arguments: Vec<_> = item.arguments.iter().map(|argument| json!({ "name": argument.name, "mode": format!("{:?}", argument.mode).to_lowercase(), "type": format!("{:?}", argument.value_type).to_lowercase(), "default": argument.default })).collect();
            json!({ "name": name, "description": item.description, "predicate": item.predicate, "arguments": arguments })
        }
        (Some(DescribeKind::Command), Some(name)) => {
            let item = config
                .commands
                .get(name)
                .ok_or_else(|| CliError::message(format!("unknown command {name:?}")))?;
            let operation = command_operation_json(&item.operation);
            json!({ "name": name, "description": item.description, "operation": operation })
        }
        (Some(_), None) => return Err(CliError::message("describe kind requires a name")),
        (None, Some(_)) => return Err(CliError::message("describe name requires a kind")),
    };
    if json_output {
        print_json(value)
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(CliError::boxed)?
        );
        Ok(())
    }
}

fn complete_description_json(config: &RepositoryConfig) -> JsonValue {
    let entity_types: BTreeMap<_, _> = config
        .entities
        .iter()
        .map(|(name, item)| {
            (
                name,
                json!({
                    "description": item.description,
                    "workflow": item.workflow,
                    "properties": property_schema_json(&item.property),
                }),
            )
        })
        .collect();
    let relations: BTreeMap<_, _> = config
        .relations
        .iter()
        .map(|(name, item)| {
            (
                name,
                json!({
                    "description": item.description,
                    "source": item.source,
                    "target": item.target,
                    "inverse": item.inverse,
                    "acyclic": item.acyclic,
                    "properties": property_schema_json(&item.property),
                }),
            )
        })
        .collect();
    let workflows: BTreeMap<_, _> = config
        .workflows
        .iter()
        .map(|(name, item)| {
            let states: BTreeMap<_, _> = item
                .states
                .iter()
                .map(|(name, state)| {
                    (
                        name,
                        json!({
                            "description": state.description,
                            "transitions": state.transitions,
                        }),
                    )
                })
                .collect();
            (name, json!({ "initial": item.initial, "states": states }))
        })
        .collect();
    let queries: BTreeMap<_, _> = config
        .queries
        .iter()
        .map(|(name, item)| {
            let arguments: Vec<_> = item
                .arguments
                .iter()
                .map(|argument| {
                    json!({
                        "name": argument.name,
                        "mode": format!("{:?}", argument.mode).to_lowercase(),
                        "type": format!("{:?}", argument.value_type).to_lowercase(),
                        "default": argument.default,
                    })
                })
                .collect();
            (
                name,
                json!({
                    "description": item.description,
                    "predicate": item.predicate,
                    "arguments": arguments,
                }),
            )
        })
        .collect();
    let commands: BTreeMap<_, _> = config
        .commands
        .iter()
        .map(|(name, item)| {
            (
                name,
                json!({
                    "description": item.description,
                    "operation": command_operation_json(&item.operation),
                }),
            )
        })
        .collect();

    json!({
        "schema_version": SCHEMA_VERSION,
        "project": {
            "name": config.project.name,
            "documents": {
                "root": json_path(&config.project.documents.root),
                "include": config.project.documents.include,
                "exclude": config.project.documents.exclude,
            },
            "frontmatter": {
                "id": config.project.frontmatter.id,
                "entity_type": config.project.frontmatter.entity_type,
                "state": config.project.frontmatter.state,
                "relations": config.project.frontmatter.relations,
                "properties": config.project.frontmatter.properties,
            },
            "agent_instructions": {
                "targets": config.project.agent_instructions.targets.iter().map(|target| json_path(target)).collect::<Vec<_>>(),
            },
            "validation": {
                "broken_internal_links": severity_name(config.project.validation.broken_internal_links),
            },
            "references": config.project.references.iter().map(|reference| json!({
                "provider": reference.provider,
                "host": reference.host,
                "repository": reference.repository,
                "remote": reference.remote,
            })).collect::<Vec<_>>(),
            "embeddings": config.project.embeddings.as_ref().map(|embedding| json!({
                "provider": embedding.provider,
                "model": embedding.model,
                "dimensions": embedding.dimensions,
                "command": embedding.command,
                "batch_size": embedding.batch_size,
                "timeout_seconds": embedding.timeout_seconds,
                "fallback": match embedding.fallback {
                    docgraph_core::EmbeddingFallback::FullText => "full_text",
                    docgraph_core::EmbeddingFallback::Error => "error",
                },
            })),
            "external_entities": {
                "cache_ttl_seconds": config.project.external_entities.cache_ttl_seconds,
                "sources": config.project.external_entities.source,
            },
            "logic_configured": config.logic.is_some(),
        },
        "entity_types": entity_types,
        "relations": relations,
        "workflows": workflows,
        "queries": queries,
        "commands": commands,
    })
}

fn command_operation_json(operation: &CommandOperation) -> JsonValue {
    match operation {
        CommandOperation::Query { query, entity_type } => json!({
            "type": "query", "query": query, "entity_type": entity_type,
        }),
        CommandOperation::Transition {
            entity_type,
            target_state,
        } => json!({
            "type": "transition", "entity_type": entity_type, "target_state": target_state,
        }),
        CommandOperation::AddRelation {
            entity_type,
            relation,
        } => json!({
            "type": "add_relation", "entity_type": entity_type, "relation": relation,
        }),
    }
}

fn get(context: &Context, reference: &str, json_output: bool) -> Result<(), CliError> {
    let node = resolve_graph_reference(&context.graph, reference)?;
    let value = if let GraphNode::ExternalUri(identity) = &node {
        external_context_value(&context.resolve_external(identity, true)?)
    } else {
        node_context_value(context, &node)
    };
    if json_output {
        print_json(value)
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(CliError::boxed)?
        );
        Ok(())
    }
}

fn node_context_value(context: &Context, node: &GraphNode) -> JsonValue {
    if let GraphNode::Entity(id) = node {
        let entity = context
            .graph
            .entities
            .iter()
            .find(|entity| entity.id == *id)
            .expect("resolved entities originate in the graph");
        let properties: BTreeMap<_, _> = entity
            .properties
            .iter()
            .map(|(key, value)| (key, toml_value_json(value)))
            .collect();
        json!({
            "kind": "entity",
            "id": entity.id,
            "type": entity.entity_type,
            "state": entity.state,
            "document": json_path(&context.graph.documents[entity.document].path),
            "properties": properties,
            "relations": relation_context(&context.graph, node),
        })
    } else if let GraphNode::Section(index) = *node {
        let section = &context.graph.sections[index];
        let document = &context.graph.documents[section.document];
        let file = context
            .corpus
            .files
            .iter()
            .find(|file| file.path == document.path)
            .expect("graph documents originate in the corpus");
        let line_count = section.location.span.line_count();
        let end_line = section
            .location
            .span
            .start_line
            .saturating_add(line_count.saturating_sub(1));
        json!({
            "kind": "section",
            "id": node_name(&context.graph, node),
            "heading": section.heading,
            "level": section.level,
            "document": json_path(&document.path),
            "parent": section.parent.map(|parent| node_name(&context.graph, &GraphNode::Section(parent))),
            "span": {
                "start_line": section.location.span.start_line,
                "end_line": end_line,
                "line_count": line_count,
            },
            "content": &file.content[section.location.span.bytes.clone()],
            "relations": relation_context(&context.graph, node),
        })
    } else if let GraphNode::Document(index) = *node {
        json!({
            "kind": "document",
            "id": node_name(&context.graph, node),
            "document": json_path(&context.graph.documents[index].path),
            "relations": relation_context(&context.graph, node),
        })
    } else if let GraphNode::ExternalUri(identity) = node {
        context.resolve_external(identity, false).map_or_else(
            |_| json!({ "kind": "external", "id": identity, "fallback": "identity_only" }),
            |view| external_context_value(&view),
        )
    } else if matches!(node, GraphNode::Unresolved(_)) {
        json!({ "kind": "unresolved", "id": node_name(&context.graph, node) })
    } else {
        unreachable!("all graph node kinds are covered")
    }
}

fn external_context_value(view: &ExternalEntityView) -> JsonValue {
    json!({
        "kind": "external",
        "id": view.identity,
        "canonical": false,
        "provenance": "derived_external",
        "trusted": false,
        "fallback": view.fallback,
        "record": view.record,
        "error": view.error,
    })
}

fn external_hit_metadata(context: &Context, identity: &str) -> Option<JsonValue> {
    ExternalIdentity::parse(identity).map(|_| {
        context.resolve_external(identity, false).map_or_else(
            |_| json!({ "provenance": "derived_external", "fallback": "identity_only" }),
            |view| {
                json!({
                    "provenance": "derived_external",
                    "trusted": false,
                    "fallback": view.fallback,
                    "provider": view.record.as_ref().map(|record| &record.record.provider),
                    "freshness": view.record.as_ref().map(|record| record.freshness),
                    "fetched_at": view.record.as_ref().map(|record| record.fetched_at),
                    "url": view.record.as_ref().map(|record| &record.record.url),
                    "error": view.error,
                })
            },
        )
    })
}

fn external_hit_label(context: &Context, identity: &str) -> String {
    external_hit_metadata(context, identity).map_or_else(String::new, |metadata| {
        let fallback = metadata["fallback"].as_str().unwrap_or("identity_only");
        let freshness = metadata["freshness"].as_str().unwrap_or("unknown");
        format!("derived_external:{fallback}:{freshness}")
    })
}

fn resolve_graph_reference(graph: &GraphIndex, reference: &str) -> Result<GraphNode, CliError> {
    if graph.entities.iter().any(|entity| entity.id == reference) {
        return Ok(GraphNode::Entity(reference.to_owned()));
    }
    if graph.relations.iter().any(|relation| {
        matches!(&relation.source, GraphNode::ExternalUri(identity) if identity == reference)
            || matches!(&relation.target, GraphNode::ExternalUri(identity) if identity == reference)
    }) || ExternalIdentity::parse(reference).is_some()
    {
        return Ok(GraphNode::ExternalUri(reference.to_owned()));
    }
    graph
        .sections
        .iter()
        .enumerate()
        .find_map(|(index, _)| {
            (node_name(graph, &GraphNode::Section(index)) == reference)
                .then_some(GraphNode::Section(index))
        })
        .ok_or_else(|| {
            CliError::message(format!(
                "entity or stable section {reference:?} does not exist"
            ))
        })
}

#[derive(Subcommand)]
enum PropertyAction {
    Set {
        entity: String,
        property: String,
        value: String,
        /// Allow this mutation only when it strictly reduces existing validation errors.
        #[arg(long)]
        repair: bool,
        #[arg(long)]
        dry_run: bool,
    },
    Unset {
        entity: String,
        property: String,
        /// Allow this mutation only when it strictly reduces existing validation errors.
        #[arg(long)]
        repair: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

fn relation_context(graph: &GraphIndex, node: &GraphNode) -> Vec<JsonValue> {
    GraphTraversal::new(graph)
        .neighbors(node, None)
        .into_iter()
        .map(|neighbor| {
            json!({
                "direction": if neighbor.outgoing { "outgoing" } else { "incoming" },
                "predicate": neighbor.relation.predicate,
                "target": node_name(graph, neighbor.node),
                "origin": origin_name(neighbor.relation.origin),
            })
        })
        .collect()
}

fn direct_relations(
    context: &Context,
    reference: &str,
    direction: Option<TraversalDirection>,
    all: bool,
    json_output: bool,
) -> Result<(), CliError> {
    let node = resolve_graph_reference(&context.graph, reference)?;
    let neighbors: Vec<_> = GraphTraversal::new(&context.graph)
        .neighbors(&node, (!all).then_some(RelationOrigin::Explicit))
        .into_iter()
        .filter(|neighbor| match direction {
            Some(TraversalDirection::Incoming) => !neighbor.outgoing,
            Some(TraversalDirection::Outgoing) => neighbor.outgoing,
            Some(TraversalDirection::Both) | None => true,
        })
        .collect();
    if json_output {
        let rows: Vec<_> = neighbors
            .iter()
            .map(|neighbor| {
                json!({
                    "node": node_name(&context.graph, neighbor.node),
                    "predicate": neighbor.relation.predicate,
                    "direction": if neighbor.outgoing { "outgoing" } else { "incoming" },
                    "origin": origin_name(neighbor.relation.origin),
                })
            })
            .collect();
        print_json(json!({ "reference": node_name(&context.graph, &node), "rows": rows }))
    } else {
        for neighbor in neighbors {
            println!(
                "{}\t{}\t{}\t{}",
                if neighbor.outgoing { "->" } else { "<-" },
                neighbor.relation.predicate,
                node_name(&context.graph, neighbor.node),
                origin_name(neighbor.relation.origin)
            );
        }
        Ok(())
    }
}

fn traverse(
    context: &Context,
    reference: &str,
    direction: TraversalDirection,
    depth: usize,
    all: bool,
    json_output: bool,
) -> Result<(), CliError> {
    let node = resolve_graph_reference(&context.graph, reference)?;
    let steps = GraphTraversal::new(&context.graph).traverse(
        &node,
        direction,
        depth,
        (!all).then_some(RelationOrigin::Explicit),
    );
    if json_output {
        let rows: Vec<_> = steps
            .iter()
            .map(|step| {
                json!({
                    "node": node_name(&context.graph, &step.node),
                    "depth": step.depth,
                    "from": node_name(&context.graph, &step.from),
                    "predicate": step.relation.predicate,
                    "direction": if step.outgoing { "outgoing" } else { "incoming" },
                    "origin": origin_name(step.relation.origin),
                })
            })
            .collect();
        print_json(json!({
            "reference": node_name(&context.graph, &node),
            "direction": traversal_direction_name(direction),
            "depth": depth,
            "rows": rows,
        }))
    } else {
        for step in steps {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                step.depth,
                node_name(&context.graph, &step.from),
                if step.outgoing { "->" } else { "<-" },
                step.relation.predicate,
                node_name(&context.graph, &step.node),
                origin_name(step.relation.origin)
            );
        }
        Ok(())
    }
}

fn expanded_context(
    context: &Context,
    reference: &str,
    depth: usize,
    all: bool,
    json_output: bool,
) -> Result<(), CliError> {
    let root = resolve_graph_reference(&context.graph, reference)?;
    let steps = GraphTraversal::new(&context.graph).traverse(
        &root,
        TraversalDirection::Both,
        depth,
        (!all).then_some(RelationOrigin::Explicit),
    );
    let mut selected = HashSet::from([root.clone()]);
    selected.extend(steps.iter().map(|step| step.node.clone()));
    let mut nodes = vec![json!({
        "depth": 0,
        "node": node_context_value(context, &root),
    })];
    nodes.extend(steps.iter().map(|step| {
        json!({
            "depth": step.depth,
            "node": node_context_value(context, &step.node),
        })
    }));
    let relations: Vec<_> = context
        .graph
        .relations
        .iter()
        .filter(|relation| {
            (all || relation.origin == RelationOrigin::Explicit)
                && selected.contains(&relation.source)
                && selected.contains(&relation.target)
        })
        .map(|relation| {
            let properties: BTreeMap<_, _> = relation
                .properties
                .iter()
                .map(|(key, value)| (key, toml_value_json(value)))
                .collect();
            json!({
                "source": node_name(&context.graph, &relation.source),
                "predicate": relation.predicate,
                "target": node_name(&context.graph, &relation.target),
                "origin": origin_name(relation.origin),
                "properties": properties,
            })
        })
        .collect();
    let value = json!({
        "reference": node_name(&context.graph, &root),
        "depth": depth,
        "nodes": nodes,
        "relations": relations,
    });
    if json_output {
        print_json(value)
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(CliError::boxed)?
        );
        Ok(())
    }
}

fn traversal_direction_name(direction: TraversalDirection) -> &'static str {
    match direction {
        TraversalDirection::Incoming => "incoming",
        TraversalDirection::Outgoing => "outgoing",
        TraversalDirection::Both => "both",
    }
}

fn mutate(request: MutationRequest, dry_run: bool, json_output: bool) -> Result<(), CliError> {
    let service = MutationService::open(".").map_err(CliError::boxed)?;
    let plan = service.apply(&request, dry_run).map_err(CliError::boxed)?;
    print_plan(&plan, dry_run, json_output)
}

fn property(action: PropertyAction, json_output: bool) -> Result<(), CliError> {
    let (entity, property, raw_value, repair, dry_run) = match action {
        PropertyAction::Set {
            entity,
            property,
            value,
            repair,
            dry_run,
        } => (entity, property, Some(value), repair, dry_run),
        PropertyAction::Unset {
            entity,
            property,
            repair,
            dry_run,
        } => (entity, property, None, repair, dry_run),
    };
    let context = Context::load()?;
    let node = context
        .graph
        .entities
        .iter()
        .find(|candidate| candidate.id == entity)
        .ok_or_else(|| CliError::message(format!("entity {entity:?} does not exist")))?;
    let property_config = context
        .config
        .entities
        .get(&node.entity_type)
        .and_then(|entity| entity.property.get(&property))
        .ok_or_else(|| {
            CliError::message(format!(
                "entity type {:?} has no declared property {property:?}",
                node.entity_type
            ))
        })?;
    let request = raw_value.map_or_else(
        || {
            Ok(MutationRequest::RemoveEntityProperty {
                entity: entity.clone(),
                property: property.clone(),
            })
        },
        |raw| {
            parse_toml_value(&raw, property_config).map(|value| {
                MutationRequest::SetEntityProperty {
                    entity: entity.clone(),
                    property: property.clone(),
                    value,
                }
            })
        },
    )?;
    if repair {
        let service = MutationService::open(".").map_err(CliError::boxed)?;
        let plan = service
            .apply_repair(&request, dry_run)
            .map_err(CliError::boxed)?;
        print_plan(&plan, dry_run, json_output)
    } else {
        mutate(request, dry_run, json_output)
    }
}

fn review(reference: &str, json_output: bool) -> Result<(), CliError> {
    let context = Context::load()?;
    let base = CanonicalCorpus::load_at_git_ref(&context.repository, &context.config, reference)
        .map_err(CliError::boxed)?;
    let report = SemanticChangeReviewer::review(&base, &context.corpus, &context.config);
    if json_output {
        return print_json(json!({
            "base": reference,
            "valid": report.is_valid(),
            "changes": report.changes,
            "diagnostics": report.diagnostics.iter().map(|diagnostic| json!({
                "code": diagnostic.code,
                "message": diagnostic.message,
                "path": json_path(&diagnostic.path),
            })).collect::<Vec<_>>(),
        }));
    }
    if report.changes.is_empty() {
        println!("no semantic changes from {reference}");
    } else {
        println!("semantic changes from {reference}:");
        for change in &report.changes {
            println!("{}", semantic_change_summary(change));
        }
    }
    for diagnostic in &report.diagnostics {
        println!(
            "! {}: {}: {}",
            diagnostic.path.display(),
            diagnostic.code,
            diagnostic.message
        );
    }
    Ok(())
}

fn semantic_change_summary(change: &SemanticChange) -> String {
    match change {
        SemanticChange::EntityAdded {
            entity,
            entity_type,
            path,
        } => format!("+ entity {entity} ({entity_type}) at {path}"),
        SemanticChange::EntityRemoved {
            entity,
            entity_type,
            path,
        } => format!("- entity {entity} ({entity_type}) at {path}"),
        SemanticChange::EntityMoved {
            entity,
            before,
            after,
        } => format!("~ entity {entity} moved {before} -> {after}"),
        SemanticChange::EntityTypeChanged {
            entity,
            before,
            after,
        } => format!("~ entity {entity} type {before} -> {after}"),
        SemanticChange::WorkflowStateChanged {
            entity,
            before,
            after,
        } => format!(
            "~ workflow {entity} {} -> {}",
            optional_value(before.as_deref()),
            optional_value(after.as_deref())
        ),
        SemanticChange::PropertyChanged {
            entity,
            property,
            before,
            after,
        } => format!(
            "~ property {entity}.{property} {} -> {}",
            optional_value(before.as_deref()),
            optional_value(after.as_deref())
        ),
        SemanticChange::SectionAdded { section, after } => {
            format!("+ section {section} {}", semantic_section_summary(after))
        }
        SemanticChange::SectionRemoved { section, before } => {
            format!("- section {section} {}", semantic_section_summary(before))
        }
        SemanticChange::SectionChanged {
            section,
            before,
            after,
        } => format!(
            "~ section {section} {} -> {}",
            semantic_section_summary(before),
            semantic_section_summary(after)
        ),
        SemanticChange::RelationAdded { relation } => format!(
            "+ relation {} --{}--> {} [{}]{}",
            relation.source,
            relation.predicate,
            relation.target,
            relation.origin,
            relation_properties(&relation.properties)
        ),
        SemanticChange::RelationRemoved { relation } => format!(
            "- relation {} --{}--> {} [{}]{}",
            relation.source,
            relation.predicate,
            relation.target,
            relation.origin,
            relation_properties(&relation.properties)
        ),
    }
}

fn semantic_section_summary(section: &SemanticSection) -> String {
    format!(
        "{:?} (document {}, level {}, parent {})",
        section.heading,
        section.document,
        section.level,
        section.parent.as_deref().unwrap_or("none")
    )
}

fn relation_properties(properties: &BTreeMap<String, String>) -> String {
    if properties.is_empty() {
        String::new()
    } else {
        format!(" {properties:?}")
    }
}

fn optional_value(value: Option<&str>) -> &str {
    value.unwrap_or("(unset)")
}

fn validate(changes: Option<&str>, json_output: bool) -> Result<(), CliError> {
    let context = Context::load()?;
    let report = Validator::validate_corpus(
        &context.repository,
        &context.config,
        &context.corpus,
        &context.graph,
    );
    let logic_error = QueryEngine::new(&context.config, &context.graph)
        .and_then(|engine| engine.validate())
        .err();
    let mut diagnostics: Vec<_> = report
        .diagnostics
        .iter()
        .map(|diagnostic| {
            json!({
                "severity": severity_name(diagnostic.severity),
                "code": diagnostic.code,
                "message": diagnostic.message,
                "path": json_path(&diagnostic.location.path),
                "line": diagnostic.location.span.as_ref().map(|span| span.start_line),
                "column": diagnostic.location.span.as_ref().map(|span| span.start_column),
            })
        })
        .collect();
    let change_report = changes
        .map(|reference| {
            CanonicalCorpus::load_at_git_ref(&context.repository, &context.config, reference).map(
                |base| ManagedChangeValidator::validate(&base, &context.corpus, &context.config),
            )
        })
        .transpose()
        .map_err(CliError::boxed)?;
    if let Some(report) = &change_report {
        diagnostics.extend(report.diagnostics.iter().map(|diagnostic| {
            json!({
                "severity": "error",
                "code": diagnostic.code,
                "message": diagnostic.message,
                "path": json_path(&diagnostic.path),
            })
        }));
    }
    if let Some(error) = &logic_error {
        diagnostics.push(json!({
            "severity": "error",
            "code": "invalid-repository-logic",
            "message": error.to_string(),
            "path": json_path(&context.repository.config_dir().join("logic.dl")),
        }));
    }
    let valid = report.is_valid()
        && logic_error.is_none()
        && change_report
            .as_ref()
            .is_none_or(|report| report.is_valid());
    if json_output {
        print_json(json!({ "valid": valid, "diagnostics": diagnostics }))?;
    } else if diagnostics.is_empty() {
        println!("valid");
    } else {
        for diagnostic in &diagnostics {
            println!(
                "{}: {}: {}",
                diagnostic["severity"].as_str().unwrap_or("error"),
                diagnostic["code"].as_str().unwrap_or("validation"),
                diagnostic["message"].as_str().unwrap_or_default()
            );
        }
    }
    if valid {
        refresh_derived(&context)?;
        Ok(())
    } else {
        Err(CliError::silent("validation failed"))
    }
}

fn query(name: &str, raw: &[String], json_output: bool) -> Result<(), CliError> {
    let mut parsed = BTreeMap::new();
    for argument in raw {
        let (name, value) = split_assignment(argument)?;
        if parsed.insert(name.to_owned(), value.to_owned()).is_some() {
            return Err(CliError::message(format!("duplicate query input {name:?}")));
        }
    }
    execute_query(name, parsed, json_output)
}

fn execute_query(
    name: &str,
    raw: BTreeMap<String, String>,
    json_output: bool,
) -> Result<(), CliError> {
    let context = Context::load()?;
    let query = context
        .config
        .queries
        .get(name)
        .ok_or_else(|| CliError::message(format!("unknown query {name:?}")))?;
    let mut inputs = BTreeMap::new();
    for argument in query
        .arguments
        .iter()
        .filter(|argument| argument.mode == docgraph_core::ArgumentMode::Input)
    {
        let value = raw
            .get(argument.name.as_str())
            .or(argument.default.as_ref())
            .ok_or_else(|| CliError::message(format!("missing --arg {}=...", argument.name)))?;
        inputs.insert(
            argument.name.clone(),
            parse_query_value(value, argument.value_type)?,
        );
    }
    if raw.keys().any(|name| !inputs.contains_key(name)) {
        return Err(CliError::message(
            "query received an unknown or duplicate input",
        ));
    }
    let external = context.external_records(true)?;
    let result = QueryEngine::new_with_external(&context.config, &context.graph, &external)
        .map_err(CliError::boxed)?
        .execute(name, inputs)
        .map_err(CliError::boxed)?;
    let columns: Vec<_> = result
        .columns
        .iter()
        .map(|column| json!({ "name": column.name, "type": query_type_name(column.value_type) }))
        .collect();
    let rows: Vec<_> = result
        .rows
        .iter()
        .map(|row| {
            JsonValue::Object(
                row.iter()
                    .map(|(key, value)| (key.clone(), query_value_json(value)))
                    .collect(),
            )
        })
        .collect();
    if json_output {
        print_json(json!({ "query": result.query, "columns": columns, "rows": rows }))
    } else {
        for row in rows {
            println!("{}", serde_json::to_string(&row).map_err(CliError::boxed)?);
        }
        Ok(())
    }
}

fn instructions(action: InstructionAction, json_output: bool) -> Result<(), CliError> {
    let context = Context::load()?;
    let service =
        InstructionService::new(&context.repository, &context.config).map_err(CliError::boxed)?;
    let skill = PortableSkillService::new(&context.repository).map_err(CliError::boxed)?;
    match action {
        InstructionAction::Sync { dry_run } => {
            let skill_changes = skill.sync(dry_run).map_err(CliError::boxed)?;
            let changes = service.sync(dry_run).map_err(CliError::boxed)?;
            if json_output {
                let mut output_changes: Vec<_> = skill_changes
                    .iter()
                    .map(|change| {
                        json!({
                            "path": json_path(&change.path),
                            "original": change.original,
                            "intended": change.intended,
                        })
                    })
                    .collect();
                output_changes.extend(changes.iter().map(|change| {
                    json!({
                        "path": json_path(&change.path),
                        "original": change.original,
                        "intended": change.intended,
                    })
                }));
                print_json(json!({
                    "dry_run": dry_run,
                    "changes": output_changes,
                }))
            } else {
                for change in &skill_changes {
                    if dry_run {
                        println!(
                            "{}",
                            render_text_patch(
                                &change.path,
                                change.original.as_deref().unwrap_or_default(),
                                &change.intended,
                            )
                        );
                    } else {
                        println!("updated {}", change.path.display());
                    }
                }
                for change in &changes {
                    if dry_run {
                        println!(
                            "{}",
                            render_text_patch(
                                &change.path,
                                change.original.as_deref().unwrap_or_default(),
                                &change.intended,
                            )
                        );
                    } else {
                        println!("updated {}", change.path.display());
                    }
                }
                Ok(())
            }
        }
        InstructionAction::Check => {
            let statuses = service.check().map_err(CliError::boxed)?;
            let skill_status = skill.check().map_err(CliError::boxed)?;
            let current = skill_status == PortableSkillStatus::Current
                && statuses
                    .iter()
                    .all(|(_, status)| *status == InstructionStatus::Current);
            if json_output {
                print_json(json!({
                    "current": current,
                    "targets": statuses.iter().map(|(path, status)| json!({ "path": json_path(path), "status": format!("{status:?}").to_lowercase() })).collect::<Vec<_>>(),
                    "skill": {
                        "path": PORTABLE_SKILL_PATH,
                        "status": format!("{skill_status:?}").to_lowercase(),
                    },
                }))?;
            } else {
                for (path, status) in statuses {
                    println!(
                        "{}\t{}",
                        path.display(),
                        format!("{status:?}").to_lowercase()
                    );
                }
                println!(
                    "{PORTABLE_SKILL_PATH}\t{}",
                    format!("{skill_status:?}").to_lowercase()
                );
            }
            if current {
                Ok(())
            } else {
                Err(CliError::silent(
                    "agent instructions or portable skill are not current",
                ))
            }
        }
    }
}

fn frontmatter(action: FrontmatterAction, json_output: bool) -> Result<(), CliError> {
    match action {
        FrontmatterAction::Sync { dry_run } => {
            mutate(MutationRequest::SyncFrontmatter, dry_run, json_output)
        }
        FrontmatterAction::Check => {
            let context = Context::load()?;
            let projections = GeneratedFrontmatterIndex::new(&context.graph, &context.config);
            let statuses: Vec<_> = context
                .graph
                .documents
                .iter()
                .enumerate()
                .filter(|(_, document)| document.entity.is_some())
                .map(|(index, document)| {
                    let status = projections.check(&context.corpus, &context.graph, index);
                    (document.path.clone(), status)
                })
                .collect();
            let current = statuses
                .iter()
                .all(|(_, status)| matches!(status, Ok(GeneratedBlockStatus::Current)));
            if json_output {
                print_json(
                    json!({ "current": current, "documents": statuses.iter().map(|(path, status)| json!({ "path": json_path(path), "status": match status { Ok(status) => format!("{status:?}").to_lowercase(), Err(_) => "malformed".to_owned() } })).collect::<Vec<_>>() }),
                )?;
            } else {
                for (path, status) in statuses {
                    let status = match status {
                        Ok(status) => format!("{status:?}").to_lowercase(),
                        Err(error) => format!("malformed: {error}"),
                    };
                    println!("{}\t{status}", path.display());
                }
            }
            if current {
                Ok(())
            } else {
                Err(CliError::silent("generated frontmatter is not current"))
            }
        }
    }
}

fn parse_properties(
    raw: &[String],
    schema: &BTreeMap<String, PropertyConfig>,
    owner: &str,
) -> Result<BTreeMap<String, Value>, CliError> {
    raw.iter()
        .map(|assignment| {
            let (name, raw) = split_assignment(assignment)?;
            let property = schema.get(name).ok_or_else(|| {
                CliError::message(format!("undeclared {owner} property {name:?}"))
            })?;
            Ok((name.to_owned(), parse_toml_value(raw, property)?))
        })
        .collect()
}

fn parse_toml_value(raw: &str, property: &PropertyConfig) -> Result<Value, CliError> {
    if property.property_type == PropertyType::String {
        // Bare text is the ordinary form, but a TOML string literal is what an
        // author reaches for when the value needs quoting; accept both rather
        // than storing the quotes as part of the value.
        return Ok(parse_string_literal(raw).unwrap_or_else(|| Value::from(raw)));
    }
    let source = format!("value = {raw}");
    let document = source
        .parse::<toml_edit::DocumentMut>()
        .map_err(CliError::boxed)?;
    document["value"]
        .as_value()
        .cloned()
        .ok_or_else(|| CliError::message("property value must be a TOML scalar or array"))
}

fn parse_string_literal(raw: &str) -> Option<Value> {
    let document = format!("value = {raw}")
        .parse::<toml_edit::DocumentMut>()
        .ok()?;
    document["value"].as_value()?.as_str().map(Value::from)
}

fn split_assignment(value: &str) -> Result<(&str, &str), CliError> {
    value
        .split_once('=')
        .filter(|(name, _)| !name.is_empty())
        .ok_or_else(|| CliError::message(format!("expected name=value, found {value:?}")))
}

fn parse_query_value(raw: &str, value_type: QueryValueType) -> Result<QueryValue, CliError> {
    match value_type {
        QueryValueType::String => Ok(QueryValue::String(raw.to_owned())),
        QueryValueType::Integer => raw
            .parse()
            .map(QueryValue::Integer)
            .map_err(CliError::boxed),
        QueryValueType::Float => raw.parse().map(QueryValue::Float).map_err(CliError::boxed),
        QueryValueType::Boolean => raw
            .parse()
            .map(QueryValue::Boolean)
            .map_err(CliError::boxed),
        QueryValueType::Datetime => Ok(QueryValue::Datetime(raw.to_owned())),
        QueryValueType::Entity => Ok(QueryValue::Entity(raw.to_owned())),
        QueryValueType::Section => Ok(QueryValue::Section(raw.to_owned())),
    }
}

fn property_schema_json(schema: &BTreeMap<String, PropertyConfig>) -> JsonValue {
    JsonValue::Object(
        schema
            .iter()
            .map(|(name, property)| {
                (
                    name.clone(),
                    json!({
                        "type": format!("{:?}", property.property_type).to_lowercase(),
                        "required": property.required,
                        "items": property.items.map(|item| format!("{item:?}").to_lowercase()),
                        "values": property.values.as_ref().map(|values| values.iter().map(scalar_value_json).collect::<Vec<_>>()),
                    }),
                )
            })
            .collect(),
    )
}

fn scalar_value_json(value: &ScalarValue) -> JsonValue {
    match value {
        ScalarValue::String(value) => JsonValue::String(value.clone()),
        ScalarValue::Integer(value) => json!(value),
        ScalarValue::Float(value) => json!(value),
        ScalarValue::Boolean(value) => json!(value),
        ScalarValue::Datetime(value) => JsonValue::String(value.to_string()),
    }
}

fn toml_value_json(value: &Value) -> JsonValue {
    if let Some(value) = value.as_str() {
        JsonValue::String(value.to_owned())
    } else if let Some(value) = value.as_integer() {
        JsonValue::from(value)
    } else if let Some(value) = value.as_float() {
        JsonValue::from(value)
    } else if let Some(value) = value.as_bool() {
        JsonValue::from(value)
    } else if let Some(value) = value.as_datetime() {
        JsonValue::String(value.to_string())
    } else if let Some(value) = value.as_array() {
        JsonValue::Array(value.iter().map(toml_value_json).collect())
    } else {
        JsonValue::Null
    }
}

fn print_plan(plan: &MutationPlan, dry_run: bool, json_output: bool) -> Result<(), CliError> {
    if json_output {
        return print_json(json!({
            "dry_run": dry_run,
            "fingerprint": plan.fingerprint.to_string(),
            "changes": plan.changes.iter().map(|change| json!({ "path": json_path(&change.path), "original": change.original, "intended": change.intended })).collect::<Vec<_>>(),
        }));
    }
    if plan.changes.is_empty() {
        println!("no changes");
    } else if dry_run {
        for change in &plan.changes {
            println!("{}", render_patch(change));
        }
    } else {
        for change in &plan.changes {
            let action = match (&change.original, &change.intended) {
                (None, Some(_)) => "created",
                (Some(_), None) => "deleted",
                _ => "updated",
            };
            println!("{action} {}", change.path.display());
        }
    }
    Ok(())
}

fn render_patch(change: &docgraph_core::FileChange) -> String {
    render_text_patch(
        &change.path,
        change.original.as_deref().unwrap_or_default(),
        change.intended.as_deref().unwrap_or_default(),
    )
}

fn render_text_patch(path: &std::path::Path, original: &str, intended: &str) -> String {
    let old: Vec<_> = original.lines().collect();
    let new: Vec<_> = intended.lines().collect();
    let prefix = old
        .iter()
        .zip(&new)
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = old[prefix..]
        .iter()
        .rev()
        .zip(new[prefix..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    let mut output = format!("--- {}\n+++ {}\n", path.display(), path.display());
    for line in &old[prefix..old.len().saturating_sub(suffix)] {
        output.push_str(&format!("-{line}\n"));
    }
    for line in &new[prefix..new.len().saturating_sub(suffix)] {
        output.push_str(&format!("+{line}\n"));
    }
    output
}

fn refresh_derived(context: &Context) -> Result<(), CliError> {
    let state = DerivedState::discover(&context.repository).map_err(CliError::boxed)?;
    let external = context.external_records(false)?;
    if let Some(config) = &context.config.project.embeddings {
        let provider = CommandEmbeddingProvider::new(config);
        state
            .refresh_with_external(
                &context.corpus,
                &context.graph,
                &external,
                Some((config, &provider)),
            )
            .map_err(CliError::boxed)
    } else {
        state
            .refresh_with_external(&context.corpus, &context.graph, &external, None)
            .map_err(CliError::boxed)
    }
}

fn node_name(graph: &GraphIndex, node: &GraphNode) -> String {
    match node {
        GraphNode::Document(index) => json_path(&graph.documents[*index].path),
        GraphNode::Entity(id) | GraphNode::ExternalUri(id) | GraphNode::Unresolved(id) => {
            id.clone()
        }
        GraphNode::Section(index) => {
            let section = &graph.sections[*index];
            let document = &graph.documents[section.document];
            section.id.as_ref().map_or_else(
                || format!("{}#<missing>", json_path(&document.path)),
                |id| {
                    document.entity.as_ref().map_or_else(
                        || format!("{}#{}", json_path(&document.path), id.as_str()),
                        |entity| format!("{entity}#{}", id.as_str()),
                    )
                },
            )
        }
    }
}

fn json_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn origin_name(origin: RelationOrigin) -> &'static str {
    match origin {
        RelationOrigin::Explicit => "explicit",
        RelationOrigin::MarkdownLink => "markdown_link",
    }
}

fn severity_name(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Error => "error",
    }
}

fn query_type_name(value_type: QueryValueType) -> &'static str {
    match value_type {
        QueryValueType::String => "string",
        QueryValueType::Integer => "integer",
        QueryValueType::Float => "float",
        QueryValueType::Boolean => "boolean",
        QueryValueType::Datetime => "datetime",
        QueryValueType::Entity => "entity",
        QueryValueType::Section => "section",
    }
}

fn query_value_json(value: &QueryValue) -> JsonValue {
    match value {
        QueryValue::String(value)
        | QueryValue::Datetime(value)
        | QueryValue::Entity(value)
        | QueryValue::Section(value) => JsonValue::String(value.clone()),
        QueryValue::Integer(value) => json!(value),
        QueryValue::Float(value) => json!(value),
        QueryValue::Boolean(value) => json!(value),
    }
}

fn print_semantic_search(
    context: &Context,
    query: &str,
    result: &SemanticSearchResult,
    json_output: bool,
) -> Result<(), CliError> {
    let mode = match result.mode {
        SemanticSearchMode::Vector => "vector",
        SemanticSearchMode::FullTextFallback => "full_text_fallback",
    };
    if json_output {
        let rows: Vec<_> = result
            .hits
            .iter()
            .map(|hit| {
                json!({
                    "node": hit.node,
                    "score": hit.score,
                    "snippet": hit.snippet,
                    "external": external_hit_metadata(context, &hit.node),
                })
            })
            .collect();
        print_json(json!({
            "query": query,
            "mode": mode,
            "reason": result.reason,
            "rows": rows,
        }))
    } else {
        if let Some(reason) = &result.reason {
            println!("mode={mode}\treason={reason}");
        } else {
            println!("mode={mode}");
        }
        for hit in &result.hits {
            println!(
                "{:.3}\t{}\t{}\t{}",
                hit.score,
                hit.node,
                external_hit_label(context, &hit.node),
                hit.snippet
            );
        }
        Ok(())
    }
}

fn print_json(value: JsonValue) -> Result<(), CliError> {
    println!(
        "{}",
        serde_json::to_string_pretty(&value).map_err(CliError::boxed)?
    );
    Ok(())
}

#[derive(Debug)]
struct CliError {
    message: String,
    silent: bool,
}

impl CliError {
    fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            silent: false,
        }
    }

    fn silent(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            silent: true,
        }
    }

    fn boxed(error: impl Error) -> Self {
        Self::message(error.to_string())
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::path::PathBuf;

    #[test]
    fn command_hierarchy_is_internally_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn dry_run_patch_contains_only_the_changed_middle() {
        let change = docgraph_core::FileChange {
            path: PathBuf::from("docs/task.md"),
            original: Some("same\nold\ntail\n".to_owned()),
            intended: Some("same\nnew\ntail\n".to_owned()),
            original_hash: Some([0; 32]),
        };
        let patch = render_patch(&change);
        assert!(patch.contains("-old\n+new"));
        assert!(!patch.contains("+tail"));
    }
}
