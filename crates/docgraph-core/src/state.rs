use crate::{
    CanonicalCorpus, DerivedExternalEntity, DerivedSearchHit, EmbeddingConfig, EmbeddingError,
    EmbeddingProvider, GraphIndex, Repository, RepositoryFingerprint, SCHEMA_VERSION,
    SemanticSearchResult, derived_index,
};
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

const INDEX_FORMAT_VERSION: u32 = 3;

pub(crate) struct StateLock {
    file: File,
}

impl StateLock {
    pub(crate) fn acquire(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        file.try_lock()?;
        Ok(Self { file })
    }
}

impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedStatePaths {
    pub directory: PathBuf,
    pub index: PathBuf,
    pub fingerprint: PathBuf,
    pub mutation_lock: PathBuf,
    pub recovery_journal: PathBuf,
    pub external_cache: PathBuf,
}

impl DerivedStatePaths {
    pub fn discover(repository: &Repository) -> Result<Self, DerivedStateError> {
        let git_marker = repository.root().join(".git");
        let directory = if git_marker.is_dir() {
            git_marker.join("docgraph")
        } else if git_marker.is_file() {
            let pointer =
                fs::read_to_string(&git_marker).map_err(|source| DerivedStateError::Io {
                    path: git_marker.clone(),
                    source,
                })?;
            let git_dir = pointer
                .lines()
                .next()
                .and_then(|line| line.trim().strip_prefix("gitdir:"))
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .ok_or_else(|| DerivedStateError::InvalidGitPointer {
                    path: git_marker.clone(),
                })?;
            let git_dir = Path::new(git_dir);
            let git_dir = if git_dir.is_absolute() {
                git_dir.to_path_buf()
            } else {
                repository.root().join(git_dir)
            };
            let git_dir = fs::canonicalize(&git_dir).map_err(|source| DerivedStateError::Io {
                path: git_dir,
                source,
            })?;
            git_dir.join("docgraph")
        } else {
            repository.config_dir().join(".state")
        };

        Ok(Self {
            index: directory.join("index.sqlite"),
            fingerprint: directory.join("fingerprint"),
            mutation_lock: directory.join("mutation.lock"),
            recovery_journal: directory.join("recovery.toml"),
            external_cache: directory.join("external.sqlite"),
            directory,
        })
    }
}

#[derive(Clone, Debug)]
pub struct DerivedState {
    pub paths: DerivedStatePaths,
}

impl DerivedState {
    pub fn discover(repository: &Repository) -> Result<Self, DerivedStateError> {
        Ok(Self {
            paths: DerivedStatePaths::discover(repository)?,
        })
    }

    pub fn status(&self, current: RepositoryFingerprint) -> Result<IndexStatus, DerivedStateError> {
        if !self.paths.index.is_file() {
            return Ok(IndexStatus::Missing);
        }
        let metadata = match fs::read_to_string(&self.paths.fingerprint) {
            Ok(metadata) => metadata,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Ok(IndexStatus::Missing);
            }
            Err(source) => {
                return Err(DerivedStateError::Io {
                    path: self.paths.fingerprint.clone(),
                    source,
                });
            }
        };
        let recorded =
            parse_metadata(&metadata).ok_or_else(|| DerivedStateError::CorruptMetadata {
                path: self.paths.fingerprint.clone(),
            })?;
        if recorded == current {
            let indexed = derived_index::recorded_fingerprint(&self.paths.index)
                .map_err(|source| DerivedStateError::Sqlite {
                    path: self.paths.index.clone(),
                    source,
                })?
                .ok_or_else(|| DerivedStateError::CorruptMetadata {
                    path: self.paths.index.clone(),
                })?;
            if indexed != recorded {
                return Err(DerivedStateError::CorruptMetadata {
                    path: self.paths.index.clone(),
                });
            }
        }
        if recorded == current {
            Ok(IndexStatus::Fresh)
        } else {
            Ok(IndexStatus::Stale { recorded, current })
        }
    }

    pub fn record(&self, fingerprint: RepositoryFingerprint) -> Result<(), DerivedStateError> {
        fs::create_dir_all(&self.paths.directory).map_err(|source| DerivedStateError::Io {
            path: self.paths.directory.clone(),
            source,
        })?;
        let metadata = format!(
            "index_format={INDEX_FORMAT_VERSION}\nschema_version={SCHEMA_VERSION}\nfingerprint={fingerprint}\n"
        );
        fs::write(&self.paths.fingerprint, metadata).map_err(|source| DerivedStateError::Io {
            path: self.paths.fingerprint.clone(),
            source,
        })
    }

    pub fn refresh(
        &self,
        corpus: &CanonicalCorpus,
        graph: &GraphIndex,
    ) -> Result<(), DerivedStateError> {
        self.refresh_with_embeddings(corpus, graph, None)
    }

    pub fn refresh_with_embeddings(
        &self,
        corpus: &CanonicalCorpus,
        graph: &GraphIndex,
        embeddings: Option<(&EmbeddingConfig, &dyn EmbeddingProvider)>,
    ) -> Result<(), DerivedStateError> {
        self.refresh_with_external(corpus, graph, &[], embeddings)
    }

    pub fn refresh_with_external(
        &self,
        corpus: &CanonicalCorpus,
        graph: &GraphIndex,
        external: &[DerivedExternalEntity],
        embeddings: Option<(&EmbeddingConfig, &dyn EmbeddingProvider)>,
    ) -> Result<(), DerivedStateError> {
        fs::create_dir_all(&self.paths.directory).map_err(|source| DerivedStateError::Io {
            path: self.paths.directory.clone(),
            source,
        })?;
        let temporary = self.paths.index.with_extension("sqlite.next");
        match fs::remove_file(&temporary) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(DerivedStateError::Io {
                    path: temporary,
                    source,
                });
            }
        }
        derived_index::build_with_external(&temporary, corpus.fingerprint, corpus, graph, external)
            .map_err(|source| DerivedStateError::Sqlite {
                path: temporary.clone(),
                source,
            })?;
        if let Some((config, provider)) = embeddings
            && let Err(error) = derived_index::index_vectors(
                &temporary,
                self.paths
                    .index
                    .is_file()
                    .then_some(self.paths.index.as_path()),
                config,
                provider,
            )
        {
            derived_index::record_vector_failure(&temporary, &error).map_err(|source| {
                DerivedStateError::Sqlite {
                    path: temporary.clone(),
                    source,
                }
            })?;
        }
        match fs::remove_file(&self.paths.index) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(DerivedStateError::Io {
                    path: self.paths.index.clone(),
                    source,
                });
            }
        }
        fs::rename(&temporary, &self.paths.index).map_err(|source| DerivedStateError::Io {
            path: self.paths.index.clone(),
            source,
        })?;
        self.record(corpus.fingerprint)
    }

    pub fn ensure_fresh(
        &self,
        corpus: &CanonicalCorpus,
        graph: &GraphIndex,
    ) -> Result<(), DerivedStateError> {
        self.ensure_fresh_with_embeddings(corpus, graph, None)
    }

    pub fn ensure_fresh_with_embeddings(
        &self,
        corpus: &CanonicalCorpus,
        graph: &GraphIndex,
        embeddings: Option<(&EmbeddingConfig, &dyn EmbeddingProvider)>,
    ) -> Result<(), DerivedStateError> {
        self.ensure_fresh_with_external(corpus, graph, &[], embeddings)
    }

    pub fn ensure_fresh_with_external(
        &self,
        corpus: &CanonicalCorpus,
        graph: &GraphIndex,
        external: &[DerivedExternalEntity],
        embeddings: Option<(&EmbeddingConfig, &dyn EmbeddingProvider)>,
    ) -> Result<(), DerivedStateError> {
        let external_matches = || {
            derived_index::recorded_external_fingerprint(&self.paths.index)
                .map(|recorded| {
                    recorded.as_deref() == Some(&derived_index::external_fingerprint(external))
                })
                .unwrap_or(false)
        };
        match self.status(corpus.fingerprint) {
            Ok(IndexStatus::Fresh) if external_matches() => Ok(()),
            Ok(IndexStatus::Fresh) => {
                self.refresh_with_external(corpus, graph, external, embeddings)
            }
            Ok(IndexStatus::Missing | IndexStatus::Stale { .. })
            | Err(DerivedStateError::CorruptMetadata { .. } | DerivedStateError::Sqlite { .. }) => {
                self.refresh_with_external(corpus, graph, external, embeddings)
            }
            Err(error) => Err(error),
        }
    }

    pub fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<DerivedSearchHit>, DerivedStateError> {
        derived_index::search(&self.paths.index, query, limit).map_err(|source| {
            DerivedStateError::Sqlite {
                path: self.paths.index.clone(),
                source,
            }
        })
    }

    pub fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        config: &EmbeddingConfig,
        provider: &dyn EmbeddingProvider,
    ) -> Result<SemanticSearchResult, EmbeddingError> {
        derived_index::semantic_search(&self.paths.index, query, limit, config, provider)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexStatus {
    Missing,
    Fresh,
    Stale {
        recorded: RepositoryFingerprint,
        current: RepositoryFingerprint,
    },
}

#[derive(Debug)]
pub enum DerivedStateError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Sqlite {
        path: PathBuf,
        source: rusqlite::Error,
    },
    InvalidGitPointer {
        path: PathBuf,
    },
    CorruptMetadata {
        path: PathBuf,
    },
}

impl fmt::Display for DerivedStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "cannot access {}: {source}", path.display())
            }
            Self::InvalidGitPointer { path } => {
                write!(
                    formatter,
                    "{} is not a valid Git worktree pointer",
                    path.display()
                )
            }
            Self::Sqlite { path, source } => {
                write!(
                    formatter,
                    "cannot use derived index {}: {source}",
                    path.display()
                )
            }
            Self::CorruptMetadata { path } => {
                write!(
                    formatter,
                    "{} contains invalid index metadata",
                    path.display()
                )
            }
        }
    }
}

impl Error for DerivedStateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Sqlite { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn parse_metadata(metadata: &str) -> Option<RepositoryFingerprint> {
    let mut index_format = None;
    let mut schema_version = None;
    let mut fingerprint = None;
    for line in metadata.lines() {
        let (key, value) = line.split_once('=')?;
        match key {
            "index_format" => index_format = value.parse::<u32>().ok(),
            "schema_version" => schema_version = value.parse::<u32>().ok(),
            "fingerprint" => fingerprint = RepositoryFingerprint::from_hex(value),
            _ => return None,
        }
    }
    (index_format == Some(INDEX_FORMAT_VERSION) && schema_version == Some(SCHEMA_VERSION))
        .then_some(fingerprint)
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "docgraph-state-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&root).unwrap();
            Self(root)
        }

        fn repository(&self, name: &str, git_dir: &Path) -> Repository {
            let root = self.0.join(name);
            fs::create_dir_all(root.join(".docgraph")).unwrap();
            fs::create_dir_all(git_dir).unwrap();
            fs::write(root.join(".docgraph/project.toml"), "schema_version = 1\n").unwrap();
            fs::write(
                root.join(".git"),
                format!("gitdir: {}\n", git_dir.display()),
            )
            .unwrap();
            Repository::discover(root).unwrap()
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn fingerprint(digit: char) -> RepositoryFingerprint {
        RepositoryFingerprint::from_hex(&digit.to_string().repeat(64)).unwrap()
    }

    #[test]
    fn linked_worktrees_get_separate_state_directories() {
        let temp = TempDirectory::new();
        let first = temp.repository("one", &temp.0.join("gitdirs/one"));
        let second = temp.repository("two", &temp.0.join("gitdirs/two"));

        let first = DerivedStatePaths::discover(&first).unwrap();
        let second = DerivedStatePaths::discover(&second).unwrap();

        assert_ne!(first.directory, second.directory);
        assert!(first.directory.ends_with("gitdirs/one/docgraph"));
        assert!(second.directory.ends_with("gitdirs/two/docgraph"));
    }

    #[test]
    fn detects_missing_fresh_and_stale_indexes() {
        let temp = TempDirectory::new();
        let repository = temp.repository("one", &temp.0.join("gitdirs/one"));
        let state = DerivedState::discover(&repository).unwrap();
        let first = fingerprint('1');
        let second = fingerprint('2');

        assert_eq!(state.status(first).unwrap(), IndexStatus::Missing);
        let corpus = CanonicalCorpus {
            repository_root: repository.root().to_path_buf(),
            files: Vec::new(),
            fingerprint: first,
        };
        let graph = GraphIndex {
            documents: Vec::new(),
            entities: Vec::new(),
            sections: Vec::new(),
            relations: Vec::new(),
            diagnostics: Vec::new(),
        };
        state.refresh(&corpus, &graph).unwrap();

        assert_eq!(state.status(first).unwrap(), IndexStatus::Fresh);
        assert_eq!(
            state.status(second).unwrap(),
            IndexStatus::Stale {
                recorded: first,
                current: second
            }
        );
    }

    #[test]
    fn replaces_an_old_or_corrupt_index_when_ensuring_freshness() {
        let temp = TempDirectory::new();
        let repository = temp.repository("one", &temp.0.join("gitdirs/one"));
        let state = DerivedState::discover(&repository).unwrap();
        let current = fingerprint('1');
        let corpus = CanonicalCorpus {
            repository_root: repository.root().to_path_buf(),
            files: Vec::new(),
            fingerprint: current,
        };
        let graph = GraphIndex {
            documents: Vec::new(),
            entities: Vec::new(),
            sections: Vec::new(),
            relations: Vec::new(),
            diagnostics: Vec::new(),
        };
        fs::create_dir_all(&state.paths.directory).unwrap();
        fs::write(&state.paths.index, b"docgraph-derived-index-v1\n").unwrap();
        state.record(current).unwrap();

        state.ensure_fresh(&corpus, &graph).unwrap();

        assert_eq!(state.status(current).unwrap(), IndexStatus::Fresh);
    }
}
