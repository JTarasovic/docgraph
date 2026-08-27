use clap::{Parser, Subcommand, ValueEnum};
use docgraph_core::{
    CanonicalCorpus, DerivedState, DiagnosticSeverity, GeneratedBlockStatus, GraphIndex, GraphNode,
    GraphTraversal, InstructionService, InstructionStatus, MutationPlan, MutationRequest,
    MutationService, PropertyConfig, PropertyType, QueryValueType, RelationOrigin, Repository,
    RepositoryConfig, Validator, check_generated_frontmatter,
};
use docgraph_logic::{QueryEngine, QueryValue};
use serde_json::{Value as JsonValue, json};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;
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
    /// Adopt an existing document into the managed graph.
    Adopt {
        path: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long = "type")]
        entity_type: String,
        #[arg(long = "property")]
        properties: Vec<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Describe the configured repository model.
    Describe {
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
    /// Apply a configured workflow transition.
    Transition {
        entity: String,
        state: String,
        #[arg(long)]
        dry_run: bool,
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
        entity: String,
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
    Validate,
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
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DescribeKind {
    Type,
    Relation,
    Workflow,
    Query,
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

struct Context {
    repository: Repository,
    config: RepositoryConfig,
    corpus: CanonicalCorpus,
    graph: GraphIndex,
}

impl Context {
    fn load() -> Result<Self, CliError> {
        MutationService::open(".")
            .map_err(CliError::boxed)?
            .recover_pending()
            .map_err(CliError::boxed)?;
        let repository = Repository::discover(".").map_err(CliError::boxed)?;
        let config = RepositoryConfig::load(&repository).map_err(CliError::boxed)?;
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
        let state = DerivedState::discover(&self.repository).map_err(CliError::boxed)?;
        state
            .ensure_fresh(&self.corpus, &self.graph)
            .map_err(CliError::boxed)?;
        Ok(state)
    }
}

fn main() -> ExitCode {
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
        Command::Adopt {
            path,
            id,
            entity_type,
            properties,
            dry_run,
        } => {
            let service = MutationService::open(".").map_err(CliError::boxed)?;
            let entity =
                service.config().entities.get(&entity_type).ok_or_else(|| {
                    CliError::message(format!("unknown entity type {entity_type:?}"))
                })?;
            let properties = parse_properties(&properties, &entity.property, "entity")?;
            let plan = service
                .apply(
                    &MutationRequest::Adopt {
                        path,
                        id,
                        entity_type,
                        properties,
                    },
                    dry_run,
                )
                .map_err(CliError::boxed)?;
            print_plan(&plan, dry_run, cli.json)
        }
        Command::Describe { kind, name } => {
            let context = Context::load()?;
            describe(&context.config, kind, name.as_deref(), cli.json)
        }
        Command::Get { reference } => {
            let context = Context::load()?;
            context.ensure_derived()?;
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
                        })
                    })
                    .collect();
                print_json(json!({ "query": query, "rows": rows }))
            } else {
                for hit in hits {
                    println!("{:.3}\t{}\t{}", hit.score, hit.node, hit.snippet);
                }
                Ok(())
            }
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
        Command::Neighbors { entity, all } => {
            let context = Context::load()?;
            context.ensure_derived()?;
            let node = GraphNode::Entity(entity);
            let origin = (!all).then_some(RelationOrigin::Explicit);
            let neighbors = GraphTraversal::new(&context.graph).neighbors(&node, origin);
            if cli.json {
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
                print_json(json!({ "rows": rows }))
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
        Command::Path {
            source,
            target,
            all,
        } => {
            let context = Context::load()?;
            context.ensure_derived()?;
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
        Command::Validate => validate(cli.json),
        Command::Query { name, arguments } => query(&name, &arguments, cli.json),
        Command::Instructions { action } => instructions(action, cli.json),
        Command::Frontmatter { action } => frontmatter(action, cli.json),
    }
}

fn describe(
    config: &RepositoryConfig,
    kind: Option<DescribeKind>,
    name: Option<&str>,
    json_output: bool,
) -> Result<(), CliError> {
    let value = match (kind, name) {
        (None, None) => json!({
            "project": config.project.name,
            "documents_root": json_path(&config.project.documents.root),
            "entity_types": config.entities.keys().collect::<Vec<_>>(),
            "relations": config.relations.keys().collect::<Vec<_>>(),
            "workflows": config.workflows.keys().collect::<Vec<_>>(),
            "queries": config.queries.keys().collect::<Vec<_>>(),
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
            let arguments: Vec<_> = item.arguments.iter().map(|argument| json!({ "name": argument.name, "mode": format!("{:?}", argument.mode).to_lowercase(), "type": format!("{:?}", argument.value_type).to_lowercase() })).collect();
            json!({ "name": name, "description": item.description, "predicate": item.predicate, "arguments": arguments })
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

fn get(context: &Context, reference: &str, json_output: bool) -> Result<(), CliError> {
    let node = resolve_graph_reference(&context.graph, reference)?;
    let value = if let GraphNode::Entity(id) = &node {
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
            "relations": relation_context(&context.graph, &node),
        })
    } else if let GraphNode::Section(index) = node {
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
            "id": reference,
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
            "relations": relation_context(&context.graph, &node),
        })
    } else {
        unreachable!("canonical graph references resolve only to entities or sections");
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

fn resolve_graph_reference(graph: &GraphIndex, reference: &str) -> Result<GraphNode, CliError> {
    if graph.entities.iter().any(|entity| entity.id == reference) {
        return Ok(GraphNode::Entity(reference.to_owned()));
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
        #[arg(long)]
        dry_run: bool,
    },
    Unset {
        entity: String,
        property: String,
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

fn mutate(request: MutationRequest, dry_run: bool, json_output: bool) -> Result<(), CliError> {
    let service = MutationService::open(".").map_err(CliError::boxed)?;
    let plan = service.apply(&request, dry_run).map_err(CliError::boxed)?;
    print_plan(&plan, dry_run, json_output)
}

fn property(action: PropertyAction, json_output: bool) -> Result<(), CliError> {
    let (entity, property, raw_value, dry_run) = match action {
        PropertyAction::Set {
            entity,
            property,
            value,
            dry_run,
        } => (entity, property, Some(value), dry_run),
        PropertyAction::Unset {
            entity,
            property,
            dry_run,
        } => (entity, property, None, dry_run),
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
    mutate(request, dry_run, json_output)
}

fn validate(json_output: bool) -> Result<(), CliError> {
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
    if let Some(error) = &logic_error {
        diagnostics.push(json!({
            "severity": "error",
            "code": "invalid-repository-logic",
            "message": error.to_string(),
            "path": json_path(&context.repository.config_dir().join("logic.dl")),
        }));
    }
    let valid = report.is_valid() && logic_error.is_none();
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
    let context = Context::load()?;
    context.ensure_derived()?;
    let query = context
        .config
        .queries
        .get(name)
        .ok_or_else(|| CliError::message(format!("unknown query {name:?}")))?;
    let raw: BTreeMap<_, _> = raw
        .iter()
        .map(|argument| split_assignment(argument))
        .collect::<Result<_, _>>()?;
    let mut inputs = BTreeMap::new();
    for argument in query
        .arguments
        .iter()
        .filter(|argument| argument.mode == docgraph_core::ArgumentMode::Input)
    {
        let value = raw
            .get(argument.name.as_str())
            .ok_or_else(|| CliError::message(format!("missing --arg {}=...", argument.name)))?;
        inputs.insert(
            argument.name.clone(),
            parse_query_value(value, argument.value_type)?,
        );
    }
    if raw.len() != inputs.len() {
        return Err(CliError::message(
            "query received an unknown or duplicate input",
        ));
    }
    let result = QueryEngine::new(&context.config, &context.graph)
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
    match action {
        InstructionAction::Sync { dry_run } => {
            let changes = service.sync(dry_run).map_err(CliError::boxed)?;
            if json_output {
                print_json(json!({
                    "dry_run": dry_run,
                    "changes": changes.iter().map(|change| json!({
                        "path": json_path(&change.path),
                        "original": change.original,
                        "intended": change.intended,
                    })).collect::<Vec<_>>()
                }))
            } else {
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
            let current = statuses
                .iter()
                .all(|(_, status)| *status == InstructionStatus::Current);
            if json_output {
                print_json(
                    json!({ "current": current, "targets": statuses.iter().map(|(path, status)| json!({ "path": json_path(path), "status": format!("{status:?}").to_lowercase() })).collect::<Vec<_>>() }),
                )?;
            } else {
                for (path, status) in statuses {
                    println!(
                        "{}\t{}",
                        path.display(),
                        format!("{status:?}").to_lowercase()
                    );
                }
            }
            if current {
                Ok(())
            } else {
                Err(CliError::silent("agent instructions are not current"))
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
            let statuses: Vec<_> = context
                .graph
                .documents
                .iter()
                .enumerate()
                .filter(|(_, document)| document.entity.is_some())
                .map(|(index, document)| {
                    let status = check_generated_frontmatter(
                        &context.corpus,
                        &context.graph,
                        &context.config,
                        index,
                    );
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
        return Ok(Value::from(raw));
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
                    }),
                )
            })
            .collect(),
    )
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
            println!("updated {}", change.path.display());
        }
    }
    Ok(())
}

fn render_patch(change: &docgraph_core::FileChange) -> String {
    render_text_patch(&change.path, &change.original, &change.intended)
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
    state
        .refresh(&context.corpus, &context.graph)
        .map_err(CliError::boxed)
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
            original: "same\nold\ntail\n".to_owned(),
            intended: "same\nnew\ntail\n".to_owned(),
            original_hash: [0; 32],
        };
        let patch = render_patch(&change);
        assert!(patch.contains("-old\n+new"));
        assert!(!patch.contains("+tail"));
    }
}
