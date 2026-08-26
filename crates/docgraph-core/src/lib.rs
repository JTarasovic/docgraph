//! Core domain model and repository services for docgraph.

mod config;
mod repository;

pub use config::{
    AgentInstructionsConfig, ArgumentMode, ConfigDiagnostic, ConfigLoadError, DiagnosticSeverity,
    DocumentsConfig, EntityTypeConfig, FrontmatterConfig, NamedQueryConfig, ProjectConfig,
    PropertyConfig, PropertyType, QueryArgumentConfig, QueryValueType, RelationTypeConfig,
    RepositoryConfig, ScalarType, ScalarValue, SourceSpan, StateConfig, ValidationConfig,
    WorkflowConfig,
};
pub use repository::{DiscoveryError, Repository};

/// Repository schema implemented by this pre-release workspace.
pub const SCHEMA_VERSION: u32 = 1;
