use crate::{DerivedState, Repository, RepositoryConfig};
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Component, Path, PathBuf};

const BEGIN: &str = "<!-- docgraph:agent-instructions:v1:begin -->";
const END: &str = "<!-- docgraph:agent-instructions:end -->";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstructionStatus {
    Current,
    Missing,
    Stale,
    Malformed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstructionChange {
    pub path: PathBuf,
    pub original: Option<String>,
    pub intended: String,
}

pub struct InstructionService<'a> {
    repository: &'a Repository,
    config: &'a RepositoryConfig,
    state: DerivedState,
}

impl<'a> InstructionService<'a> {
    pub fn new(
        repository: &'a Repository,
        config: &'a RepositoryConfig,
    ) -> Result<Self, InstructionError> {
        let state = DerivedState::discover(repository)
            .map_err(|error| InstructionError::State(error.to_string()))?;
        Ok(Self {
            repository,
            config,
            state,
        })
    }

    pub fn check(&self) -> Result<Vec<(PathBuf, InstructionStatus)>, InstructionError> {
        self.config
            .project
            .agent_instructions
            .targets
            .iter()
            .map(|target| {
                let path = self.target_path(target)?;
                let status = match fs::read_to_string(&path) {
                    Ok(source) if !source.contains(BEGIN) && !source.contains(END) => {
                        InstructionStatus::Missing
                    }
                    Ok(source) => match replace_block(&source, &self.generated_block()) {
                        Ok(intended) if intended == source => InstructionStatus::Current,
                        Ok(_) => InstructionStatus::Stale,
                        Err(InstructionError::MalformedMarkers(_)) => InstructionStatus::Malformed,
                        Err(error) => return Err(error),
                    },
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        InstructionStatus::Missing
                    }
                    Err(source) => return Err(InstructionError::io(&path, source)),
                };
                Ok((target.clone(), status))
            })
            .collect()
    }

    pub fn sync(&self, dry_run: bool) -> Result<Vec<InstructionChange>, InstructionError> {
        let block = self.generated_block();
        let mut changes = Vec::new();
        for target in &self.config.project.agent_instructions.targets {
            let path = self.target_path(target)?;
            let original = match fs::read_to_string(&path) {
                Ok(source) => Some(source),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(source) => return Err(InstructionError::io(&path, source)),
            };
            let intended = original
                .as_deref()
                .map_or_else(
                    || Ok(format!("{block}\n")),
                    |source| replace_block(source, &block),
                )
                .map_err(|error| match error {
                    InstructionError::MalformedMarkers(_) => {
                        InstructionError::MalformedMarkers(target.clone())
                    }
                    error => error,
                })?;
            if original.as_deref() != Some(&intended) {
                changes.push(InstructionChange {
                    path: target.clone(),
                    original,
                    intended,
                });
            }
        }
        if dry_run || changes.is_empty() {
            return Ok(changes);
        }

        fs::create_dir_all(&self.state.paths.directory)
            .map_err(|source| InstructionError::io(&self.state.paths.directory, source))?;
        let _lock = InstructionLock::acquire(&self.state.paths.mutation_lock)?;
        for change in &changes {
            let absolute = self.target_path(&change.path)?;
            let current = match fs::read_to_string(&absolute) {
                Ok(source) => Some(source),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(source) => return Err(InstructionError::io(&absolute, source)),
            };
            if current != change.original {
                return Err(InstructionError::ConcurrentEdit(change.path.clone()));
            }
        }
        for change in &changes {
            let absolute = self.target_path(&change.path)?;
            if let Some(parent) = absolute.parent() {
                fs::create_dir_all(parent)
                    .map_err(|source| InstructionError::io(parent, source))?;
            }
            replace_file(&absolute, &change.intended)?;
        }
        Ok(changes)
    }

    fn generated_block(&self) -> String {
        let entity_types = joined(self.config.entities.keys().map(String::as_str));
        let relations = joined(self.config.relations.keys().map(String::as_str));
        let workflows = joined(self.config.workflows.keys().map(String::as_str));
        let queries = joined(self.config.queries.keys().map(String::as_str));
        format!(
            "{BEGIN}\nThis repository uses docgraph.\n\n- Edit prose directly. Use `docgraph` commands for managed frontmatter and semantic relationships.\n- Inspect the repository model with `docgraph describe`; do not reconstruct semantic impact with grep.\n- Preview substantial changes with `--dry-run`, then run `docgraph validate`.\n- Keep generated frontmatter current with `docgraph frontmatter sync`.\n- Portable guidance lives in `skills/docgraph/SKILL.md`.\n\nModel: entities [{entity_types}]; relations [{relations}]; workflows [{workflows}]; queries [{queries}].\n{END}"
        )
    }

    fn target_path(&self, target: &Path) -> Result<PathBuf, InstructionError> {
        if target.is_absolute()
            || target
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
        {
            return Err(InstructionError::OutsideRepository(target.to_path_buf()));
        }
        Ok(self.repository.root().join(target))
    }
}

fn joined<'a>(values: impl Iterator<Item = &'a str>) -> String {
    values.collect::<Vec<_>>().join(", ")
}

fn replace_block(source: &str, block: &str) -> Result<String, InstructionError> {
    let mut begins = Vec::new();
    let mut ends = Vec::new();
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let logical = line.strip_suffix('\n').unwrap_or(line);
        let logical = logical.strip_suffix('\r').unwrap_or(logical);
        if logical == BEGIN {
            begins.push(offset);
        } else if logical == END {
            ends.push(offset + line.len());
        } else if logical.contains("docgraph:agent-instructions") {
            return Err(InstructionError::MalformedMarkers(PathBuf::new()));
        }
        offset += line.len();
    }
    match (begins.as_slice(), ends.as_slice()) {
        ([], []) => {
            let mut output = source.to_owned();
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push('\n');
            }
            output.push_str(block);
            output.push('\n');
            Ok(output)
        }
        ([begin], [end]) if begin < end => {
            let newline = if source.contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            };
            let block = block.replace('\n', newline);
            let mut output = source.to_owned();
            output.replace_range(*begin..*end, &format!("{block}{newline}"));
            Ok(output)
        }
        _ => Err(InstructionError::MalformedMarkers(PathBuf::new())),
    }
}

fn replace_file(path: &Path, intended: &str) -> Result<(), InstructionError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("instructions");
    let temporary =
        path.with_file_name(format!(".{file_name}.docgraph-{}.tmp", std::process::id()));
    fs::write(&temporary, intended).map_err(|source| InstructionError::io(&temporary, source))?;
    if let Err(source) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(InstructionError::io(path, source));
    }
    Ok(())
}

struct InstructionLock(PathBuf);

impl InstructionLock {
    fn acquire(path: &Path) -> Result<Self, InstructionError> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|source| {
                if source.kind() == io::ErrorKind::AlreadyExists {
                    InstructionError::Locked(path.to_path_buf())
                } else {
                    InstructionError::io(path, source)
                }
            })?;
        Ok(Self(path.to_path_buf()))
    }
}

impl Drop for InstructionLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[derive(Debug)]
pub enum InstructionError {
    State(String),
    OutsideRepository(PathBuf),
    MalformedMarkers(PathBuf),
    ConcurrentEdit(PathBuf),
    Locked(PathBuf),
    Io { path: PathBuf, source: io::Error },
}

impl InstructionError {
    fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

impl fmt::Display for InstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(message) => formatter.write_str(message),
            Self::OutsideRepository(path) => write!(
                formatter,
                "instruction target {} is outside the repository",
                path.display()
            ),
            Self::MalformedMarkers(path) => write!(
                formatter,
                "instruction markers are malformed or ambiguous in {}",
                path.display()
            ),
            Self::ConcurrentEdit(path) => write!(
                formatter,
                "instruction target {} changed during sync",
                path.display()
            ),
            Self::Locked(path) => write!(formatter, "another mutation holds {}", path.display()),
            Self::Io { path, source } => {
                write!(formatter, "cannot update {}: {source}", path.display())
            }
        }
    }
}

impl Error for InstructionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn block_updates_preserve_user_content_and_are_idempotent() {
        let original = "# Local guidance\n\n<!-- docgraph:agent-instructions:v1:begin -->\nold\n<!-- docgraph:agent-instructions:end -->\n\nFooter\n";
        let block = format!("{BEGIN}\nnew\n{END}");
        let once = replace_block(original, &block).unwrap();
        let twice = replace_block(&once, &block).unwrap();
        assert_eq!(once, twice);
        assert!(once.starts_with("# Local guidance"));
        assert!(once.ends_with("\nFooter\n"));
    }

    #[test]
    fn malformed_markers_are_never_guessed() {
        let source = format!("{BEGIN}\n{BEGIN}\n{END}\n");
        assert!(matches!(
            replace_block(&source, "replacement"),
            Err(InstructionError::MalformedMarkers(_))
        ));
    }

    #[test]
    fn service_syncs_configured_targets_without_touching_user_guidance() {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "docgraph-instructions-test-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join(".docgraph")).unwrap();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(
            root.join(".docgraph/project.toml"),
            "schema_version = 1\n[project]\nname = \"fixture\"\n[documents]\nroot = \"docs\"\n[agent_instructions]\ntargets = [\"AGENTS.md\"]\n",
        )
        .unwrap();
        fs::write(root.join("AGENTS.md"), "# Keep me\n").unwrap();
        let repository = Repository::discover(&root).unwrap();
        let config = RepositoryConfig::load(&repository).unwrap();
        let service = InstructionService::new(&repository, &config).unwrap();

        assert_eq!(service.check().unwrap()[0].1, InstructionStatus::Missing);
        assert_eq!(service.sync(true).unwrap().len(), 1);
        assert_eq!(
            fs::read_to_string(root.join("AGENTS.md")).unwrap(),
            "# Keep me\n"
        );
        service.sync(false).unwrap();
        let synced = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(synced.starts_with("# Keep me\n"));
        assert!(synced.contains(BEGIN));
        assert!(service.sync(false).unwrap().is_empty());
        assert_eq!(service.check().unwrap()[0].1, InstructionStatus::Current);
        let _ = fs::remove_dir_all(root);
    }
}
