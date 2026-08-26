use crate::{Repository, RepositoryFingerprint, SCHEMA_VERSION};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const INDEX_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedStatePaths {
    pub directory: PathBuf,
    pub index: PathBuf,
    pub fingerprint: PathBuf,
    pub mutation_lock: PathBuf,
    pub recovery_journal: PathBuf,
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
    Io { path: PathBuf, source: io::Error },
    InvalidGitPointer { path: PathBuf },
    CorruptMetadata { path: PathBuf },
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
        fs::create_dir_all(&state.paths.directory).unwrap();
        fs::write(&state.paths.index, b"derived index").unwrap();
        state.record(first).unwrap();

        assert_eq!(state.status(first).unwrap(), IndexStatus::Fresh);
        assert_eq!(
            state.status(second).unwrap(),
            IndexStatus::Stale {
                recorded: first,
                current: second
            }
        );
    }
}
