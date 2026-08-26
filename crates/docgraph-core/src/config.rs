use crate::{Repository, SCHEMA_VERSION};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

const RESERVED_FILES: &[&str] = &["commands.toml"];

#[derive(Clone, Debug, PartialEq)]
pub struct RepositoryConfig {
    pub project: ProjectConfig,
    pub entities: BTreeMap<String, EntityTypeConfig>,
    pub relations: BTreeMap<String, RelationTypeConfig>,
    pub workflows: BTreeMap<String, WorkflowConfig>,
    pub queries: BTreeMap<String, NamedQueryConfig>,
    pub logic: Option<String>,
}

impl RepositoryConfig {
    pub fn load(repository: &Repository) -> Result<Self, ConfigLoadError> {
        for reserved in RESERVED_FILES {
            let path = repository.config_dir().join(reserved);
            if path.exists() {
                return Err(ConfigLoadError::UnsupportedFile { path });
            }
        }

        let project_path = repository.project_file();
        let project_file: ProjectFile = read_required(&project_path)?;
        if project_file.schema_version != SCHEMA_VERSION {
            return Err(ConfigLoadError::UnsupportedSchemaVersion {
                path: project_path,
                found: project_file.schema_version,
                supported: SCHEMA_VERSION,
            });
        }

        let entities: EntityFile = read_optional(&repository.config_dir().join("entities.toml"))?;
        let relations: RelationFile =
            read_optional(&repository.config_dir().join("relations.toml"))?;
        let workflows: WorkflowFile =
            read_optional(&repository.config_dir().join("workflows.toml"))?;
        let logic_path = repository.config_dir().join("logic.cozo");
        let logic = read_optional_text(&logic_path)?;

        Ok(Self {
            project: ProjectConfig {
                name: project_file.project.name,
                documents: project_file.documents,
                frontmatter: project_file.frontmatter,
                agent_instructions: project_file.agent_instructions,
                validation: project_file.validation,
            },
            entities: entities.entity,
            relations: relations.relation,
            workflows: workflows.workflow,
            queries: project_file.query,
            logic,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectConfig {
    pub name: String,
    pub documents: DocumentsConfig,
    pub frontmatter: FrontmatterConfig,
    pub agent_instructions: AgentInstructionsConfig,
    pub validation: ValidationConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DocumentsConfig {
    pub root: PathBuf,
    #[serde(default = "default_document_includes")]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn default_document_includes() -> Vec<String> {
    vec!["**/*.md".to_owned()]
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct FrontmatterConfig {
    pub id: String,
    pub entity_type: String,
    pub state: String,
    pub relations: String,
    pub properties: String,
}

impl Default for FrontmatterConfig {
    fn default() -> Self {
        Self {
            id: "id".to_owned(),
            entity_type: "type".to_owned(),
            state: "state".to_owned(),
            relations: "relations".to_owned(),
            properties: "properties".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct AgentInstructionsConfig {
    pub targets: Vec<PathBuf>,
}

impl Default for AgentInstructionsConfig {
    fn default() -> Self {
        Self {
            targets: vec![PathBuf::from("AGENTS.md"), PathBuf::from("CLAUDE.md")],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    #[default]
    Warning,
    Error,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ValidationConfig {
    pub broken_internal_links: DiagnosticSeverity,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EntityTypeConfig {
    pub description: String,
    #[serde(default)]
    pub workflow: Option<String>,
    #[serde(default)]
    pub property: BTreeMap<String, PropertyConfig>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RelationTypeConfig {
    pub description: String,
    #[serde(default)]
    pub source: Vec<String>,
    #[serde(default)]
    pub target: Vec<String>,
    #[serde(default)]
    pub inverse: Option<String>,
    #[serde(default)]
    pub acyclic: bool,
    #[serde(default)]
    pub property: BTreeMap<String, PropertyConfig>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PropertyConfig {
    #[serde(rename = "type")]
    pub property_type: PropertyType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub values: Option<Vec<ScalarValue>>,
    #[serde(default)]
    pub items: Option<ScalarType>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PropertyType {
    String,
    Integer,
    Float,
    Boolean,
    Datetime,
    Array,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScalarType {
    String,
    Integer,
    Float,
    Boolean,
    Datetime,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ScalarValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Datetime(toml_edit::Datetime),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowConfig {
    pub initial: String,
    pub states: BTreeMap<String, StateConfig>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StateConfig {
    pub description: String,
    #[serde(default)]
    pub transitions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NamedQueryConfig {
    pub description: String,
    pub predicate: String,
    pub arguments: Vec<QueryArgumentConfig>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct QueryArgumentConfig {
    pub name: String,
    pub mode: ArgumentMode,
    #[serde(rename = "type")]
    pub value_type: QueryValueType,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArgumentMode {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QueryValueType {
    String,
    Integer,
    Float,
    Boolean,
    Datetime,
    Entity,
    Section,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectFile {
    schema_version: u32,
    project: ProjectTable,
    documents: DocumentsConfig,
    #[serde(default)]
    frontmatter: FrontmatterConfig,
    #[serde(default)]
    agent_instructions: AgentInstructionsConfig,
    #[serde(default)]
    validation: ValidationConfig,
    #[serde(default)]
    query: BTreeMap<String, NamedQueryConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectTable {
    name: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntityFile {
    #[serde(default)]
    entity: BTreeMap<String, EntityTypeConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RelationFile {
    #[serde(default)]
    relation: BTreeMap<String, RelationTypeConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowFile {
    #[serde(default)]
    workflow: BTreeMap<String, WorkflowConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceSpan {
    pub bytes: Range<usize>,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigDiagnostic {
    pub path: PathBuf,
    pub span: Option<SourceSpan>,
    pub message: String,
}

#[derive(Debug)]
pub enum ConfigLoadError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Parse(ConfigDiagnostic),
    UnsupportedSchemaVersion {
        path: PathBuf,
        found: u32,
        supported: u32,
    },
    UnsupportedFile {
        path: PathBuf,
    },
}

impl fmt::Display for ConfigLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::Parse(diagnostic) => {
                write!(formatter, "{}", diagnostic.path.display())?;
                if let Some(span) = &diagnostic.span {
                    write!(formatter, ":{}:{}", span.line, span.column)?;
                }
                write!(formatter, ": {}", diagnostic.message)
            }
            Self::UnsupportedSchemaVersion {
                path,
                found,
                supported,
            } => write!(
                formatter,
                "{}: unsupported schema_version {found}; this binary supports {supported}",
                path.display()
            ),
            Self::UnsupportedFile { path } => write!(
                formatter,
                "{} is reserved for a post-v0 extension and is not supported",
                path.display()
            ),
        }
    }
}

impl Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn read_required<T>(path: &Path) -> Result<T, ConfigLoadError>
where
    T: for<'de> Deserialize<'de>,
{
    let source = fs::read_to_string(path).map_err(|source| ConfigLoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse(path, &source)
}

fn read_optional<T>(path: &Path) -> Result<T, ConfigLoadError>
where
    T: for<'de> Deserialize<'de> + Default,
{
    match fs::read_to_string(path) {
        Ok(source) => parse(path, &source),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(T::default()),
        Err(source) => Err(ConfigLoadError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn read_optional_text(path: &Path) -> Result<Option<String>, ConfigLoadError> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ConfigLoadError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn parse<T>(path: &Path, source: &str) -> Result<T, ConfigLoadError>
where
    T: for<'de> Deserialize<'de>,
{
    toml_edit::de::from_str(source).map_err(|error| {
        let span = error.span().map(|bytes| source_span(source, bytes));
        ConfigLoadError::Parse(ConfigDiagnostic {
            path: path.to_path_buf(),
            span,
            message: error.message().to_owned(),
        })
    })
}

fn source_span(source: &str, bytes: Range<usize>) -> SourceSpan {
    let prefix = &source[..bytes.start.min(source.len())];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.chars().count() + 1, |(_, tail)| {
            tail.chars().count() + 1
        });
    SourceSpan {
        bytes,
        line,
        column,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "docgraph-config-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(root.join(".docgraph")).unwrap();
            Self { root }
        }

        fn write(&self, relative: &str, contents: &str) {
            fs::write(self.root.join(".docgraph").join(relative), contents).unwrap();
        }

        fn load(&self) -> Result<RepositoryConfig, ConfigLoadError> {
            let repository = Repository::discover(&self.root).unwrap();
            RepositoryConfig::load(&repository)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn loads_the_complete_typed_configuration() {
        let fixture = Fixture::new();
        fixture.write(
            "project.toml",
            r#"schema_version = 1

[project]
name = "example"

[documents]
root = "docs"

[validation]
broken_internal_links = "error"

[query.task_blockers]
description = "Find blockers"
predicate = "task_blockers"
arguments = [
  { name = "task", mode = "input", type = "entity" },
  { name = "blocker", mode = "output", type = "entity" },
]
"#,
        );
        fixture.write(
            "entities.toml",
            r#"[entity.task]
description = "A unit of work"
workflow = "task"

[entity.task.property.priority]
type = "string"
values = ["normal", "high"]
"#,
        );
        fixture.write(
            "relations.toml",
            r#"[relation.blocked_by]
description = "Target blocks source"
source = ["task"]
target = ["task"]
acyclic = true
"#,
        );
        fixture.write(
            "workflows.toml",
            r#"[workflow.task]
initial = "open"

[workflow.task.states.open]
description = "Work remains"
transitions = ["done"]

[workflow.task.states.done]
description = "Work is complete"
"#,
        );
        fixture.write(
            "logic.cozo",
            "actionable[task] := entity_state[task, 'open']\n",
        );

        let config = fixture.load().unwrap();

        assert_eq!(config.project.name, "example");
        assert_eq!(config.project.documents.root, PathBuf::from("docs"));
        assert_eq!(config.project.documents.include, ["**/*.md"]);
        assert_eq!(
            config.project.validation.broken_internal_links,
            DiagnosticSeverity::Error
        );
        assert!(config.entities.contains_key("task"));
        assert!(config.relations["blocked_by"].acyclic);
        assert_eq!(config.workflows["task"].initial, "open");
        assert_eq!(config.queries["task_blockers"].arguments.len(), 2);
        assert!(config.logic.as_deref().unwrap().contains("actionable"));
        assert_eq!(
            config.project.agent_instructions.targets,
            [PathBuf::from("AGENTS.md"), PathBuf::from("CLAUDE.md")]
        );
    }

    #[test]
    fn reports_a_typed_parse_error_with_a_source_location() {
        let fixture = Fixture::new();
        fixture.write(
            "project.toml",
            r#"schema_version = 1
[project]
name = "example"
[documents]
root = "docs"
include = "not-an-array"
"#,
        );

        let ConfigLoadError::Parse(diagnostic) = fixture.load().unwrap_err() else {
            panic!("expected a parse diagnostic");
        };

        let span = diagnostic.span.expect("diagnostic should have a span");
        assert_eq!(span.line, 6);
        assert!(diagnostic.path.ends_with("project.toml"));
    }

    #[test]
    fn rejects_unknown_fields_instead_of_ignoring_typos() {
        let fixture = Fixture::new();
        fixture.write(
            "project.toml",
            r#"schema_version = 1
[project]
name = "example"
[documents]
root = "docs"
excludes = ["generated/**"]
"#,
        );

        assert!(matches!(fixture.load(), Err(ConfigLoadError::Parse(_))));
    }

    #[test]
    fn rejects_reserved_post_v0_files() {
        let fixture = Fixture::new();
        fixture.write(
            "project.toml",
            "schema_version = 1\n[project]\nname = \"example\"\n[documents]\nroot = \"docs\"\n",
        );
        fixture.write("commands.toml", "");

        assert!(matches!(
            fixture.load(),
            Err(ConfigLoadError::UnsupportedFile { path }) if path.ends_with("commands.toml")
        ));
    }
}
