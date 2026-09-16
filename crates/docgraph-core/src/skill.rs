use crate::{DerivedState, Repository, state::StateLock};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const PORTABLE_SKILL_CONTRACT_VERSION: i64 = 1;

const MANAGED_FILES: &[(&str, &str)] = &[
    (
        "SKILL.md",
        include_str!("../../../skills/docgraph/SKILL.md"),
    ),
    (
        "commands.md",
        include_str!("../../../skills/docgraph/commands.md"),
    ),
    (
        "config-authorship.md",
        include_str!("../../../skills/docgraph/config-authorship.md"),
    ),
    (
        "document-authoring.md",
        include_str!("../../../skills/docgraph/document-authoring.md"),
    ),
    (
        "mutations.md",
        include_str!("../../../skills/docgraph/mutations.md"),
    ),
    (
        "querying.md",
        include_str!("../../../skills/docgraph/querying.md"),
    ),
    (
        "relationships.md",
        include_str!("../../../skills/docgraph/relationships.md"),
    ),
    (
        "repository-maintenance.md",
        include_str!("../../../skills/docgraph/repository-maintenance.md"),
    ),
    (
        "skill.toml",
        include_str!("../../../skills/docgraph/skill.toml"),
    ),
    (
        "troubleshooting.md",
        include_str!("../../../skills/docgraph/troubleshooting.md"),
    ),
    (
        "workflows.md",
        include_str!("../../../skills/docgraph/workflows.md"),
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortableSkillStatus {
    Current,
    Missing,
    Modified,
    Incompatible,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortableSkillChange {
    pub path: PathBuf,
    pub original: Option<String>,
    pub intended: String,
}

pub struct PortableSkillService<'a> {
    repository: &'a Repository,
    state: DerivedState,
    targets: &'a [PathBuf],
}

impl<'a> PortableSkillService<'a> {
    pub fn new(
        repository: &'a Repository,
        targets: &'a [PathBuf],
    ) -> Result<Self, PortableSkillError> {
        let state = DerivedState::discover(repository)
            .map_err(|error| PortableSkillError::State(error.to_string()))?;
        Ok(Self {
            repository,
            state,
            targets,
        })
    }

    pub fn check(&self) -> Result<Vec<(PathBuf, PortableSkillStatus)>, PortableSkillError> {
        self.targets
            .iter()
            .map(|target| Ok((target.clone(), self.check_target(target)?)))
            .collect()
    }

    fn check_target(&self, target: &Path) -> Result<PortableSkillStatus, PortableSkillError> {
        let manifest_path = self.skill_path(target)?.join("skill.toml");
        let manifest = match fs::read_to_string(&manifest_path) {
            Ok(source) => source,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(PortableSkillStatus::Missing);
            }
            Err(source) => return Err(PortableSkillError::io(&manifest_path, source)),
        };
        if !manifest_is_compatible(&manifest) {
            return Ok(PortableSkillStatus::Incompatible);
        }

        let mut modified = false;
        for (name, intended) in MANAGED_FILES {
            let path = self.skill_path(target)?.join(name);
            match fs::read_to_string(&path) {
                Ok(source) if normalized(&source) == normalized(intended) => {}
                Ok(_) => modified = true,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    return Ok(PortableSkillStatus::Missing);
                }
                Err(source) => return Err(PortableSkillError::io(&path, source)),
            }
        }
        Ok(if modified {
            PortableSkillStatus::Modified
        } else {
            PortableSkillStatus::Current
        })
    }

    pub fn sync(&self, dry_run: bool) -> Result<Vec<PortableSkillChange>, PortableSkillError> {
        let mut changes = Vec::new();
        for target in self.targets {
            for (name, embedded) in MANAGED_FILES {
                let path = self.skill_path(target)?.join(name);
                let original = match fs::read_to_string(&path) {
                    Ok(source) => Some(source),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                    Err(source) => return Err(PortableSkillError::io(&path, source)),
                };
                let intended = normalized(embedded);
                if original.as_deref().map(normalized).as_deref() != Some(&intended) {
                    changes.push(PortableSkillChange {
                        path: target.join(name),
                        original,
                        intended,
                    });
                }
            }
        }
        if dry_run || changes.is_empty() {
            return Ok(changes);
        }

        fs::create_dir_all(&self.state.paths.directory)
            .map_err(|source| PortableSkillError::io(&self.state.paths.directory, source))?;
        let _lock = StateLock::acquire(&self.state.paths.mutation_lock).map_err(|source| {
            if source.kind() == io::ErrorKind::WouldBlock {
                PortableSkillError::Locked(self.state.paths.mutation_lock.clone())
            } else {
                PortableSkillError::io(&self.state.paths.mutation_lock, source)
            }
        })?;
        for change in &changes {
            let absolute = self.skill_path(&change.path)?;
            let current = match fs::read_to_string(&absolute) {
                Ok(source) => Some(source),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(source) => return Err(PortableSkillError::io(&absolute, source)),
            };
            if current != change.original {
                return Err(PortableSkillError::ConcurrentEdit(change.path.clone()));
            }
        }
        for change in &changes {
            let absolute = self.skill_path(&change.path)?;
            if let Some(parent) = absolute.parent() {
                fs::create_dir_all(parent)
                    .map_err(|source| PortableSkillError::io(parent, source))?;
            }
            replace_file(&absolute, &change.intended)?;
        }
        Ok(changes)
    }

    fn skill_path(&self, target: &Path) -> Result<PathBuf, PortableSkillError> {
        if target.as_os_str().is_empty()
            || target.is_absolute()
            || target.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir | std::path::Component::Prefix(_)
                )
            })
        {
            return Err(PortableSkillError::InvalidTarget(target.to_path_buf()));
        }
        let mut path = self.repository.root().to_path_buf();
        for component in target.components() {
            path.push(component);
            match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(PortableSkillError::SymlinkTarget(target.to_path_buf()));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(source) => return Err(PortableSkillError::io(&path, source)),
            }
        }
        Ok(path)
    }
}

fn manifest_is_compatible(source: &str) -> bool {
    let Ok(document) = source.parse::<toml_edit::DocumentMut>() else {
        return false;
    };
    document["schema_version"].as_integer() == Some(1)
        && document["contract_version"].as_integer() == Some(PORTABLE_SKILL_CONTRACT_VERSION)
        && document["cli_version"].as_str() == Some(env!("CARGO_PKG_VERSION"))
}

fn normalized(source: &str) -> String {
    source.replace("\r\n", "\n")
}

fn replace_file(path: &Path, intended: &str) -> Result<(), PortableSkillError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("skill");
    let temporary =
        path.with_file_name(format!(".{file_name}.docgraph-{}.tmp", std::process::id()));
    fs::write(&temporary, intended).map_err(|source| PortableSkillError::io(&temporary, source))?;
    if let Err(source) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(PortableSkillError::io(path, source));
    }
    Ok(())
}

#[derive(Debug)]
pub enum PortableSkillError {
    State(String),
    InvalidTarget(PathBuf),
    SymlinkTarget(PathBuf),
    ConcurrentEdit(PathBuf),
    Locked(PathBuf),
    Io { path: PathBuf, source: io::Error },
}

impl PortableSkillError {
    fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

impl fmt::Display for PortableSkillError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(message) => formatter.write_str(message),
            Self::InvalidTarget(path) => write!(
                formatter,
                "skill target {} must be repository-relative",
                path.display()
            ),
            Self::SymlinkTarget(path) => write!(
                formatter,
                "skill target {} passes through a symlink",
                path.display()
            ),
            Self::ConcurrentEdit(path) => {
                write!(
                    formatter,
                    "portable skill {} changed during sync",
                    path.display()
                )
            }
            Self::Locked(path) => write!(formatter, "another mutation holds {}", path.display()),
            Self::Io { path, source } => {
                write!(formatter, "cannot update {}: {source}", path.display())
            }
        }
    }
}

impl Error for PortableSkillError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
