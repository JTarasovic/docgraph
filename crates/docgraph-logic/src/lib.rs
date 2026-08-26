//! Restricted, process-isolated Datalog adapter.
//!
//! The public language is deliberately smaller than Souffle. This crate owns
//! declarations, SQLite input directives, the output relation, and the child
//! process. The engine is distributed as an opaque companion executable.

use docgraph_core::{
    ArgumentMode, GraphIndex, GraphNode, NamedQueryConfig, QueryArgumentConfig, QueryValueType,
    RelationOrigin, RepositoryConfig,
};
use rusqlite::{Connection, params_from_iter};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use toml_edit::Value;

const QUERY_TIMEOUT: Duration = Duration::from_secs(5);
const RUNTIME_OVERRIDE: &str = "DOCGRAPH_LOGIC_RUNTIME";
const RESULT_RELATION: &str = "__docgraph_result";
const VALIDATION_RELATION: &str = "__docgraph_validation";
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
        let mut predicates = BTreeMap::new();
        let mut calls = Vec::new();
        for clause in clauses(&structural)? {
            let (head, body) = clause.split_once(":-").ok_or_else(|| {
                LogicError::InvalidRule("each clause must contain a rule body".to_owned())
            })?;
            let (name, arity) = predicate_call(head.trim())?.ok_or_else(|| {
                LogicError::InvalidRule(
                    "each rule must begin with a named predicate head".to_owned(),
                )
            })?;
            if BUILTINS.iter().any(|(builtin, _)| *builtin == name)
                || matches!(name.as_str(), RESULT_RELATION | VALIDATION_RELATION)
            {
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
            calls.push((name, arity));
            calls.extend(predicate_calls(body)?);
        }
        if !source.trim().is_empty() && predicates.is_empty() {
            return Err(LogicError::InvalidRule(
                "logic.dl contains no rule definitions".to_owned(),
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
    fn souffle_literal(&self) -> String {
        match self {
            Self::String(value)
            | Self::Datetime(value)
            | Self::Entity(value)
            | Self::Section(value) => quote(value),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Boolean(value) => {
                if *value {
                    "1".to_owned()
                } else {
                    "0".to_owned()
                }
            }
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
        let expected: BTreeSet<_> = query
            .arguments
            .iter()
            .filter(|argument| argument.mode == ArgumentMode::Input)
            .map(|argument| argument.name.as_str())
            .collect();
        let actual: BTreeSet<_> = inputs.keys().map(String::as_str).collect();
        if expected != actual {
            return Err(QueryError::InputBinding {
                expected: expected.into_iter().map(str::to_owned).collect(),
                found: actual.into_iter().map(str::to_owned).collect(),
            });
        }
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
        }
        let outputs: Vec<_> = query
            .arguments
            .iter()
            .filter(|argument| argument.mode == ArgumentMode::Output)
            .collect();
        let scratch = Scratch::new()?;
        self.write_sqlite_input(&scratch.database)?;
        fs::write(
            &scratch.program,
            self.program(query, &inputs, &outputs, &scratch.database),
        )
        .map_err(|error| QueryError::Io {
            path: scratch.program.clone(),
            error,
        })?;
        run_souffle(&scratch.program, &scratch.output)?;
        self.decode_output(name, &scratch.database, &outputs)
    }

    pub fn validate(&self) -> Result<(), QueryError> {
        let scratch = Scratch::new()?;
        self.write_sqlite_input(&scratch.database)?;
        fs::write(&scratch.program, self.validation_program(&scratch.database)).map_err(
            |error| QueryError::Io {
                path: scratch.program.clone(),
                error,
            },
        )?;
        run_souffle(&scratch.program, &scratch.output)
    }

    fn program(
        &self,
        query: &NamedQueryConfig,
        inputs: &BTreeMap<String, QueryValue>,
        outputs: &[&QueryArgumentConfig],
        database: &Path,
    ) -> String {
        let mut declarations = builtin_declarations();
        for (name, arity) in &self.logic.predicates {
            let types = self
                .config
                .queries
                .values()
                .find(|candidate| candidate.predicate == *name)
                .map(|candidate| {
                    candidate
                        .arguments
                        .iter()
                        .map(|argument| souffle_type(argument.value_type))
                        .collect()
                })
                .unwrap_or_else(|| vec!["symbol"; *arity]);
            declarations.push(declaration(name, &types));
        }
        let result_types: Vec<_> = outputs
            .iter()
            .map(|argument| souffle_type(argument.value_type))
            .collect();
        declarations.push(declaration(RESULT_RELATION, &result_types));
        let database = sqlite_database_uri(database);
        for (name, _) in BUILTINS {
            declarations.push(format!(
                ".input {name}(IO=sqlite, dbname={})",
                quote(&database)
            ));
        }
        let arguments = query
            .arguments
            .iter()
            .map(|argument| match argument.mode {
                ArgumentMode::Input => inputs[&argument.name].souffle_literal(),
                ArgumentMode::Output => argument.name.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let output_names = outputs
            .iter()
            .map(|argument| argument.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{}\n{}\n{RESULT_RELATION}({output_names}) :- {}({arguments}).\n.output {RESULT_RELATION}(IO=sqlite, dbname={})\n",
            declarations.join("\n"),
            self.logic.source,
            query.predicate,
            quote(&database)
        )
    }

    fn validation_program(&self, database: &Path) -> String {
        let mut declarations = builtin_declarations();
        for (name, arity) in &self.logic.predicates {
            let types = self
                .config
                .queries
                .values()
                .find(|candidate| candidate.predicate == *name)
                .map(|candidate| {
                    candidate
                        .arguments
                        .iter()
                        .map(|argument| souffle_type(argument.value_type))
                        .collect()
                })
                .unwrap_or_else(|| vec!["symbol"; *arity]);
            declarations.push(declaration(name, &types));
        }
        declarations.push(declaration(VALIDATION_RELATION, &["symbol"]));
        let database = sqlite_database_uri(database);
        for (name, _) in BUILTINS {
            declarations.push(format!(
                ".input {name}(IO=sqlite, dbname={})",
                quote(&database)
            ));
        }
        format!(
            "{}\n{}\n{VALIDATION_RELATION}(\"ok\").\n.output {VALIDATION_RELATION}(IO=sqlite, dbname={})\n",
            declarations.join("\n"),
            self.logic.source,
            quote(&database)
        )
    }

    fn write_sqlite_input(&self, path: &Path) -> Result<(), QueryError> {
        let connection =
            Connection::open(path).map_err(|error| QueryError::Execution(error.to_string()))?;
        let facts = self.builtin_facts();
        for (name, arity) in BUILTINS {
            let columns = (0..*arity)
                .map(|index| format!("c{index} TEXT NOT NULL"))
                .collect::<Vec<_>>()
                .join(", ");
            connection
                .execute_batch(&format!(
                    "CREATE TABLE _{name} ({columns}); CREATE VIEW {name} AS SELECT * FROM _{name};"
                ))
                .map_err(|error| QueryError::Execution(error.to_string()))?;
            let placeholders = (1..=*arity)
                .map(|index| format!("?{index}"))
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!("INSERT INTO _{name} VALUES ({placeholders})");
            for row in &facts[name] {
                connection
                    .execute(&sql, params_from_iter(row))
                    .map_err(|error| QueryError::Execution(error.to_string()))?;
            }
        }
        Ok(())
    }

    fn decode_output(
        &self,
        query: &str,
        path: &Path,
        outputs: &[&QueryArgumentConfig],
    ) -> Result<QueryResult, QueryError> {
        let connection =
            Connection::open(path).map_err(|error| QueryError::Execution(error.to_string()))?;
        let mut statement = connection
            .prepare(&format!("SELECT * FROM {RESULT_RELATION}"))
            .map_err(|error| QueryError::Execution(error.to_string()))?;
        if statement.column_count() != outputs.len() {
            return Err(QueryError::OutputArity {
                expected: outputs.len(),
                found: statement.column_count(),
            });
        }
        let mut rows = Vec::new();
        let mut sqlite_rows = statement
            .query([])
            .map_err(|error| QueryError::Execution(error.to_string()))?;
        while let Some(sqlite_row) = sqlite_rows
            .next()
            .map_err(|error| QueryError::Execution(error.to_string()))?
        {
            let mut row = BTreeMap::new();
            for (index, argument) in outputs.iter().enumerate() {
                let value = self
                    .output_value(sqlite_row, index, argument.value_type)
                    .ok_or_else(|| QueryError::OutputType {
                        argument: argument.name.clone(),
                        expected: argument.value_type,
                    })?;
                row.insert(argument.name.clone(), value);
            }
            rows.push(row);
        }
        Ok(QueryResult {
            query: query.to_owned(),
            columns: outputs
                .iter()
                .map(|argument| QueryColumn {
                    name: argument.name.clone(),
                    value_type: argument.value_type,
                })
                .collect(),
            rows,
        })
    }

    fn reference_value_exists(&self, value: &QueryValue) -> bool {
        match value {
            QueryValue::Entity(id) => self.graph.entities.iter().any(|entity| entity.id == *id),
            QueryValue::Section(id) => section_exists(self.graph, id),
            QueryValue::Datetime(value) => datetime_is_valid(value),
            _ => true,
        }
    }
    fn output_value(
        &self,
        row: &rusqlite::Row<'_>,
        index: usize,
        value_type: QueryValueType,
    ) -> Option<QueryValue> {
        match value_type {
            QueryValueType::String => row.get(index).ok().map(QueryValue::String),
            QueryValueType::Integer => row.get(index).ok().map(QueryValue::Integer),
            QueryValueType::Float => row.get(index).ok().map(QueryValue::Float),
            QueryValueType::Boolean => match row.get::<_, i64>(index).ok()? {
                0 => Some(QueryValue::Boolean(false)),
                1 => Some(QueryValue::Boolean(true)),
                _ => None,
            },
            QueryValueType::Datetime => row
                .get::<_, String>(index)
                .ok()
                .filter(|value| datetime_is_valid(value))
                .map(QueryValue::Datetime),
            QueryValueType::Entity => row.get::<_, String>(index).ok().and_then(|value| {
                self.graph
                    .entities
                    .iter()
                    .any(|entity| entity.id == value)
                    .then_some(QueryValue::Entity(value))
            }),
            QueryValueType::Section => {
                let value = row.get::<_, String>(index).ok()?;
                section_exists(self.graph, &value).then_some(QueryValue::Section(value))
            }
        }
    }
    fn builtin_facts(&self) -> HashMap<&'static str, Vec<Vec<String>>> {
        let mut facts: HashMap<&str, Vec<Vec<String>>> = BUILTINS
            .iter()
            .map(|(name, _)| (*name, Vec::new()))
            .collect();
        for (index, document) in self.graph.documents.iter().enumerate() {
            facts
                .get_mut("document")
                .unwrap()
                .push(vec![document.path.to_string_lossy().into_owned()]);
            if let Some(id) = &document.entity {
                facts.get_mut("entity").unwrap().push(vec![id.clone()]);
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
                    .push(vec![entity.id.clone(), entity.entity_type.clone()]);
                if let Some(state) = &entity.state {
                    facts
                        .get_mut("entity_state")
                        .unwrap()
                        .push(vec![entity.id.clone(), state.clone()]);
                }
            }
        }
        for (index, section) in self.graph.sections.iter().enumerate() {
            if section.id.is_some() {
                let id = node_identity(self.graph, &GraphNode::Section(index))
                    .expect("identified section has identity");
                facts.get_mut("section").unwrap().push(vec![
                    id,
                    self.graph.documents[section.document]
                        .path
                        .to_string_lossy()
                        .into_owned(),
                    section.heading.clone(),
                ]);
            }
        }
        for relation in self
            .graph
            .relations
            .iter()
            .filter(|relation| relation.origin == RelationOrigin::Explicit)
        {
            let (Some(source), Some(target)) = (
                node_identity(self.graph, &relation.source),
                node_identity(self.graph, &relation.target),
            ) else {
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
        facts
    }
}

struct Scratch {
    directory: PathBuf,
    database: PathBuf,
    program: PathBuf,
    output: PathBuf,
}
impl Scratch {
    fn new() -> Result<Self, QueryError> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("docgraph-souffle-{}-{nonce}", std::process::id()));
        fs::create_dir_all(directory.join("output")).map_err(|error| QueryError::Io {
            path: directory.clone(),
            error,
        })?;
        Ok(Self {
            database: directory.join("facts.sqlite"),
            program: directory.join("query.dl"),
            output: directory.join("output"),
            directory,
        })
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn run_souffle(program: &Path, output: &Path) -> Result<(), QueryError> {
    let executable = runtime_executable()?;
    let mut child = Command::new(executable)
        .arg("--no-preprocessor")
        .arg("-D")
        .arg(output)
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| QueryError::Execution(error.to_string()))?;
    let started = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| QueryError::Execution(error.to_string()))?
        {
            if status.success() {
                return Ok(());
            }
            let stderr = child
                .stderr
                .take()
                .map(|mut stderr| {
                    use std::io::Read;
                    let mut text = String::new();
                    let _ = stderr.read_to_string(&mut text);
                    text
                })
                .unwrap_or_default();
            return Err(QueryError::Execution(stderr));
        }
        if started.elapsed() >= QUERY_TIMEOUT {
            child
                .kill()
                .map_err(|error| QueryError::Execution(error.to_string()))?;
            let _ = child.wait();
            return Err(QueryError::Timeout);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn runtime_executable() -> Result<PathBuf, QueryError> {
    if let Some(path) = std::env::var_os(RUNTIME_OVERRIDE) {
        if path.is_empty() {
            return Err(QueryError::RuntimeUnavailable);
        }
        return Ok(PathBuf::from(path));
    }
    let current = std::env::current_exe().map_err(|error| QueryError::Io {
        path: PathBuf::from("current executable"),
        error,
    })?;
    let name = if cfg!(windows) {
        "docgraph-logic-runtime.exe"
    } else {
        "docgraph-logic-runtime"
    };
    let companion = current
        .parent()
        .ok_or(QueryError::RuntimeUnavailable)?
        .join(name);
    companion
        .is_file()
        .then_some(companion)
        .ok_or(QueryError::RuntimeUnavailable)
}

fn sqlite_database_uri(path: &Path) -> String {
    if cfg!(windows) {
        // Souffle treats a drive-letter path as relative because it only recognises
        // slash-prefixed paths. Its SQLite connector accepts file URIs on Windows.
        format!("file:///{}", path.to_string_lossy().replace('\\', "/"))
    } else {
        path.to_string_lossy().into_owned()
    }
}

fn builtin_declarations() -> Vec<String> {
    BUILTINS
        .iter()
        .map(|(name, arity)| declaration(name, &vec!["symbol"; *arity]))
        .collect()
}
fn declaration(name: &str, types: &[&str]) -> String {
    format!(
        ".decl {name}({})",
        types
            .iter()
            .enumerate()
            .map(|(index, kind)| format!("v{index}:{kind}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
fn souffle_type(value_type: QueryValueType) -> &'static str {
    match value_type {
        QueryValueType::Integer | QueryValueType::Boolean => "number",
        QueryValueType::Float => "float",
        QueryValueType::String
        | QueryValueType::Datetime
        | QueryValueType::Entity
        | QueryValueType::Section => "symbol",
    }
}
fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
fn builtin_arity(name: &str) -> Option<usize> {
    BUILTINS
        .iter()
        .find_map(|(builtin, arity)| (*builtin == name).then_some(*arity))
}
fn section_exists(graph: &GraphIndex, reference: &str) -> bool {
    graph.sections.iter().enumerate().any(|(index, section)| {
        section
            .id
            .as_ref()
            .is_some_and(|id| id.as_str() == reference)
            || node_identity(graph, &GraphNode::Section(index)).as_deref() == Some(reference)
    })
}
fn datetime_is_valid(value: &str) -> bool {
    format!("value = {value}")
        .parse::<toml_edit::DocumentMut>()
        .ok()
        .and_then(|document| document["value"].as_datetime().cloned())
        .is_some()
}
fn add_relation_facts(
    facts: &mut HashMap<&str, Vec<Vec<String>>>,
    source: &str,
    predicate: &str,
    target: &str,
    properties: &BTreeMap<String, Value>,
) {
    facts.get_mut("relation").unwrap().push(vec![
        source.to_owned(),
        predicate.to_owned(),
        target.to_owned(),
    ]);
    for (key, value) in properties {
        facts.get_mut("relation_property").unwrap().push(vec![
            source.to_owned(),
            predicate.to_owned(),
            target.to_owned(),
            key.clone(),
            property_value(value),
        ]);
    }
}
fn property_value(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .or_else(|| value.as_integer().map(|value| value.to_string()))
        .or_else(|| value.as_float().map(|value| value.to_string()))
        .or_else(|| value.as_bool().map(|value| value.to_string()))
        .or_else(|| value.as_datetime().map(|value| value.to_string()))
        .unwrap_or_else(|| value.to_string())
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

fn mask_strings_and_comments(source: &str) -> Result<String, LogicError> {
    let mut output = String::with_capacity(source.len());
    let mut quote = false;
    let mut escaped = false;
    let mut comment = false;
    let mut characters = source.chars().peekable();
    while let Some(character) = characters.next() {
        if comment {
            if character == '\n' {
                comment = false;
                output.push(character);
            } else {
                output.push(' ');
            }
            continue;
        }
        if quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quote = false;
            }
            output.push(if character == '\n' { '\n' } else { ' ' });
            continue;
        }
        if character == '/' && characters.peek() == Some(&'/') {
            comment = true;
            output.push_str("  ");
            let _ = characters.next();
        } else if character == '"' {
            quote = true;
            output.push(' ');
        } else {
            output.push(character);
        }
    }
    if quote {
        return Err(LogicError::InvalidRule(
            "unterminated string literal".to_owned(),
        ));
    }
    Ok(output)
}
fn reject_unsupported(source: &str) -> Result<(), LogicError> {
    let forbidden = [
        "#",
        ".decl",
        ".input",
        ".output",
        ".include",
        ".component",
        ".init",
        ".type",
        ".pragma",
        "functor",
        "@",
        "[",
        "]",
        ";",
        "{",
    ];
    if let Some(token) = forbidden.into_iter().find(|token| source.contains(token)) {
        return Err(LogicError::UnsupportedConstruct(token.to_owned()));
    }
    if source
        .lines()
        .any(|line| line.trim_start().starts_with('.'))
    {
        return Err(LogicError::UnsupportedConstruct("directive".to_owned()));
    }
    Ok(())
}
fn clauses(source: &str) -> Result<Vec<&str>, LogicError> {
    let clauses = source
        .split('.')
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .collect::<Vec<_>>();
    if source.trim().is_empty() {
        Ok(clauses)
    } else if !source.trim_end().ends_with('.') {
        Err(LogicError::InvalidRule(
            "each rule must end with a period".to_owned(),
        ))
    } else {
        Ok(clauses)
    }
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
            let mut parenthesis = index;
            while parenthesis < bytes.len() && bytes[parenthesis].is_ascii_whitespace() {
                parenthesis += 1;
            }
            if parenthesis < bytes.len() && bytes[parenthesis] == b'(' {
                let end = matching_parenthesis(source, parenthesis)?;
                calls.push((name.to_owned(), arity(&source[parenthesis + 1..end])));
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
    Ok((calls.len() == 1).then(|| calls.pop().expect("one call")))
}
fn matching_parenthesis(source: &str, start: usize) -> Result<usize, LogicError> {
    let mut depth = 0;
    for (offset, byte) in source.as_bytes()[start..].iter().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => {
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
            Self::UnsupportedConstruct(token) => write!(
                formatter,
                "repository logic uses unsupported construct {token:?}"
            ),
            Self::ReservedPredicate(name) => write!(
                formatter,
                "repository logic cannot define reserved predicate {name:?}"
            ),
            Self::UnknownPredicate(name) => write!(formatter, "unknown predicate {name:?}"),
            Self::ArityMismatch {
                predicate,
                expected,
                found,
            } => write!(
                formatter,
                "predicate {predicate:?} has arity {found}, expected {expected}"
            ),
            Self::QueryPredicateMissing { query, predicate } => write!(
                formatter,
                "query {query:?} references unknown predicate {predicate:?}"
            ),
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
    RuntimeUnavailable,
    Timeout,
    Execution(String),
    Io {
        path: PathBuf,
        error: std::io::Error,
    },
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
            Self::InputBinding { expected, found } => write!(
                formatter,
                "query inputs must be {expected:?}, found {found:?}"
            ),
            Self::InputType { argument, expected } => {
                write!(formatter, "query input {argument:?} must be {expected:?}")
            }
            Self::RuntimeUnavailable => write!(
                formatter,
                "docgraph logic runtime is unavailable; install the companion executable or set {RUNTIME_OVERRIDE}"
            ),
            Self::Timeout => write!(
                formatter,
                "repository query exceeded the five-second limit and was killed"
            ),
            Self::Execution(error) => write!(formatter, "repository query failed: {error}"),
            Self::Io { path, error } => {
                write!(formatter, "cannot access {}: {error}", path.display())
            }
            Self::OutputArity { expected, found } => write!(
                formatter,
                "query returned {found} columns, expected {expected}"
            ),
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
            Self::Io { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_recursion_negation_and_comparisons_but_not_engine_escape_hatches() {
        let module = LogicModule::parse("reachable(x, y) :- relation(x, \"links\", y).\nreachable(x, z) :- relation(x, \"links\", y), reachable(y, z).\nblocked(x) :- entity_state(x, \"blocked\").\nactionable(x) :- entity(x), !blocked(x), x != \"archived\".\n").unwrap();
        assert_eq!(module.predicate_arity("reachable"), Some(2));
        for source in [
            ".output entity",
            "#include \"other.dl\"",
            ".component X {}",
            "p(x) :- @fun(x).",
        ] {
            assert!(
                matches!(
                    LogicModule::parse(source),
                    Err(LogicError::UnsupportedConstruct(_))
                ),
                "{source}"
            );
        }
    }
    #[test]
    fn requires_period_terminated_souffle_rules() {
        assert!(matches!(
            LogicModule::parse("p(x) :- entity(x)"),
            Err(LogicError::InvalidRule(_))
        ));
    }
    #[test]
    fn unavailable_runtime_has_an_explicit_cross_platform_error() {
        assert!(
            QueryError::RuntimeUnavailable
                .to_string()
                .contains(RUNTIME_OVERRIDE)
        );
    }
    #[test]
    fn sqlite_input_uses_a_windows_file_uri() {
        let path = Path::new(r"C:\tmp\docgraph\facts.sqlite");
        let location = sqlite_database_uri(path);
        if cfg!(windows) {
            assert_eq!(location, "file:///C:/tmp/docgraph/facts.sqlite");
        } else {
            assert_eq!(location, path.to_string_lossy());
        }
    }

    #[test]
    fn packaged_runtime_name_is_engine_opaque() {
        let name = if cfg!(windows) {
            "docgraph-logic-runtime.exe"
        } else {
            "docgraph-logic-runtime"
        };
        assert!(!name.to_ascii_lowercase().contains("souffle"));
    }
}
