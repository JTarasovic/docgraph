//! Restricted, read-only Cozo adapter for docgraph repository logic.

use cozo::{DataValue, DbInstance, ScriptMutability};
use docgraph_core::{
    ArgumentMode, GraphIndex, GraphNode, NamedQueryConfig, QueryValueType, RelationOrigin,
    RepositoryConfig,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fmt;
use toml_edit::Value;

const QUERY_TIMEOUT_SECONDS: u8 = 5;
const BUILTINS: &[(&str, usize)] = &[
    ("entity", 1),
    ("entity_type", 2),
    ("entity_state", 2),
    ("relation", 3),
    ("relation_property", 5),
    ("section", 3),
    ("document", 1),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicModule {
    source: String,
    predicates: BTreeMap<String, usize>,
}

impl LogicModule {
    pub fn parse(source: &str) -> Result<Self, LogicError> {
        let structural = mask_strings_and_comments(source)?;
        reject_unsupported(&structural)?;
        let calls = predicate_calls(&structural)?;
        let mut predicates = BTreeMap::new();
        for line in structural.lines() {
            let Some((head, _)) = line.split_once(":=") else {
                continue;
            };
            let Some((name, arity)) = predicate_call(head.trim())? else {
                return Err(LogicError::InvalidRule(
                    "each rule must begin with a named predicate head".to_owned(),
                ));
            };
            if BUILTINS.iter().any(|(builtin, _)| *builtin == name) {
                return Err(LogicError::ReservedPredicate(name));
            }
            if let Some(previous) = predicates.insert(name.clone(), arity)
                && previous != arity
            {
                return Err(LogicError::ArityMismatch {
                    predicate: name,
                    expected: previous,
                    found: arity,
                });
            }
        }
        if source.lines().any(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with('#')
        }) && predicates.is_empty()
        {
            return Err(LogicError::InvalidRule(
                "logic.cozo contains no inline rule definitions".to_owned(),
            ));
        }
        for (name, arity) in calls {
            let expected = predicates
                .get(&name)
                .copied()
                .or_else(|| builtin_arity(&name))
                .ok_or_else(|| LogicError::UnknownPredicate(name.clone()))?;
            if expected != arity {
                return Err(LogicError::ArityMismatch {
                    predicate: name,
                    expected,
                    found: arity,
                });
            }
        }
        Ok(Self {
            source: source.to_owned(),
            predicates,
        })
    }

    pub fn predicate_arity(&self, name: &str) -> Option<usize> {
        self.predicates
            .get(name)
            .copied()
            .or_else(|| builtin_arity(name))
    }

    pub fn validate_queries(
        &self,
        queries: &BTreeMap<String, NamedQueryConfig>,
    ) -> Result<(), LogicError> {
        for (name, query) in queries {
            let arity = self.predicate_arity(&query.predicate).ok_or_else(|| {
                LogicError::QueryPredicateMissing {
                    query: name.clone(),
                    predicate: query.predicate.clone(),
                }
            })?;
            if arity != query.arguments.len() {
                return Err(LogicError::QueryArityMismatch {
                    query: name.clone(),
                    predicate: query.predicate.clone(),
                    expected: arity,
                    found: query.arguments.len(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum QueryValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Datetime(String),
    Entity(String),
    Section(String),
}

impl QueryValue {
    fn value_type(&self) -> QueryValueType {
        match self {
            Self::String(_) => QueryValueType::String,
            Self::Integer(_) => QueryValueType::Integer,
            Self::Float(_) => QueryValueType::Float,
            Self::Boolean(_) => QueryValueType::Boolean,
            Self::Datetime(_) => QueryValueType::Datetime,
            Self::Entity(_) => QueryValueType::Entity,
            Self::Section(_) => QueryValueType::Section,
        }
    }

    fn into_data_value(self) -> DataValue {
        match self {
            Self::String(value)
            | Self::Datetime(value)
            | Self::Entity(value)
            | Self::Section(value) => value.into(),
            Self::Integer(value) => value.into(),
            Self::Float(value) => value.into(),
            Self::Boolean(value) => value.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryColumn {
    pub name: String,
    pub value_type: QueryValueType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueryResult {
    pub query: String,
    pub columns: Vec<QueryColumn>,
    pub rows: Vec<BTreeMap<String, QueryValue>>,
}

pub struct QueryEngine<'a> {
    config: &'a RepositoryConfig,
    graph: &'a GraphIndex,
    logic: LogicModule,
}

impl<'a> QueryEngine<'a> {
    pub fn new(config: &'a RepositoryConfig, graph: &'a GraphIndex) -> Result<Self, QueryError> {
        let logic = LogicModule::parse(config.logic.as_deref().unwrap_or_default())?;
        logic.validate_queries(&config.queries)?;
        Ok(Self {
            config,
            graph,
            logic,
        })
    }

    pub fn execute(
        &self,
        name: &str,
        inputs: BTreeMap<String, QueryValue>,
    ) -> Result<QueryResult, QueryError> {
        let query = self
            .config
            .queries
            .get(name)
            .ok_or_else(|| QueryError::UnknownQuery(name.to_owned()))?;
        let expected_inputs: BTreeSet<_> = query
            .arguments
            .iter()
            .filter(|argument| argument.mode == ArgumentMode::Input)
            .map(|argument| argument.name.as_str())
            .collect();
        let actual_inputs: BTreeSet<_> = inputs.keys().map(String::as_str).collect();
        if expected_inputs != actual_inputs {
            return Err(QueryError::InputBinding {
                expected: expected_inputs.into_iter().map(str::to_owned).collect(),
                found: actual_inputs.into_iter().map(str::to_owned).collect(),
            });
        }

        let mut params = BTreeMap::new();
        for argument in query
            .arguments
            .iter()
            .filter(|argument| argument.mode == ArgumentMode::Input)
        {
            let value = inputs
                .get(&argument.name)
                .expect("input names were compared above");
            if value.value_type() != argument.value_type || !self.reference_value_exists(value) {
                return Err(QueryError::InputType {
                    argument: argument.name.clone(),
                    expected: argument.value_type,
                });
            }
            params.insert(argument.name.clone(), value.clone().into_data_value());
        }

        let output_arguments: Vec<_> = query
            .arguments
            .iter()
            .filter(|argument| argument.mode == ArgumentMode::Output)
            .collect();
        let invocation = query
            .arguments
            .iter()
            .map(|argument| match argument.mode {
                ArgumentMode::Input => format!("${}", argument.name),
                ArgumentMode::Output => argument.name.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let outputs = output_arguments
            .iter()
            .map(|argument| argument.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let script = format!(
            "{}\n{}\n?[{}] := {}[{}]\n:timeout {}",
            self.builtin_facts(),
            self.logic.source,
            outputs,
            query.predicate,
            invocation,
            QUERY_TIMEOUT_SECONDS
        );
        let database = DbInstance::default();
        let rows = database
            .run_script(&script, params, ScriptMutability::Immutable)
            .map_err(|error| QueryError::Execution(error.to_string()))?;
        if rows.headers.len() != output_arguments.len() {
            return Err(QueryError::OutputArity {
                expected: output_arguments.len(),
                found: rows.headers.len(),
            });
        }
        let mut result_rows = Vec::with_capacity(rows.rows.len());
        for row in rows.rows {
            if row.len() != output_arguments.len() {
                return Err(QueryError::OutputArity {
                    expected: output_arguments.len(),
                    found: row.len(),
                });
            }
            let mut output = BTreeMap::new();
            for (value, argument) in row.into_iter().zip(&output_arguments) {
                let value = self
                    .output_value(value, argument.value_type)
                    .ok_or_else(|| QueryError::OutputType {
                        argument: argument.name.clone(),
                        expected: argument.value_type,
                    })?;
                output.insert(argument.name.clone(), value);
            }
            result_rows.push(output);
        }
        Ok(QueryResult {
            query: name.to_owned(),
            columns: output_arguments
                .iter()
                .map(|argument| QueryColumn {
                    name: argument.name.clone(),
                    value_type: argument.value_type,
                })
                .collect(),
            rows: result_rows,
        })
    }

    fn reference_value_exists(&self, value: &QueryValue) -> bool {
        match value {
            QueryValue::Entity(id) => self.graph.entities.iter().any(|entity| entity.id == *id),
            QueryValue::Section(id) => self.graph.sections.iter().any(|section| {
                section
                    .id
                    .as_ref()
                    .is_some_and(|section_id| section_id.as_str() == id)
            }),
            _ => true,
        }
    }

    fn output_value(&self, value: DataValue, value_type: QueryValueType) -> Option<QueryValue> {
        match value_type {
            QueryValueType::String => value
                .get_str()
                .map(|value| QueryValue::String(value.to_owned())),
            QueryValueType::Integer => value.get_int().map(QueryValue::Integer),
            QueryValueType::Float => value.get_float().map(QueryValue::Float),
            QueryValueType::Boolean => value.get_bool().map(QueryValue::Boolean),
            QueryValueType::Datetime => value
                .get_str()
                .map(|value| QueryValue::Datetime(value.to_owned())),
            QueryValueType::Entity => value.get_str().and_then(|id| {
                self.graph
                    .entities
                    .iter()
                    .any(|entity| entity.id == id)
                    .then(|| QueryValue::Entity(id.to_owned()))
            }),
            QueryValueType::Section => value.get_str().and_then(|id| {
                self.graph
                    .sections
                    .iter()
                    .any(|section| {
                        section
                            .id
                            .as_ref()
                            .is_some_and(|section_id| section_id.as_str() == id)
                    })
                    .then(|| QueryValue::Section(id.to_owned()))
            }),
        }
    }

    fn builtin_facts(&self) -> String {
        let mut facts: HashMap<&str, Vec<Vec<String>>> = BUILTINS
            .iter()
            .map(|(name, _)| (*name, Vec::new()))
            .collect();
        for (index, document) in self.graph.documents.iter().enumerate() {
            facts
                .get_mut("document")
                .unwrap()
                .push(vec![quote(&document.path.to_string_lossy())]);
            if let Some(id) = &document.entity {
                facts.get_mut("entity").unwrap().push(vec![quote(id)]);
            }
            for entity in self
                .graph
                .entities
                .iter()
                .filter(|entity| entity.document == index)
            {
                facts
                    .get_mut("entity_type")
                    .unwrap()
                    .push(vec![quote(&entity.id), quote(&entity.entity_type)]);
                if let Some(state) = &entity.state {
                    facts
                        .get_mut("entity_state")
                        .unwrap()
                        .push(vec![quote(&entity.id), quote(state)]);
                }
            }
        }
        for section in &self.graph.sections {
            if let Some(id) = &section.id {
                facts.get_mut("section").unwrap().push(vec![
                    quote(id.as_str()),
                    quote(
                        &self.graph.documents[section.document]
                            .path
                            .to_string_lossy(),
                    ),
                    quote(&section.heading),
                ]);
            }
        }
        for relation in self
            .graph
            .relations
            .iter()
            .filter(|relation| relation.origin == RelationOrigin::Explicit)
        {
            let Some(source) = node_identity(self.graph, &relation.source) else {
                continue;
            };
            let Some(target) = node_identity(self.graph, &relation.target) else {
                continue;
            };
            add_relation_facts(
                &mut facts,
                &source,
                &relation.predicate,
                &target,
                &relation.properties,
            );
            if let Some(inverse) = self
                .config
                .relations
                .get(&relation.predicate)
                .and_then(|config| config.inverse.as_deref())
            {
                add_relation_facts(&mut facts, &target, inverse, &source, &relation.properties);
            }
        }
        BUILTINS
            .iter()
            .map(|(name, arity)| fact_rule(name, *arity, &facts[name]))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn add_relation_facts(
    facts: &mut HashMap<&str, Vec<Vec<String>>>,
    source: &str,
    predicate: &str,
    target: &str,
    properties: &BTreeMap<String, Value>,
) {
    facts
        .get_mut("relation")
        .unwrap()
        .push(vec![quote(source), quote(predicate), quote(target)]);
    for (key, value) in properties {
        facts.get_mut("relation_property").unwrap().push(vec![
            quote(source),
            quote(predicate),
            quote(target),
            quote(key),
            cozo_value(value),
        ]);
    }
}

fn node_identity(graph: &GraphIndex, node: &GraphNode) -> Option<String> {
    match node {
        GraphNode::Document(index) => Some(
            graph
                .documents
                .get(*index)?
                .path
                .to_string_lossy()
                .into_owned(),
        ),
        GraphNode::Entity(id) | GraphNode::ExternalUri(id) => Some(id.clone()),
        GraphNode::Section(index) => {
            let section = graph.sections.get(*index)?;
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

fn fact_rule(name: &str, arity: usize, rows: &[Vec<String>]) -> String {
    let variables = (0..arity)
        .map(|index| format!("v{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    if rows.is_empty() {
        format!("{name}[{variables}] := v0 = null, v0 != v0")
    } else {
        let rows = rows
            .iter()
            .map(|row| format!("[{}]", row.join(", ")))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{name}[{variables}] <- [{rows}]")
    }
}

fn cozo_value(value: &Value) -> String {
    if let Some(value) = value.as_str() {
        quote(value)
    } else if let Some(value) = value.as_integer() {
        value.to_string()
    } else if let Some(value) = value.as_float() {
        value.to_string()
    } else if let Some(value) = value.as_bool() {
        value.to_string()
    } else if let Some(value) = value.as_datetime() {
        quote(&value.to_string())
    } else if let Some(value) = value.as_array() {
        format!(
            "[{}]",
            value.iter().map(cozo_value).collect::<Vec<_>>().join(", ")
        )
    } else {
        "null".to_owned()
    }
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn builtin_arity(name: &str) -> Option<usize> {
    BUILTINS
        .iter()
        .find_map(|(builtin, arity)| (*builtin == name).then_some(*arity))
}

fn mask_strings_and_comments(source: &str) -> Result<String, LogicError> {
    let mut output = String::with_capacity(source.len());
    let mut quote = None;
    let mut escaped = false;
    let mut comment = false;
    for character in source.chars() {
        if comment {
            if character == '\n' {
                comment = false;
                output.push(character);
            } else {
                output.push(' ');
            }
            continue;
        }
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            output.push(if character == '\n' { '\n' } else { ' ' });
            continue;
        }
        match character {
            '\'' | '"' => {
                quote = Some(character);
                output.push(' ');
            }
            '#' => {
                comment = true;
                output.push(' ');
            }
            _ => output.push(character),
        }
    }
    if quote.is_some() {
        return Err(LogicError::InvalidRule(
            "unterminated string literal".to_owned(),
        ));
    }
    Ok(output)
}

fn reject_unsupported(source: &str) -> Result<(), LogicError> {
    let forbidden = [
        "?[", "<-", "<~", "::", "=>", "$", "{", "}", "(", ")", ";", "*",
    ];
    if let Some(token) = forbidden.into_iter().find(|token| source.contains(token)) {
        return Err(LogicError::UnsupportedConstruct(token.to_owned()));
    }
    let mut has_rule = false;
    for line in source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if line.starts_with(':') || (!has_rule && !line.contains(":=")) {
            return Err(LogicError::InvalidRule(
                "only inline rule clauses are allowed".to_owned(),
            ));
        }
        has_rule |= line.contains(":=");
    }
    Ok(())
}

fn predicate_calls(source: &str) -> Result<Vec<(String, usize)>, LogicError> {
    let bytes = source.as_bytes();
    let mut calls = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            let name = &source[start..index];
            let mut bracket = index;
            while bracket < bytes.len() && bytes[bracket].is_ascii_whitespace() {
                bracket += 1;
            }
            if bracket < bytes.len() && bytes[bracket] == b'[' {
                let end = matching_bracket(source, bracket)?;
                calls.push((name.to_owned(), arity(&source[bracket + 1..end])));
                index = end + 1;
            }
        } else {
            index += 1;
        }
    }
    Ok(calls)
}

fn predicate_call(source: &str) -> Result<Option<(String, usize)>, LogicError> {
    let mut calls = predicate_calls(source)?;
    if calls.len() == 1 {
        Ok(calls.pop())
    } else {
        Ok(None)
    }
}

fn matching_bracket(source: &str, start: usize) -> Result<usize, LogicError> {
    let mut depth = 0;
    for (offset, byte) in source.as_bytes()[start..].iter().enumerate() {
        match byte {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(start + offset);
                }
            }
            _ => {}
        }
    }
    Err(LogicError::InvalidRule(
        "unclosed predicate call".to_owned(),
    ))
}

fn arity(arguments: &str) -> usize {
    if arguments.trim().is_empty() {
        0
    } else {
        arguments.split(',').count()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogicError {
    InvalidRule(String),
    UnsupportedConstruct(String),
    ReservedPredicate(String),
    UnknownPredicate(String),
    ArityMismatch {
        predicate: String,
        expected: usize,
        found: usize,
    },
    QueryPredicateMissing {
        query: String,
        predicate: String,
    },
    QueryArityMismatch {
        query: String,
        predicate: String,
        expected: usize,
        found: usize,
    },
}

impl fmt::Display for LogicError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRule(reason) => write!(formatter, "invalid repository logic: {reason}"),
            Self::UnsupportedConstruct(token) => {
                write!(
                    formatter,
                    "repository logic uses unsupported construct {token:?}"
                )
            }
            Self::ReservedPredicate(name) => {
                write!(
                    formatter,
                    "repository logic cannot define reserved predicate {name:?}"
                )
            }
            Self::UnknownPredicate(name) => write!(formatter, "unknown predicate {name:?}"),
            Self::ArityMismatch {
                predicate,
                expected,
                found,
            } => write!(
                formatter,
                "predicate {predicate:?} has arity {found}, expected {expected}"
            ),
            Self::QueryPredicateMissing { query, predicate } => {
                write!(
                    formatter,
                    "query {query:?} references unknown predicate {predicate:?}"
                )
            }
            Self::QueryArityMismatch {
                query,
                predicate,
                expected,
                found,
            } => write!(
                formatter,
                "query {query:?} declares {found} arguments but predicate {predicate:?} has arity {expected}"
            ),
        }
    }
}

impl Error for LogicError {}

#[derive(Debug)]
pub enum QueryError {
    Logic(LogicError),
    UnknownQuery(String),
    InputBinding {
        expected: Vec<String>,
        found: Vec<String>,
    },
    InputType {
        argument: String,
        expected: QueryValueType,
    },
    Execution(String),
    OutputArity {
        expected: usize,
        found: usize,
    },
    OutputType {
        argument: String,
        expected: QueryValueType,
    },
}

impl From<LogicError> for QueryError {
    fn from(error: LogicError) -> Self {
        Self::Logic(error)
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Logic(error) => error.fmt(formatter),
            Self::UnknownQuery(name) => write!(formatter, "unknown named query {name:?}"),
            Self::InputBinding { expected, found } => {
                write!(
                    formatter,
                    "query inputs must be {expected:?}, found {found:?}"
                )
            }
            Self::InputType { argument, expected } => {
                write!(formatter, "query input {argument:?} must be {expected:?}")
            }
            Self::Execution(error) => write!(formatter, "repository query failed: {error}"),
            Self::OutputArity { expected, found } => {
                write!(
                    formatter,
                    "query returned {found} columns, expected {expected}"
                )
            }
            Self::OutputType { argument, expected } => {
                write!(formatter, "query output {argument:?} is not {expected:?}")
            }
        }
    }
}

impl Error for QueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Logic(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docgraph_core::{
        AgentInstructionsConfig, DocumentsConfig, EntityNode, EntityTypeConfig, FrontmatterConfig,
        GraphLocation, ProjectConfig, QueryArgumentConfig, RelationTypeConfig, ValidationConfig,
    };
    use std::path::PathBuf;

    #[test]
    fn accepts_only_the_documented_inline_rule_surface() {
        let module = LogicModule::parse(
            "actionable[task] := entity_state[task, 'open']\nblocked[task] := relation[task, 'blocked_by', blocker]\n",
        )
        .unwrap();
        assert_eq!(module.predicate_arity("actionable"), Some(1));
        assert!(matches!(
            LogicModule::parse("entity[id] := id = 'x'"),
            Err(LogicError::ReservedPredicate(_))
        ));
        assert!(matches!(
            LogicModule::parse("?[x] := entity[x]"),
            Err(LogicError::UnsupportedConstruct(_))
        ));
        assert!(matches!(
            LogicModule::parse("counted[count(x)] := entity[x]"),
            Err(LogicError::UnsupportedConstruct(_))
        ));
    }

    #[test]
    fn executes_a_typed_named_query_through_the_read_only_api() {
        let mut config = RepositoryConfig {
            project: ProjectConfig {
                name: "fixture".to_owned(),
                documents: DocumentsConfig {
                    root: "docs".into(),
                    include: vec!["**/*.md".to_owned()],
                    exclude: Vec::new(),
                },
                frontmatter: FrontmatterConfig::default(),
                agent_instructions: AgentInstructionsConfig::default(),
                validation: ValidationConfig::default(),
            },
            entities: BTreeMap::from([(
                "task".to_owned(),
                EntityTypeConfig {
                    description: "Task".to_owned(),
                    workflow: None,
                    property: BTreeMap::new(),
                },
            )]),
            relations: BTreeMap::<String, RelationTypeConfig>::new(),
            workflows: BTreeMap::new(),
            queries: BTreeMap::new(),
            logic: Some("open_task[task] := entity_state[task, 'open']\n".to_owned()),
        };
        config.queries.insert(
            "open_tasks".to_owned(),
            NamedQueryConfig {
                description: "Open tasks".to_owned(),
                predicate: "open_task".to_owned(),
                arguments: vec![QueryArgumentConfig {
                    name: "task".to_owned(),
                    mode: ArgumentMode::Output,
                    value_type: QueryValueType::Entity,
                }],
            },
        );
        let location = GraphLocation {
            path: PathBuf::from("docs/task.md"),
            span: docgraph_markdown::SourceSpan::from_offsets("", 0..0),
        };
        let graph = GraphIndex {
            documents: vec![docgraph_core::DocumentNode {
                path: PathBuf::from("docs/task.md"),
                entity: Some("task:1".to_owned()),
                content_hash: [0; 32],
            }],
            entities: vec![EntityNode {
                id: "task:1".to_owned(),
                entity_type: "task".to_owned(),
                state: Some("open".to_owned()),
                document: 0,
                properties: BTreeMap::new(),
                location,
            }],
            sections: Vec::new(),
            relations: Vec::new(),
            diagnostics: Vec::new(),
        };

        let result = QueryEngine::new(&config, &graph)
            .unwrap()
            .execute("open_tasks", BTreeMap::new())
            .unwrap();

        assert_eq!(result.columns[0].name, "task");
        assert_eq!(
            result.rows[0]["task"],
            QueryValue::Entity("task:1".to_owned())
        );
    }
}
