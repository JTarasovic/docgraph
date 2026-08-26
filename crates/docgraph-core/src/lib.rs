//! Core domain model and repository services for docgraph.

mod config;
mod corpus;
mod graph;
mod repository;
mod state;

pub use config::{
    AgentInstructionsConfig, ArgumentMode, ConfigDiagnostic, ConfigLoadError, DiagnosticSeverity,
    DocumentsConfig, EntityTypeConfig, FrontmatterConfig, NamedQueryConfig, ProjectConfig,
    PropertyConfig, PropertyType, QueryArgumentConfig, QueryValueType, RelationTypeConfig,
    RepositoryConfig, ScalarType, ScalarValue, SourceSpan, StateConfig, ValidationConfig,
    WorkflowConfig,
};
pub use corpus::{CanonicalCorpus, CorpusError, CorpusFile, RepositoryFingerprint};
pub use graph::{
    DiagnosticKind as GraphDiagnosticKind, DocumentNode, EntityNode, GraphDiagnostic, GraphIndex,
    GraphLocation, GraphNode, Relation, RelationOrigin, SectionNode,
};
pub use repository::{DiscoveryError, Repository};
pub use state::{DerivedState, DerivedStateError, DerivedStatePaths, IndexStatus};

/// Repository schema implemented by this pre-release workspace.
pub const SCHEMA_VERSION: u32 = 1;
