//! Core domain model and repository services for docgraph.

mod changes;
mod config;
mod corpus;
mod derived_index;
mod generated_frontmatter;
mod graph;
mod instructions;
mod mutation;
mod repository;
mod retrieval;
mod state;
mod validation;

pub use config::{
    AgentInstructionsConfig, ArgumentMode, CommandConfig, CommandOperation, ConfigDiagnostic,
    ConfigLoadError, DiagnosticSeverity, DocumentsConfig, EntityTypeConfig, FrontmatterConfig,
    NamedQueryConfig, ProjectConfig, PropertyConfig, PropertyType, QueryArgumentConfig,
    QueryValueType, RelationTypeConfig, RepositoryConfig, ScalarType, ScalarValue, SourceSpan,
    StateConfig, ValidationConfig, WorkflowConfig,
};
pub use corpus::{CanonicalCorpus, CorpusError, CorpusFile, RepositoryFingerprint};
pub use derived_index::DerivedSearchHit;
pub use generated_frontmatter::{
    GeneratedBlockError, GeneratedBlockStatus, check_generated_frontmatter,
    sync_generated_frontmatter,
};
pub use graph::{
    DiagnosticKind as GraphDiagnosticKind, DocumentNode, EntityNode, GraphDiagnostic, GraphIndex,
    GraphLocation, GraphNode, Relation, RelationOrigin, SectionNode,
};
pub use instructions::{
    InstructionChange, InstructionError, InstructionService, InstructionStatus,
};
pub use mutation::{
    Adoption, FileChange, MutationError, MutationPlan, MutationRequest, MutationService,
};
pub use repository::{DiscoveryError, Repository};
pub use retrieval::{GraphTraversal, Neighbor};
pub use state::{DerivedState, DerivedStateError, DerivedStatePaths, IndexStatus};
pub use validation::{ValidationDiagnostic, ValidationLocation, ValidationReport, Validator};

/// Repository schema implemented by this pre-release workspace.
pub const SCHEMA_VERSION: u32 = 1;
pub use changes::{
    ChangeDiagnostic, ManagedChange, ManagedChangeReport, ManagedChangeValidator, SemanticChange,
    SemanticChangeReport, SemanticChangeReviewer, SemanticRelation, SemanticSection,
};
