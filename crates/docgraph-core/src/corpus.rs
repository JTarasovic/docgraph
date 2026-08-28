use crate::{Repository, RepositoryConfig};
use docgraph_markdown::{FrontmatterError, ParsedDocument};
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const FINGERPRINT_DOMAIN: &[u8] = b"docgraph-canonical-inputs-v1\0";
const PARSER_REVISION: u32 = 2;

#[derive(Clone, Debug)]
pub struct CanonicalCorpus {
    pub files: Vec<CorpusFile>,
    pub fingerprint: RepositoryFingerprint,
}

#[derive(Clone, Debug)]
pub struct CorpusFile {
    pub path: PathBuf,
    pub content: String,
    pub content_hash: [u8; 32],
    pub document: ParsedDocument,
}

impl CanonicalCorpus {
    pub fn load(repository: &Repository, config: &RepositoryConfig) -> Result<Self, CorpusError> {
        Self::load_incremental(repository, config, None)
    }

    pub fn refresh(
        repository: &Repository,
        config: &RepositoryConfig,
        previous: &Self,
    ) -> Result<Self, CorpusError> {
        Self::load_incremental(repository, config, Some(previous))
    }

    pub fn load_at_git_ref(
        repository: &Repository,
        config: &RepositoryConfig,
        reference: &str,
    ) -> Result<Self, CorpusError> {
        let output = Command::new("git")
            .current_dir(repository.root())
            .args(["ls-tree", "-r", "-z", "--name-only", reference, "--"])
            .arg(&config.project.documents.root)
            .output()
            .map_err(CorpusError::GitIo)?;
        if !output.status.success() {
            return Err(CorpusError::Git(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }
        let overrides = document_overrides(repository, config)?;
        let root = &config.project.documents.root;
        let mut contents = Vec::new();
        for raw in output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|raw| !raw.is_empty())
        {
            let text = std::str::from_utf8(raw).map_err(|_| {
                CorpusError::Git("Git returned a non-UTF-8 document path".to_owned())
            })?;
            let path = PathBuf::from(text);
            let relative = path.strip_prefix(root).map_err(|_| {
                CorpusError::Git(format!(
                    "Git returned a path outside the document root: {text}"
                ))
            })?;
            if !overrides.matched(relative, false).is_whitelist() {
                continue;
            }
            let object = format!("{reference}:{text}");
            let content = Command::new("git")
                .current_dir(repository.root())
                .args(["show", &object])
                .output()
                .map_err(CorpusError::GitIo)?;
            if !content.status.success() {
                return Err(CorpusError::Git(
                    String::from_utf8_lossy(&content.stderr).trim().to_owned(),
                ));
            }
            contents.push((
                path,
                String::from_utf8(content.stdout).map_err(|_| {
                    CorpusError::Git(format!(
                        "document {text:?} is not valid UTF-8 at {reference}"
                    ))
                })?,
            ));
        }
        Self::from_contents(repository, contents)
    }

    pub fn from_contents(
        repository: &Repository,
        mut contents: Vec<(PathBuf, String)>,
    ) -> Result<Self, CorpusError> {
        contents.sort_by(|left, right| left.0.cmp(&right.0));
        let mut files = Vec::with_capacity(contents.len());
        for (path, content) in contents {
            let document =
                ParsedDocument::parse(&content).map_err(|source| CorpusError::Markdown {
                    path: repository.root().join(&path),
                    source,
                })?;
            files.push(CorpusFile {
                path,
                content_hash: *blake3::hash(content.as_bytes()).as_bytes(),
                content,
                document,
            });
        }
        let fingerprint = fingerprint(repository, &files)?;
        Ok(Self { files, fingerprint })
    }

    fn load_incremental(
        repository: &Repository,
        config: &RepositoryConfig,
        previous: Option<&Self>,
    ) -> Result<Self, CorpusError> {
        let docs_root = repository.root().join(&config.project.documents.root);
        let docs_root = fs::canonicalize(&docs_root).map_err(|source| CorpusError::Io {
            path: docs_root,
            source,
        })?;
        if !docs_root.starts_with(repository.root()) {
            return Err(CorpusError::OutsideRepository { path: docs_root });
        }
        if !docs_root.is_dir() {
            return Err(CorpusError::NotDirectory { path: docs_root });
        }

        let overrides = document_overrides(repository, config)?;

        let mut paths = Vec::new();
        if !config.project.documents.include.is_empty() {
            let mut walk = WalkBuilder::new(&docs_root);
            walk.hidden(false).overrides(overrides);
            for entry in walk.build() {
                let entry = entry.map_err(CorpusError::Walk)?;
                if entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_file())
                {
                    paths.push(entry.into_path());
                }
            }
        }
        paths.sort();

        let mut files = Vec::with_capacity(paths.len());
        for absolute in paths {
            let content = fs::read_to_string(&absolute).map_err(|source| CorpusError::Io {
                path: absolute.clone(),
                source,
            })?;
            let path = absolute
                .strip_prefix(repository.root())
                .expect("document root is inside the repository")
                .to_path_buf();
            let content_hash = *blake3::hash(content.as_bytes()).as_bytes();
            let document = previous
                .and_then(|corpus| corpus.files.iter().find(|file| file.path == path))
                .filter(|file| file.content_hash == content_hash)
                .map(|file| file.document.clone())
                .map_or_else(
                    || {
                        ParsedDocument::parse(&content).map_err(|source| CorpusError::Markdown {
                            path: absolute.clone(),
                            source,
                        })
                    },
                    Ok,
                )?;
            files.push(CorpusFile {
                path,
                content_hash,
                content,
                document,
            });
        }

        let fingerprint = fingerprint(repository, &files)?;
        Ok(Self { files, fingerprint })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RepositoryFingerprint([u8; 32]);

impl RepositoryFingerprint {
    pub fn from_hex(value: &str) -> Option<Self> {
        let mut bytes = [0_u8; 32];
        if value.len() != 64 {
            return None;
        }
        for (index, chunk) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let text = std::str::from_utf8(chunk).ok()?;
            bytes[index] = u8::from_str_radix(text, 16).ok()?;
        }
        Some(Self(bytes))
    }

    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

impl fmt::Display for RepositoryFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

#[derive(Debug)]
pub enum CorpusError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    OutsideRepository {
        path: PathBuf,
    },
    NotDirectory {
        path: PathBuf,
    },
    Pattern {
        pattern: String,
        source: ignore::Error,
    },
    Walk(ignore::Error),
    Markdown {
        path: PathBuf,
        source: FrontmatterError,
    },
    Git(String),
    GitIo(io::Error),
}

impl fmt::Display for CorpusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::OutsideRepository { path } => write!(
                formatter,
                "document root {} is outside the repository",
                path.display()
            ),
            Self::NotDirectory { path } => {
                write!(
                    formatter,
                    "document root {} is not a directory",
                    path.display()
                )
            }
            Self::Pattern { pattern, source } => {
                write!(formatter, "invalid document pattern {pattern:?}: {source}")
            }
            Self::Walk(source) => write!(formatter, "cannot enumerate documents: {source}"),
            Self::Markdown { path, source } => {
                write!(formatter, "cannot parse {}: {source}", path.display())
            }
            Self::Git(message) => write!(formatter, "cannot read Git corpus: {message}"),
            Self::GitIo(source) => write!(formatter, "cannot execute Git: {source}"),
        }
    }
}

impl Error for CorpusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::GitIo(source) => Some(source),
            Self::Pattern { source, .. } | Self::Walk(source) => Some(source),
            Self::Markdown { source, .. } => Some(source),
            Self::OutsideRepository { .. } | Self::NotDirectory { .. } | Self::Git(_) => None,
        }
    }
}

fn document_overrides(
    repository: &Repository,
    config: &RepositoryConfig,
) -> Result<ignore::overrides::Override, CorpusError> {
    let docs_root = repository.root().join(&config.project.documents.root);
    let mut overrides = OverrideBuilder::new(&docs_root);
    for include in &config.project.documents.include {
        overrides
            .add(include)
            .map_err(|source| CorpusError::Pattern {
                pattern: include.clone(),
                source,
            })?;
    }
    for exclude in &config.project.documents.exclude {
        let pattern = format!("!{exclude}");
        overrides
            .add(&pattern)
            .map_err(|source| CorpusError::Pattern {
                pattern: exclude.clone(),
                source,
            })?;
    }
    overrides.build().map_err(|source| CorpusError::Pattern {
        pattern: "<combined document patterns>".to_owned(),
        source,
    })
}

fn fingerprint(
    repository: &Repository,
    files: &[CorpusFile],
) -> Result<RepositoryFingerprint, CorpusError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FINGERPRINT_DOMAIN);
    hasher.update(&PARSER_REVISION.to_le_bytes());

    let mut inputs = Vec::new();
    for name in [
        "project.toml",
        "entities.toml",
        "relations.toml",
        "workflows.toml",
        "commands.toml",
        "logic.dl",
    ] {
        let absolute = repository.config_dir().join(name);
        match fs::read(&absolute) {
            Ok(content) => inputs.push((PathBuf::from(".docgraph").join(name), content)),
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(CorpusError::Io {
                    path: absolute,
                    source,
                });
            }
        }
    }
    if let Ok(output) = Command::new("git")
        .args(["config", "--get-regexp", r"^remote\..*\.url$"])
        .current_dir(repository.root())
        .output()
        && output.status.success()
        && !output.stdout.is_empty()
    {
        inputs.push((PathBuf::from(".git-remotes"), output.stdout));
    }
    for file in files {
        inputs.push((file.path.clone(), file.content.as_bytes().to_vec()));
    }
    inputs.sort_by(|left, right| left.0.cmp(&right.0));
    for (path, content) in inputs {
        hash_field(&mut hasher, &portable_path(&path));
        hash_field(&mut hasher, &content);
    }
    Ok(RepositoryFingerprint(*hasher.finalize().as_bytes()))
}

fn hash_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn portable_path(path: &Path) -> Vec<u8> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
        .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RepositoryConfig;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "docgraph-corpus-test-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(root.join(".git")).unwrap();
            fs::create_dir_all(root.join(".docgraph")).unwrap();
            fs::create_dir_all(root.join("docs/generated")).unwrap();
            fs::write(
                root.join(".docgraph/project.toml"),
                "schema_version = 1\n[project]\nname = \"fixture\"\n[documents]\nroot = \"docs\"\nexclude = [\"generated/**\"]\n",
            )
            .unwrap();
            fs::write(root.join("docs/a.md"), "# A\n").unwrap();
            fs::write(root.join("docs/generated/ignored.md"), "# Ignored\n").unwrap();
            fs::write(root.join("unrelated.txt"), "one\n").unwrap();
            Self(root)
        }

        fn load(&self) -> (Repository, RepositoryConfig, CanonicalCorpus) {
            let repository = Repository::discover(&self.0).unwrap();
            let config = RepositoryConfig::load(&repository).unwrap();
            let corpus = CanonicalCorpus::load(&repository, &config).unwrap();
            (repository, config, corpus)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn enumerates_only_the_configured_corpus() {
        let fixture = Fixture::new();

        let (_, _, corpus) = fixture.load();

        assert_eq!(corpus.files.len(), 1);
        assert_eq!(corpus.files[0].path, PathBuf::from("docs/a.md"));
    }

    #[test]
    fn fingerprints_only_canonical_inputs() {
        let fixture = Fixture::new();
        let (_, _, first) = fixture.load();

        fs::write(fixture.0.join("unrelated.txt"), "two\n").unwrap();
        let (_, _, unrelated_change) = fixture.load();
        fs::write(fixture.0.join(".docgraph/commands.toml"), "").unwrap();
        let (_, _, command_change) = fixture.load();
        fs::write(fixture.0.join("docs/a.md"), "# Changed\n").unwrap();
        let (_, _, canonical_change) = fixture.load();

        assert_eq!(first.fingerprint, unrelated_change.fingerprint);
        assert_ne!(first.fingerprint, command_change.fingerprint);
        assert_ne!(first.fingerprint, canonical_change.fingerprint);
    }

    #[test]
    fn refresh_reuses_unchanged_parses_by_content_hash() {
        let fixture = Fixture::new();
        let (repository, config, mut first) = fixture.load();
        first.files[0].document.headings[0].title = "cached parse".to_owned();

        let refreshed = CanonicalCorpus::refresh(&repository, &config, &first).unwrap();

        assert_eq!(
            refreshed.files[0].document.headings[0].title,
            "cached parse"
        );
    }

    #[test]
    fn an_explicitly_empty_include_indexes_nothing() {
        let fixture = Fixture::new();
        fs::write(
            fixture.0.join(".docgraph/project.toml"),
            "schema_version = 1\n[project]\nname = \"fixture\"\n[documents]\nroot = \"docs\"\ninclude = []\n",
        )
        .unwrap();

        let (_, _, corpus) = fixture.load();

        assert!(corpus.files.is_empty());
    }
}
