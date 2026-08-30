use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CONFIG_DIRECTORY: &str = ".docgraph";
const PROJECT_FILE: &str = "project.toml";

/// A repository containing a discoverable docgraph configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Repository {
    root: PathBuf,
    config_dir: PathBuf,
}

impl Repository {
    /// Constructs a repository handle for an exact root. Initialization uses
    /// this before `.docgraph/project.toml` exists; ordinary callers should
    /// prefer [`Self::discover`].
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, DiscoveryError> {
        let requested = root.as_ref();
        let root = fs::canonicalize(requested).map_err(|source| DiscoveryError::Access {
            path: requested.to_path_buf(),
            source,
        })?;
        Ok(Self {
            config_dir: root.join(CONFIG_DIRECTORY),
            root,
        })
    }

    /// Finds `.docgraph/project.toml` at the enclosing Git worktree root. Outside
    /// Git, the nearest ancestor containing the file defines the repository root.
    pub fn discover(start: impl AsRef<Path>) -> Result<Self, DiscoveryError> {
        let requested = start.as_ref();
        let canonical = fs::canonicalize(requested).map_err(|source| DiscoveryError::Access {
            path: requested.to_path_buf(),
            source,
        })?;
        let mut current = if canonical.is_file() {
            canonical
                .parent()
                .expect("a file has a parent")
                .to_path_buf()
        } else {
            canonical
        };

        let mut standalone_candidate = None;
        loop {
            let config_dir = current.join(CONFIG_DIRECTORY);
            if config_dir.join(PROJECT_FILE).is_file() {
                standalone_candidate.get_or_insert_with(|| Self {
                    root: current.clone(),
                    config_dir: config_dir.clone(),
                });
            }

            // A `.git` directory or worktree pointer marks the repository boundary.
            if current.join(".git").exists() {
                if config_dir.join(PROJECT_FILE).is_file() {
                    return Ok(Self {
                        root: current,
                        config_dir,
                    });
                }
                return Err(DiscoveryError::NotConfigured { boundary: current });
            }

            let Some(parent) = current.parent() else {
                return standalone_candidate
                    .ok_or(DiscoveryError::NotConfigured { boundary: current });
            };
            current = parent.to_path_buf();
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn project_file(&self) -> PathBuf {
        self.config_dir.join(PROJECT_FILE)
    }
}

#[derive(Debug)]
pub enum DiscoveryError {
    Access { path: PathBuf, source: io::Error },
    NotConfigured { boundary: PathBuf },
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Access { path, source } => {
                write!(formatter, "cannot inspect {}: {source}", path.display())
            }
            Self::NotConfigured { boundary } => write!(
                formatter,
                "no .docgraph/project.toml found at or below {}",
                boundary.display()
            ),
        }
    }
}

impl Error for DiscoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Access { source, .. } => Some(source),
            Self::NotConfigured { .. } => None,
        }
    }
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
            let path = std::env::temp_dir().join(format!(
                "docgraph-repository-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn discovers_from_a_nested_path() {
        let temp = TempDirectory::new();
        let nested = temp.0.join("docs/reference");
        fs::create_dir_all(temp.0.join(".docgraph")).unwrap();
        fs::create_dir_all(&nested).unwrap();
        fs::write(temp.0.join(".docgraph/project.toml"), "schema_version = 1").unwrap();

        let repository = Repository::discover(&nested).unwrap();

        assert_eq!(repository.root(), fs::canonicalize(&temp.0).unwrap());
    }

    #[test]
    fn does_not_escape_an_unconfigured_git_worktree() {
        let temp = TempDirectory::new();
        let outer_config = temp.0.join(".docgraph");
        let worktree = temp.0.join("worktree");
        let nested = worktree.join("docs");
        fs::create_dir_all(&outer_config).unwrap();
        fs::write(outer_config.join("project.toml"), "schema_version = 1").unwrap();
        fs::create_dir_all(&nested).unwrap();
        fs::write(worktree.join(".git"), "gitdir: ../bare/worktrees/main").unwrap();

        let error = Repository::discover(&nested).unwrap_err();

        assert!(
            matches!(error, DiscoveryError::NotConfigured { boundary } if boundary == fs::canonicalize(worktree).unwrap())
        );
    }

    #[test]
    fn ignores_nested_configs_inside_a_git_worktree() {
        let temp = TempDirectory::new();
        let nested_root = temp.0.join("docs");
        fs::create_dir_all(nested_root.join(".docgraph")).unwrap();
        fs::create_dir_all(temp.0.join(".git")).unwrap();
        fs::write(
            nested_root.join(".docgraph/project.toml"),
            "schema_version = 1",
        )
        .unwrap();

        let error = Repository::discover(&nested_root).unwrap_err();

        assert!(
            matches!(error, DiscoveryError::NotConfigured { boundary } if boundary == fs::canonicalize(&temp.0).unwrap())
        );
    }
}
