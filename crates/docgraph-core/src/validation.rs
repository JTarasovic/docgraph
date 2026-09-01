use crate::identity::validate_entity_id;
use crate::{
    ArgumentMode, CommandOperation, DiagnosticSeverity, GeneratedFrontmatterIndex, GraphIndex,
    GraphLocation, GraphNode, PropertyConfig, PropertyType, RelationOrigin, Repository,
    RepositoryConfig, ScalarType, ScalarValue,
};
use docgraph_markdown::SourceSpan;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use toml_edit::Value;

const BUILT_IN_COMMANDS: &[&str] = &[
    "adopt",
    "describe",
    "get",
    "search",
    "transition",
    "workflow",
    "property",
    "relate",
    "unrelate",
    "neighbors",
    "path",
    "normalize",
    "validate",
    "query",
    "instructions",
    "frontmatter",
    "help",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationLocation {
    pub path: PathBuf,
    pub span: Option<SourceSpan>,
}

impl From<&GraphLocation> for ValidationLocation {
    fn from(location: &GraphLocation) -> Self {
        Self {
            path: location.path.clone(),
            span: Some(location.span.clone()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: &'static str,
    pub message: String,
    pub location: ValidationLocation,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidationReport {
    pub diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &ValidationDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

pub struct Validator<'a> {
    repository: &'a Repository,
    config: &'a RepositoryConfig,
    graph: &'a GraphIndex,
    diagnostics: Vec<ValidationDiagnostic>,
}

impl<'a> Validator<'a> {
    pub fn validate_corpus(
        repository: &'a Repository,
        config: &'a RepositoryConfig,
        corpus: &'a crate::CanonicalCorpus,
        graph: &'a GraphIndex,
    ) -> ValidationReport {
        let mut report = Self::validate(repository, config, graph);
        for file in &corpus.files {
            if let Some(frontmatter) = &file.document.yaml_frontmatter {
                report.diagnostics.push(ValidationDiagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: "yaml-frontmatter",
                    message: "YAML frontmatter found; docgraph frontmatter is TOML in +++ fences; run `docgraph frontmatter migrate`".to_owned(),
                    location: ValidationLocation {
                        path: file.path.clone(),
                        span: Some(frontmatter.span.clone()),
                    },
                });
            }
        }
        let projections = GeneratedFrontmatterIndex::new(graph, config);
        let entity_locations: HashMap<_, ValidationLocation> = graph
            .entities
            .iter()
            .map(|entity| (entity.document, (&entity.location).into()))
            .collect();
        for (document, node) in graph.documents.iter().enumerate() {
            if node.entity.is_none() {
                continue;
            }
            let (code, message) = match projections.check(corpus, graph, document) {
                Ok(crate::GeneratedBlockStatus::Current) => continue,
                Ok(crate::GeneratedBlockStatus::Missing) => (
                    "missing-generated-frontmatter",
                    "entity document has no generated frontmatter block".to_owned(),
                ),
                Ok(crate::GeneratedBlockStatus::Stale) => (
                    "stale-generated-frontmatter",
                    "entity document generated frontmatter is stale".to_owned(),
                ),
                Err(error) => ("malformed-generated-frontmatter", error.to_string()),
            };
            let location =
                entity_locations
                    .get(&document)
                    .cloned()
                    .unwrap_or_else(|| ValidationLocation {
                        path: node.path.clone(),
                        span: None,
                    });
            report.diagnostics.push(ValidationDiagnostic {
                severity: DiagnosticSeverity::Error,
                code,
                message,
                location,
            });
        }
        report
    }

    pub fn validate(
        repository: &'a Repository,
        config: &'a RepositoryConfig,
        graph: &'a GraphIndex,
    ) -> ValidationReport {
        let mut validator = Self {
            repository,
            config,
            graph,
            diagnostics: Vec::new(),
        };
        validator.validate_config();
        validator.validate_graph();
        validator.diagnostics.sort_by(|left, right| {
            left.location
                .path
                .cmp(&right.location.path)
                .then_with(|| {
                    left.location
                        .span
                        .as_ref()
                        .map(|span| span.bytes.start)
                        .cmp(&right.location.span.as_ref().map(|span| span.bytes.start))
                })
                .then_with(|| left.code.cmp(right.code))
                .then_with(|| left.message.cmp(&right.message))
        });
        ValidationReport {
            diagnostics: validator.diagnostics,
        }
    }

    fn validate_config(&mut self) {
        for (entity_type, entity) in &self.config.entities {
            if let Some(workflow) = &entity.workflow
                && !self.config.workflows.contains_key(workflow)
            {
                self.config_error(
                    "unknown-workflow",
                    format!("entity type {entity_type:?} references unknown workflow {workflow:?}"),
                    "entities.toml",
                    workflow,
                );
            }
            self.validate_property_schema(
                &entity.property,
                "entities.toml",
                &format!("entity type {entity_type:?}"),
            );
        }

        for (name, workflow) in &self.config.workflows {
            if !workflow.states.contains_key(&workflow.initial) {
                self.config_error(
                    "unknown-workflow-state",
                    format!(
                        "workflow {name:?} has unknown initial state {:?}",
                        workflow.initial
                    ),
                    "workflows.toml",
                    &workflow.initial,
                );
            }
            for (state_name, state) in &workflow.states {
                for target in &state.transitions {
                    if !workflow.states.contains_key(target) {
                        self.config_error(
                            "unknown-workflow-state",
                            format!(
                                "workflow {name:?} state {state_name:?} transitions to unknown state {target:?}"
                            ),
                            "workflows.toml",
                            target,
                        );
                    }
                }
            }
        }

        for (name, relation) in &self.config.relations {
            self.validate_endpoints(name, "source", &relation.source);
            self.validate_endpoints(name, "target", &relation.target);
            self.validate_property_schema(
                &relation.property,
                "relations.toml",
                &format!("relation {name:?}"),
            );
            if let Some(inverse_name) = &relation.inverse {
                match self.config.relations.get(inverse_name) {
                    None => self.config_error(
                        "unknown-inverse-relation",
                        format!("relation {name:?} references unknown inverse {inverse_name:?}"),
                        "relations.toml",
                        inverse_name,
                    ),
                    Some(inverse) => {
                        if inverse
                            .inverse
                            .as_deref()
                            .is_some_and(|other| other != name)
                        {
                            self.config_error(
                                "invalid-inverse-relation",
                                format!(
                                    "relation {name:?} names {inverse_name:?} as its inverse, but that relation names {:?}",
                                    inverse.inverse.as_deref().unwrap_or_default()
                                ),
                                "relations.toml",
                                inverse_name,
                            );
                        }
                        if !constraints_fit(&relation.source, &inverse.target)
                            || !constraints_fit(&relation.target, &inverse.source)
                        {
                            self.config_error(
                                "invalid-inverse-endpoints",
                                format!("relation {name:?} and inverse {inverse_name:?} have incompatible endpoints"),
                                "relations.toml",
                                inverse_name,
                            );
                        }
                    }
                }
            }
        }

        for (name, query) in &self.config.queries {
            let mut arguments = HashSet::new();
            for argument in &query.arguments {
                if !arguments.insert(&argument.name) {
                    self.config_error(
                        "duplicate-query-argument",
                        format!("query {name:?} repeats argument {:?}", argument.name),
                        "project.toml",
                        &argument.name,
                    );
                }
                if argument.default.is_some() && argument.mode != ArgumentMode::Input {
                    self.config_error(
                        "invalid-query-default",
                        format!(
                            "query {name:?} output argument {:?} cannot have a default",
                            argument.name
                        ),
                        "project.toml",
                        &argument.name,
                    );
                }
            }
            if query.arguments.is_empty() {
                self.config_error(
                    "empty-query-abi",
                    format!("query {name:?} must declare at least one argument"),
                    "project.toml",
                    name,
                );
            }
        }

        for (name, command) in &self.config.commands {
            if name.split('.').any(|segment| {
                segment.is_empty()
                    || !segment.chars().all(|character| {
                        character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
                    })
            }) {
                self.config_error(
                    "invalid-command-name",
                    format!("command {name:?} contains an empty path segment"),
                    "commands.toml",
                    name,
                );
            }
            if name
                .split('.')
                .next()
                .is_some_and(|root| BUILT_IN_COMMANDS.contains(&root))
            {
                self.config_error(
                    "reserved-command-name",
                    format!("command {name:?} conflicts with a built-in command"),
                    "commands.toml",
                    name,
                );
            }
            match &command.operation {
                CommandOperation::Query { query, entity_type } => {
                    let configured_query = self.config.queries.get(query);
                    if configured_query.is_none() {
                        self.config_error(
                            "unknown-command-query",
                            format!("command {name:?} references unknown query {query:?}"),
                            "commands.toml",
                            query,
                        );
                    }
                    if let Some(entity_type) = entity_type
                        && !self.config.entities.contains_key(entity_type)
                    {
                        self.config_error(
                            "unknown-command-entity-type",
                            format!(
                                "command {name:?} references unknown entity type {entity_type:?}"
                            ),
                            "commands.toml",
                            entity_type,
                        );
                    }
                    if entity_type.is_some()
                        && !configured_query.is_some_and(|query| {
                            query.arguments.first().is_some_and(|argument| {
                                argument.mode == ArgumentMode::Input
                                    && argument.value_type == crate::QueryValueType::Entity
                            })
                        })
                    {
                        self.config_error(
                            "invalid-command-entity-input",
                            format!("command {name:?} with entity_type requires an entity-valued first query input"),
                            "commands.toml",
                            query,
                        );
                    }
                }
                CommandOperation::Transition {
                    entity_type,
                    target_state,
                } => {
                    let workflow = self
                        .config
                        .entities
                        .get(entity_type)
                        .and_then(|entity| entity.workflow.as_ref())
                        .and_then(|workflow| self.config.workflows.get(workflow));
                    if workflow.is_none() {
                        self.config_error(
                            "invalid-command-workflow",
                            format!("command {name:?} entity type {entity_type:?} has no configured workflow"),
                            "commands.toml",
                            entity_type,
                        );
                    } else if !workflow
                        .is_some_and(|workflow| workflow.states.contains_key(target_state))
                    {
                        self.config_error(
                            "unknown-command-target-state",
                            format!(
                                "command {name:?} references unknown target state {target_state:?}"
                            ),
                            "commands.toml",
                            target_state,
                        );
                    }
                }
                CommandOperation::AddRelation {
                    entity_type,
                    relation,
                } => {
                    if !self.config.entities.contains_key(entity_type) {
                        self.config_error(
                            "unknown-command-entity-type",
                            format!(
                                "command {name:?} references unknown entity type {entity_type:?}"
                            ),
                            "commands.toml",
                            entity_type,
                        );
                    }
                    if !self.config.relations.contains_key(relation) {
                        self.config_error(
                            "unknown-command-relation",
                            format!("command {name:?} references unknown relation {relation:?}"),
                            "commands.toml",
                            relation,
                        );
                    }
                    if self
                        .config
                        .relations
                        .get(relation)
                        .is_some_and(|configured| {
                            !configured.source.is_empty()
                                && !configured.source.iter().any(|source| source == entity_type)
                        })
                    {
                        self.config_error(
                            "invalid-command-relation-source",
                            format!("command {name:?} entity type {entity_type:?} cannot source relation {relation:?}"),
                            "commands.toml",
                            relation,
                        );
                    }
                }
            }
        }
    }

    fn validate_endpoints(&mut self, relation: &str, side: &str, endpoints: &[String]) {
        for endpoint in endpoints {
            if !matches!(endpoint.as_str(), "document" | "section" | "external")
                && !self.config.entities.contains_key(endpoint)
            {
                self.config_error(
                    "unknown-endpoint-type",
                    format!("relation {relation:?} has unknown {side} endpoint type {endpoint:?}"),
                    "relations.toml",
                    endpoint,
                );
            }
        }
    }

    fn validate_property_schema(
        &mut self,
        properties: &BTreeMap<String, PropertyConfig>,
        file: &str,
        owner: &str,
    ) {
        for (name, property) in properties {
            match property.property_type {
                PropertyType::Array => {
                    if property.items.is_none() {
                        self.config_error(
                            "missing-array-item-type",
                            format!("{owner} property {name:?} is an array but has no item type"),
                            file,
                            name,
                        );
                    }
                    if property.values.is_some() {
                        self.config_error(
                            "invalid-property-enum",
                            format!("{owner} array property {name:?} cannot declare scalar values"),
                            file,
                            name,
                        );
                    }
                }
                scalar => {
                    if property.items.is_some() {
                        self.config_error(
                            "unexpected-array-item-type",
                            format!("{owner} scalar property {name:?} cannot declare an item type"),
                            file,
                            name,
                        );
                    }
                    if let Some(values) = &property.values {
                        for value in values {
                            if !scalar_value_matches(value, scalar) {
                                self.config_error(
                                    "invalid-property-enum",
                                    format!("{owner} property {name:?} has an enumerated value of the wrong type"),
                                    file,
                                    name,
                                );
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn validate_graph(&mut self) {
        for diagnostic in &self.graph.diagnostics {
            self.error(
                "malformed-managed-frontmatter",
                format!("{:?}", diagnostic.kind),
                (&diagnostic.location).into(),
            );
        }

        let mut entity_ids: HashMap<&str, Vec<usize>> = HashMap::new();
        for (index, entity) in self.graph.entities.iter().enumerate() {
            entity_ids.entry(&entity.id).or_default().push(index);
            let Some(entity_config) = self.config.entities.get(&entity.entity_type) else {
                self.error(
                    "unknown-entity-type",
                    format!(
                        "entity {:?} has unknown type {:?}",
                        entity.id, entity.entity_type
                    ),
                    (&entity.location).into(),
                );
                continue;
            };
            if let Err(error) = validate_entity_id(&entity.id, &entity.entity_type) {
                self.error(
                    "invalid-entity-id",
                    format!("entity ID {:?} {error}", entity.id),
                    (&entity.location).into(),
                );
            }
            self.validate_entity_state(entity, entity_config.workflow.as_deref());
            self.validate_values(
                &entity.properties,
                &entity_config.property,
                &format!("entity {:?}", entity.id),
                &entity.location,
            );
        }
        for (id, indexes) in entity_ids {
            if indexes.len() > 1 {
                for index in indexes {
                    self.error(
                        "duplicate-entity-id",
                        format!("entity ID {id:?} is declared more than once"),
                        (&self.graph.entities[index].location).into(),
                    );
                }
            }
        }

        let mut section_ids: HashMap<&str, Vec<usize>> = HashMap::new();
        for (index, section) in self.graph.sections.iter().enumerate() {
            match &section.id {
                Some(id) => section_ids.entry(id.as_str()).or_default().push(index),
                None => self.error(
                    "missing-section-id",
                    format!(
                        "heading {:?} has no stable section ID; if the heading should remain, preview repository-wide ID insertion with `docgraph normalize --dry-run`, then run `docgraph normalize`",
                        section.heading
                    ),
                    (&section.location).into(),
                ),
            }
        }
        for (id, indexes) in section_ids {
            if indexes.len() > 1 {
                for index in indexes {
                    self.error(
                        "duplicate-section-id",
                        format!("stable section ID {id:?} is used more than once"),
                        (&self.graph.sections[index].location).into(),
                    );
                }
            }
        }

        let mut triples: HashMap<(GraphNode, &str, GraphNode), Vec<usize>> = HashMap::new();
        for (index, relation) in self.graph.relations.iter().enumerate() {
            if relation.origin == RelationOrigin::MarkdownLink {
                if matches!(relation.target, GraphNode::Unresolved(_)) {
                    self.diagnostics.push(ValidationDiagnostic {
                        severity: self.config.project.validation.broken_internal_links,
                        code: "broken-internal-link",
                        message: format!(
                            "Markdown link target {:?} cannot be resolved",
                            relation.target
                        ),
                        location: (&relation.location).into(),
                    });
                }
                continue;
            }
            let Some(relation_config) = self.config.relations.get(&relation.predicate) else {
                self.error(
                    "unknown-relation",
                    format!(
                        "explicit relation uses unknown predicate {:?}",
                        relation.predicate
                    ),
                    (&relation.location).into(),
                );
                continue;
            };
            if matches!(relation.source, GraphNode::Unresolved(_))
                || matches!(relation.target, GraphNode::Unresolved(_))
            {
                self.error(
                    "unresolved-managed-reference",
                    format!(
                        "explicit relation {:?} has an unresolved endpoint",
                        relation.predicate
                    ),
                    (&relation.location).into(),
                );
            }
            self.validate_relation_endpoint(
                &relation.source,
                &relation_config.source,
                "source",
                &relation.predicate,
                &relation.location,
            );
            self.validate_relation_endpoint(
                &relation.target,
                &relation_config.target,
                "target",
                &relation.predicate,
                &relation.location,
            );
            self.validate_values(
                &relation.properties,
                &relation_config.property,
                &format!("relation {:?}", relation.predicate),
                &relation.location,
            );
            triples
                .entry((
                    relation.source.clone(),
                    &relation.predicate,
                    relation.target.clone(),
                ))
                .or_default()
                .push(index);
        }
        for ((source, predicate, target), indexes) in triples {
            if indexes.len() > 1 {
                for index in indexes {
                    self.error(
                        "duplicate-relation",
                        format!(
                            "explicit relation ({source:?}, {predicate:?}, {target:?}) is repeated"
                        ),
                        (&self.graph.relations[index].location).into(),
                    );
                }
            }
        }
        self.validate_acyclic_relations();
    }

    fn validate_entity_state(&mut self, entity: &crate::EntityNode, workflow_name: Option<&str>) {
        match workflow_name {
            None if entity.state.is_some() => self.error(
                "unexpected-entity-state",
                format!(
                    "entity {:?} has state but its type has no workflow",
                    entity.id
                ),
                (&entity.location).into(),
            ),
            Some(name) => {
                let Some(workflow) = self.config.workflows.get(name) else {
                    return;
                };
                match &entity.state {
                    None => self.error(
                        "missing-entity-state",
                        format!(
                            "entity {:?} requires a state from workflow {name:?}",
                            entity.id
                        ),
                        (&entity.location).into(),
                    ),
                    Some(state) if !workflow.states.contains_key(state) => self.error(
                        "invalid-entity-state",
                        format!(
                            "entity {:?} has unknown workflow state {state:?}",
                            entity.id
                        ),
                        (&entity.location).into(),
                    ),
                    _ => {}
                }
            }
            None => {}
        }
    }

    fn validate_values(
        &mut self,
        values: &BTreeMap<String, Value>,
        schema: &BTreeMap<String, PropertyConfig>,
        owner: &str,
        location: &GraphLocation,
    ) {
        for (name, property) in schema {
            if property.required && !values.contains_key(name) {
                self.error(
                    "missing-required-property",
                    format!("{owner} is missing required property {name:?}"),
                    location.into(),
                );
            }
        }
        for (name, value) in values {
            let Some(property) = schema.get(name) else {
                self.error(
                    "undeclared-property",
                    format!("{owner} has undeclared property {name:?}"),
                    location.into(),
                );
                continue;
            };
            if !value_matches(value, property) {
                self.error(
                    "invalid-property-value",
                    format!("{owner} property {name:?} does not match its declared type or values"),
                    location.into(),
                );
            }
        }
    }

    fn validate_relation_endpoint(
        &mut self,
        node: &GraphNode,
        allowed: &[String],
        side: &str,
        predicate: &str,
        location: &GraphLocation,
    ) {
        if allowed.is_empty() || matches!(node, GraphNode::Unresolved(_)) {
            return;
        }
        let endpoint_type = match node {
            GraphNode::Document(_) => Some("document"),
            GraphNode::Section(_) => Some("section"),
            GraphNode::ExternalUri(_) => Some("external"),
            GraphNode::Entity(id) => self
                .graph
                .entities
                .iter()
                .find(|entity| &entity.id == id)
                .map(|entity| entity.entity_type.as_str()),
            GraphNode::Unresolved(_) => None,
        };
        if endpoint_type.is_none_or(|endpoint| !allowed.iter().any(|item| item == endpoint)) {
            self.error(
                "invalid-relation-endpoint",
                format!("relation {predicate:?} {side} {node:?} violates its endpoint constraint"),
                location.into(),
            );
        }
    }

    fn validate_acyclic_relations(&mut self) {
        for (predicate, relation_config) in &self.config.relations {
            if !relation_config.acyclic {
                continue;
            }
            let relations: Vec<_> = self
                .graph
                .relations
                .iter()
                .filter(|relation| {
                    relation.origin == RelationOrigin::Explicit && relation.predicate == *predicate
                })
                .collect();
            let mut adjacency: HashMap<&GraphNode, Vec<&GraphNode>> = HashMap::new();
            for relation in &relations {
                adjacency
                    .entry(&relation.source)
                    .or_default()
                    .push(&relation.target);
            }
            for relation in relations {
                if reachable(&relation.target, &relation.source, &adjacency) {
                    self.error(
                        "cyclic-relation",
                        format!("relation type {predicate:?} contains a cycle"),
                        (&relation.location).into(),
                    );
                }
            }
        }
    }

    fn config_error(&mut self, code: &'static str, message: String, file: &str, needle: &str) {
        let path = self.repository.config_dir().join(file);
        let span = fs::read_to_string(&path)
            .ok()
            .and_then(|source| locate(&source, needle));
        self.error(code, message, ValidationLocation { path, span });
    }

    fn error(&mut self, code: &'static str, message: String, location: ValidationLocation) {
        self.diagnostics.push(ValidationDiagnostic {
            severity: DiagnosticSeverity::Error,
            code,
            message,
            location,
        });
    }
}

fn constraints_fit(original: &[String], inverse: &[String]) -> bool {
    inverse.is_empty()
        || (!original.is_empty()
            && original
                .iter()
                .all(|item| inverse.iter().any(|allowed| item == allowed)))
}

fn scalar_value_matches(value: &ScalarValue, property_type: PropertyType) -> bool {
    matches!(
        (value, property_type),
        (ScalarValue::String(_), PropertyType::String)
            | (ScalarValue::Integer(_), PropertyType::Integer)
            | (ScalarValue::Float(_), PropertyType::Float)
            | (ScalarValue::Boolean(_), PropertyType::Boolean)
            | (ScalarValue::Datetime(_), PropertyType::Datetime)
    )
}

fn value_matches(value: &Value, property: &PropertyConfig) -> bool {
    let typed = match property.property_type {
        PropertyType::String => value.is_str(),
        PropertyType::Integer => value.is_integer(),
        PropertyType::Float => value.is_float(),
        PropertyType::Boolean => value.is_bool(),
        PropertyType::Datetime => value.is_datetime(),
        PropertyType::Array => value.as_array().is_some_and(|array| {
            property.items.is_some_and(|item_type| {
                array
                    .iter()
                    .all(|item| scalar_type_matches(item, item_type))
            })
        }),
    };
    typed
        && property.values.as_ref().is_none_or(|allowed| {
            allowed
                .iter()
                .any(|candidate| scalar_equals(candidate, value))
        })
}

fn scalar_type_matches(value: &Value, scalar_type: ScalarType) -> bool {
    match scalar_type {
        ScalarType::String => value.is_str(),
        ScalarType::Integer => value.is_integer(),
        ScalarType::Float => value.is_float(),
        ScalarType::Boolean => value.is_bool(),
        ScalarType::Datetime => value.is_datetime(),
    }
}

fn scalar_equals(expected: &ScalarValue, actual: &Value) -> bool {
    match expected {
        ScalarValue::String(value) => actual.as_str().is_some_and(|actual| actual == value),
        ScalarValue::Integer(value) => actual.as_integer() == Some(*value),
        ScalarValue::Float(value) => actual.as_float() == Some(*value),
        ScalarValue::Boolean(value) => actual.as_bool() == Some(*value),
        ScalarValue::Datetime(value) => actual.as_datetime() == Some(value),
    }
}

fn reachable<'a>(
    start: &'a GraphNode,
    goal: &'a GraphNode,
    adjacency: &HashMap<&'a GraphNode, Vec<&'a GraphNode>>,
) -> bool {
    let mut pending = vec![start];
    let mut visited = HashSet::new();
    while let Some(node) = pending.pop() {
        if node == goal {
            return true;
        }
        if visited.insert(node)
            && let Some(next) = adjacency.get(node)
        {
            pending.extend(next.iter().copied());
        }
    }
    false
}

fn locate(source: &str, needle: &str) -> Option<SourceSpan> {
    let quoted = format!("\"{needle}\"");
    let start = source.find(&quoted).or_else(|| source.find(needle))?;
    let end = start + source[start..].find(needle)? + needle.len();
    Some(SourceSpan::from_offsets(source, start..end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CanonicalCorpus, GraphIndex, Repository, RepositoryConfig};
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new(document: &str) -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "docgraph-validation-test-{}-{sequence}",
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
                "[entity.task]\ndescription = \"Task\"\nworkflow = \"task\"\n[entity.task.property.title]\ntype = \"string\"\nrequired = true\n",
            )
            .unwrap();
            fs::write(
                root.join(".docgraph/workflows.toml"),
                "[workflow.task]\ninitial = \"open\"\n[workflow.task.states.open]\ndescription = \"Open\"\ntransitions = [\"done\"]\n[workflow.task.states.done]\ndescription = \"Done\"\n",
            )
            .unwrap();
            fs::write(
                root.join(".docgraph/relations.toml"),
                "[relation.blocks]\ndescription = \"Blocks\"\nsource = [\"task\"]\ntarget = [\"task\"]\nacyclic = true\n",
            )
            .unwrap();
            fs::write(root.join("docs/task.md"), document).unwrap();
            Self(root)
        }

        fn validate(&self) -> ValidationReport {
            let repository = Repository::discover(&self.0).unwrap();
            let config = RepositoryConfig::load(&repository).unwrap();
            let corpus = CanonicalCorpus::load(&repository, &config).unwrap();
            let graph = GraphIndex::build(&corpus, &config);
            Validator::validate(&repository, &config, &graph)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn validates_a_well_formed_graph() {
        let fixture = Fixture::new(
            "+++\nid = \"task:1\"\ntype = \"task\"\nstate = \"open\"\n[properties]\ntitle = \"Ship\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# Task\n",
        );

        assert!(fixture.validate().is_valid());
    }

    #[test]
    fn reports_the_offending_entity_id_local_construct() {
        for (id, expected) in [
            ("task:", "empty local component"),
            ("task:.hidden", "starts with disallowed character '.'"),
            ("task:has space", "contains disallowed character ' '"),
            ("task:has/slash", "contains disallowed character '/'"),
            ("task:has\\slash", "contains disallowed character '\\\\'"),
            ("task:naïve", "contains disallowed character 'ï'"),
        ] {
            let fixture = Fixture::new(&format!(
                "+++\nid = {id:?}\ntype = \"task\"\nstate = \"open\"\n[properties]\ntitle = \"Ship\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# Task\n"
            ));
            let report = fixture.validate();
            let diagnostic = report
                .errors()
                .find(|diagnostic| diagnostic.code == "invalid-entity-id")
                .unwrap();
            assert!(
                diagnostic.message.contains(expected),
                "{} did not contain {expected:?}",
                diagnostic.message
            );
        }

        let valid = Fixture::new(
            "+++\nid = \"task:Alpha-2_beta.v3~draft\"\ntype = \"task\"\nstate = \"open\"\n[properties]\ntitle = \"Ship\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# Task\n",
        );
        assert!(valid.validate().is_valid());
    }

    #[test]
    fn reports_semantic_errors_together() {
        let fixture = Fixture::new(
            "+++\nid = \"wrong:1\"\ntype = \"task\"\nstate = \"bogus\"\n[properties]\ntitel = \"typo\"\n[[relations]]\ntype = \"blocks\"\ntarget = \"task:2\"\n+++\n# Task\n",
        );

        let report = fixture.validate();
        let codes: BTreeSet<_> = report.errors().map(|error| error.code).collect();
        assert!(codes.contains("invalid-entity-id"));
        assert!(codes.contains("invalid-entity-state"));
        assert!(codes.contains("missing-required-property"));
        assert!(codes.contains("undeclared-property"));
        assert!(codes.contains("unresolved-managed-reference"), "{codes:?}");
        assert!(codes.contains("missing-section-id"));
        let missing_id = report
            .errors()
            .find(|error| error.code == "missing-section-id")
            .unwrap();
        assert!(missing_id.message.contains("if the heading should remain"));
        assert!(missing_id.message.contains("docgraph normalize --dry-run"));
    }

    #[test]
    fn detects_cycles_and_duplicate_triples() {
        let fixture = Fixture::new(
            "+++\nid = \"task:1\"\ntype = \"task\"\nstate = \"open\"\n[properties]\ntitle = \"Ship\"\n[[relations]]\ntype = \"blocks\"\ntarget = \"task:1\"\n[[relations]]\ntype = \"blocks\"\ntarget = \"task:1\"\n+++\n<a id=\"s-83JRT4K2P6\"></a>\n# Task\n",
        );

        let report = fixture.validate();
        assert!(
            report
                .errors()
                .any(|error| error.code == "duplicate-relation")
        );
        assert!(report.errors().any(|error| error.code == "cyclic-relation"));
    }
}
