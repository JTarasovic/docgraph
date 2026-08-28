use crate::{Repository, SCHEMA_VERSION};
use serde::{Deserialize, Deserializer, de};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, PartialEq)]
pub struct RepositoryConfig {
    pub project: ProjectConfig,
    pub entities: BTreeMap<String, EntityTypeConfig>,
    pub relations: BTreeMap<String, RelationTypeConfig>,
    pub workflows: BTreeMap<String, WorkflowConfig>,
    pub queries: BTreeMap<String, NamedQueryConfig>,
    pub commands: BTreeMap<String, CommandConfig>,
    pub logic: Option<String>,
}

impl RepositoryConfig {
    pub fn load(repository: &Repository) -> Result<Self, ConfigLoadError> {
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
        let commands: CommandFile = read_optional(&repository.config_dir().join("commands.toml"))?;
        let logic_path = repository.config_dir().join("logic.dl");
        let logic = read_optional_text(&logic_path)?;
        validate_embedding_config(&project_path, project_file.embeddings.as_ref())?;

        Ok(Self {
            project: ProjectConfig {
                name: project_file.project.name,
                documents: project_file.documents,
                frontmatter: project_file.frontmatter,
                agent_instructions: project_file.agent_instructions,
                validation: project_file.validation,
                references: resolve_git_references(repository, project_file.references)?,
                embeddings: project_file.embeddings,
            },
            entities: entities.entity,
            relations: relations.relation,
            workflows: workflows.workflow,
            queries: project_file.query,
            commands: commands.command,
            logic,
        })
    }
}

fn validate_embedding_config(
    path: &Path,
    config: Option<&EmbeddingConfig>,
) -> Result<(), ConfigLoadError> {
    let Some(config) = config else {
        return Ok(());
    };
    if config.provider.trim().is_empty()
        || config.model.trim().is_empty()
        || config.dimensions == 0
        || config.command.is_empty()
        || config.command[0].trim().is_empty()
        || config.batch_size == 0
        || config.timeout_seconds == 0
    {
        return Err(ConfigLoadError::Invalid {
            path: path.to_path_buf(),
            message: "embeddings requires non-empty provider, model, command, and positive dimensions, batch_size, and timeout_seconds".to_owned(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectConfig {
    pub name: String,
    pub documents: DocumentsConfig,
    pub frontmatter: FrontmatterConfig,
    pub agent_instructions: AgentInstructionsConfig,
    pub validation: ValidationConfig,
    pub references: Vec<GitReferenceConfig>,
    pub embeddings: Option<EmbeddingConfig>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: String,
    pub dimensions: usize,
    pub command: Vec<String>,
    #[serde(default = "default_embedding_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_embedding_timeout_seconds")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub fallback: EmbeddingFallback,
}

fn default_embedding_batch_size() -> usize {
    32
}

fn default_embedding_timeout_seconds() -> u64 {
    30
}

impl EmbeddingConfig {
    pub fn identity(&self) -> String {
        format!("{}:{}:{}", self.provider, self.model, self.dimensions)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingFallback {
    #[default]
    FullText,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitReferenceConfig {
    pub provider: String,
    pub repository: String,
    pub host: String,
    pub remote: String,
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
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandConfig {
    pub description: String,
    pub operation: CommandOperation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandOperation {
    Query {
        query: String,
        entity_type: Option<String>,
    },
    Transition {
        entity_type: String,
        target_state: String,
    },
    AddRelation {
        entity_type: String,
        relation: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCommandConfig {
    description: String,
    operation: RawCommandOperation,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    entity_type: Option<String>,
    #[serde(default)]
    target_state: Option<String>,
    #[serde(default)]
    relation: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawCommandOperation {
    Query,
    Transition,
    AddRelation,
}

impl<'de> Deserialize<'de> for CommandConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawCommandConfig::deserialize(deserializer)?;
        let operation = match raw.operation {
            RawCommandOperation::Query => CommandOperation::Query {
                query: raw.query.ok_or_else(|| de::Error::missing_field("query"))?,
                entity_type: raw.entity_type,
            },
            RawCommandOperation::Transition => CommandOperation::Transition {
                entity_type: raw
                    .entity_type
                    .ok_or_else(|| de::Error::missing_field("entity_type"))?,
                target_state: raw
                    .target_state
                    .ok_or_else(|| de::Error::missing_field("target_state"))?,
            },
            RawCommandOperation::AddRelation => CommandOperation::AddRelation {
                entity_type: raw
                    .entity_type
                    .ok_or_else(|| de::Error::missing_field("entity_type"))?,
                relation: raw
                    .relation
                    .ok_or_else(|| de::Error::missing_field("relation"))?,
            },
        };
        Ok(Self {
            description: raw.description,
            operation,
        })
    }
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
    #[serde(default)]
    references: RawReferencesConfig,
    #[serde(default)]
    embeddings: Option<EmbeddingConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawReferencesConfig {
    git: GitReferenceEntries,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GitReferenceEntries {
    One(RawGitReferenceConfig),
    Many(Vec<RawGitReferenceConfig>),
}

impl Default for GitReferenceEntries {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGitReferenceConfig {
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default = "default_git_remote")]
    remote: String,
    #[serde(default)]
    host: Option<String>,
}

fn default_git_remote() -> String {
    "origin".to_owned()
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

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandFile {
    #[serde(default)]
    command: BTreeMap<String, CommandConfig>,
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
    Invalid {
        path: PathBuf,
        message: String,
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
            Self::Invalid { path, message } => write!(formatter, "{}: {message}", path.display()),
        }
    }
}

fn resolve_git_references(
    repository: &Repository,
    references: RawReferencesConfig,
) -> Result<Vec<GitReferenceConfig>, ConfigLoadError> {
    let configured = match references.git {
        GitReferenceEntries::One(entry) => vec![entry],
        GitReferenceEntries::Many(entries) => entries,
    };
    let infer_default = configured.is_empty();
    let entries = if infer_default {
        vec![RawGitReferenceConfig {
            provider: None,
            repository: None,
            remote: default_git_remote(),
            host: None,
        }]
    } else {
        configured
    };
    let mut resolved = Vec::new();
    for entry in entries {
        let inferred = infer_remote(repository, &entry.remote);
        if infer_default && inferred.is_none() {
            continue;
        }
        let provider = entry
            .provider
            .or_else(|| inferred.as_ref().map(|value| value.0.clone()))
            .ok_or_else(|| invalid_reference_config(repository, &entry.remote))?;
        if docgraph_markdown::reference_adapter(&provider).is_none() {
            return Err(ConfigLoadError::Invalid {
                path: repository.project_file(),
                message: format!("unknown reference provider {provider:?}"),
            });
        }
        let repository_name = entry
            .repository
            .or_else(|| inferred.as_ref().map(|value| value.2.clone()))
            .ok_or_else(|| invalid_reference_config(repository, &entry.remote))?;
        let host = entry
            .host
            .or_else(|| inferred.map(|value| value.1))
            .unwrap_or_else(|| match provider.as_str() {
                "github" => "github.com".to_owned(),
                "gitlab" => "gitlab.com".to_owned(),
                _ => String::new(),
            });
        resolved.push(GitReferenceConfig {
            provider,
            repository: repository_name,
            host,
            remote: entry.remote,
        });
    }
    Ok(resolved)
}

fn invalid_reference_config(repository: &Repository, remote: &str) -> ConfigLoadError {
    ConfigLoadError::Invalid {
        path: repository.project_file(),
        message: format!(
            "references.git for remote {remote:?} requires provider and repository when the remote cannot be inferred"
        ),
    }
}

fn infer_remote(repository: &Repository, remote: &str) -> Option<(String, String, String)> {
    let output = Command::new("git")
        .args(["config", "--get", &format!("remote.{remote}.url")])
        .current_dir(repository.root())
        .output()
        .ok()?;
    output.status.success().then_some(())?;
    parse_remote_url(String::from_utf8(output.stdout).ok()?.trim())
}

fn parse_remote_url(url: &str) -> Option<(String, String, String)> {
    let without_scheme = url
        .split_once("://")
        .map_or(url, |(_, remainder)| remainder);
    let without_user = without_scheme
        .rsplit_once('@')
        .map_or(without_scheme, |(_, value)| value);
    let (host, path) = if url.contains("://") {
        without_user.split_once('/')?
    } else {
        without_user
            .split_once(':')
            .or_else(|| without_user.split_once('/'))?
    };
    let repository = path.trim_end_matches('/').trim_end_matches(".git");
    if repository.is_empty() {
        return None;
    }
    let provider = if host.eq_ignore_ascii_case("github.com") {
        "github"
    } else if host.eq_ignore_ascii_case("gitlab.com")
        || host.to_ascii_lowercase().contains("gitlab")
    {
        "gitlab"
    } else {
        return None;
    };
    Some((provider.to_owned(), host.to_owned(), repository.to_owned()))
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
            "logic.dl",
            "actionable(task) :- entity_state(task, \"open\").\n",
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
    fn loads_dynamic_commands() {
        let fixture = Fixture::new();
        fixture.write(
            "project.toml",
            "schema_version = 1\n[project]\nname = \"example\"\n[documents]\nroot = \"docs\"\n",
        );
        fixture.write(
            "commands.toml",
            "[command.next]\ndescription = \"Find candidate work\"\noperation = \"query\"\nquery = \"next_work\"\n",
        );

        let config = fixture.load().unwrap();
        assert!(matches!(
            config.commands["next"].operation,
            CommandOperation::Query { ref query, entity_type: None } if query == "next_work"
        ));
    }

    #[test]
    fn rejects_unknown_dynamic_command_fields() {
        let fixture = Fixture::new();
        fixture.write(
            "project.toml",
            "schema_version = 1\n[project]\nname = \"example\"\n[documents]\nroot = \"docs\"\n",
        );
        fixture.write(
            "commands.toml",
            "[command.next]\ndescription = \"Find candidate work\"\nopertion = \"query\"\nquery = \"next_work\"\n",
        );

        assert!(matches!(fixture.load(), Err(ConfigLoadError::Parse(_))));
    }

    #[test]
    fn parses_common_git_remote_forms_without_network_access() {
        assert_eq!(
            parse_remote_url("git@github.com:owner/repo.git"),
            Some((
                "github".to_owned(),
                "github.com".to_owned(),
                "owner/repo".to_owned()
            ))
        );
        assert_eq!(
            parse_remote_url("https://gitlab.com/group/project.git"),
            Some((
                "gitlab".to_owned(),
                "gitlab.com".to_owned(),
                "group/project".to_owned()
            ))
        );
        assert_eq!(
            parse_remote_url("https://code.example.com/owner/repo"),
            None
        );
    }

    #[test]
    fn loads_multiple_explicit_reference_adapters() {
        let fixture = Fixture::new();
        fixture.write(
            "project.toml",
            r#"schema_version = 1
[project]
name = "providers"
[documents]
root = "docs"
[[references.git]]
provider = "github"
host = "github.com"
repository = "owner/repo"
remote = "origin"
[[references.git]]
provider = "gitlab"
host = "git.example.com"
repository = "group/project"
remote = "upstream"
"#,
        );

        let config = fixture.load().unwrap();

        assert_eq!(config.project.references.len(), 2);
        assert_eq!(config.project.references[1].provider, "gitlab");
        assert_eq!(config.project.references[1].host, "git.example.com");
    }

    #[test]
    fn loads_embedding_provider_configuration() {
        let fixture = Fixture::new();
        fixture.write(
            "project.toml",
            r#"schema_version = 1
[project]
name = "vectors"
[documents]
root = "docs"
[embeddings]
provider = "local"
model = "example"
dimensions = 3
command = ["embed", "--stdio"]
fallback = "error"
"#,
        );

        let config = fixture.load().unwrap();
        let embeddings = config.project.embeddings.unwrap();
        assert_eq!(embeddings.identity(), "local:example:3");
        assert_eq!(embeddings.batch_size, 32);
        assert_eq!(embeddings.timeout_seconds, 30);
        assert_eq!(embeddings.fallback, EmbeddingFallback::Error);
    }
}
