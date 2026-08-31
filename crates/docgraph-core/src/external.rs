use crate::{ExternalSourceConfig, GitReferenceConfig};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, ETAG, IF_NONE_MATCH, USER_AGENT};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalSourceCapabilities {
    pub read: bool,
    pub search: bool,
    pub mutate: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ExternalEntityRecord {
    pub identity: String,
    pub provider: String,
    pub remote_kind: String,
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub state: String,
    pub author: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub url: String,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalFreshness {
    Fresh,
    Stale,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DerivedExternalEntity {
    #[serde(flatten)]
    pub record: ExternalEntityRecord,
    pub fetched_at: u64,
    pub freshness: ExternalFreshness,
    pub provider_version: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExternalEntityView {
    pub identity: String,
    pub record: Option<DerivedExternalEntity>,
    pub fallback: ExternalFallback,
    pub error: Option<ExternalSourceError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalFallback {
    Live,
    Cached,
    Stale,
    IdentityOnly,
    Deleted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSourceErrorKind {
    Unsupported,
    Authentication,
    RateLimited,
    Timeout,
    Unavailable,
    NotFound,
    MalformedResponse,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExternalSourceError {
    pub kind: ExternalSourceErrorKind,
    pub message: String,
}

impl ExternalSourceError {
    fn new(kind: ExternalSourceErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for ExternalSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for ExternalSourceError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExternalReadResult {
    Record {
        record: Box<ExternalEntityRecord>,
        etag: Option<String>,
        provider_version: Option<String>,
    },
    NotModified,
    Deleted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalSearchResult {
    pub records: Vec<ExternalEntityRecord>,
    pub etag: Option<String>,
    pub provider_version: Option<String>,
    pub not_modified: bool,
}

pub trait ExternalEntitySource {
    fn provider(&self) -> &str;
    fn host(&self) -> &str;
    fn capabilities(&self) -> ExternalSourceCapabilities;
    fn read(
        &self,
        identity: &ExternalIdentity,
        etag: Option<&str>,
    ) -> Result<ExternalReadResult, ExternalSourceError>;
    fn search_repository(
        &self,
        repository: &str,
        etag: Option<&str>,
    ) -> Result<ExternalSearchResult, ExternalSourceError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalIdentity {
    pub provider: String,
    pub remote_kind: String,
    pub host: String,
    pub repository: String,
    pub key: String,
}

impl ExternalIdentity {
    pub fn parse(raw: &str) -> Option<Self> {
        let mut parts = raw.splitn(3, ':');
        let provider = parts.next()?;
        let remote_kind = parts.next()?;
        let location = parts.next()?;
        let (location, key) = location.rsplit_once(':')?;
        let (host, repository) = location.split_once('/')?;
        if [provider, remote_kind, host, repository, key]
            .iter()
            .any(|part| part.is_empty())
        {
            return None;
        }
        Some(Self {
            provider: provider.to_owned(),
            remote_kind: remote_kind.to_owned(),
            host: host.to_owned(),
            repository: repository.to_owned(),
            key: key.to_owned(),
        })
    }

    pub fn canonical(&self) -> String {
        format!(
            "{}:{}:{}/{}:{}",
            self.provider, self.remote_kind, self.host, self.repository, self.key
        )
    }
}

pub struct GithubExternalEntitySource {
    host: String,
    api_base: String,
    token: Option<String>,
    credential_configured: bool,
    client: Client,
}

impl GithubExternalEntitySource {
    pub fn new(config: &ExternalSourceConfig) -> Result<Self, ExternalSourceError> {
        let credential_configured = config.token_env.is_some() || !config.token_command.is_empty();
        let token = config
            .token_env
            .as_deref()
            .and_then(|name| std::env::var(name).ok())
            .filter(|value| !value.trim().is_empty())
            .or_else(|| credential_command(&config.token_command));
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|error| {
                ExternalSourceError::new(
                    ExternalSourceErrorKind::Unavailable,
                    format!("cannot initialize GitHub client: {error}"),
                )
            })?;
        let api_base = config.api_url.clone().unwrap_or_else(|| {
            if config.host.eq_ignore_ascii_case("github.com") {
                "https://api.github.com".to_owned()
            } else {
                format!("https://{}/api/v3", config.host)
            }
        });
        Ok(Self {
            host: config.host.clone(),
            api_base: api_base.trim_end_matches('/').to_owned(),
            token,
            credential_configured,
            client,
        })
    }

    fn request(
        &self,
        url: &str,
        etag: Option<&str>,
    ) -> Result<reqwest::blocking::Response, ExternalSourceError> {
        let mut request = self
            .client
            .get(url)
            .header(USER_AGENT, "docgraph-external-entity-source")
            .header(ACCEPT, "application/vnd.github+json");
        if let Some(token) = &self.token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        if let Some(etag) = etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        request.send().map_err(map_transport_error)
    }
}

fn credential_command(command: &[String]) -> Option<String> {
    let (program, arguments) = command.split_first()?;
    let output = Command::new(program).args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

impl ExternalEntitySource for GithubExternalEntitySource {
    fn provider(&self) -> &str {
        "github"
    }

    fn host(&self) -> &str {
        &self.host
    }

    fn capabilities(&self) -> ExternalSourceCapabilities {
        ExternalSourceCapabilities {
            read: true,
            search: true,
            mutate: false,
        }
    }

    fn read(
        &self,
        identity: &ExternalIdentity,
        etag: Option<&str>,
    ) -> Result<ExternalReadResult, ExternalSourceError> {
        if identity.provider != "github"
            || !identity.host.eq_ignore_ascii_case(&self.host)
            || identity.remote_kind != "issue"
        {
            return Err(ExternalSourceError::new(
                ExternalSourceErrorKind::Unsupported,
                format!("GitHub source cannot read {}", identity.canonical()),
            ));
        }
        let url = format!(
            "{}/repos/{}/issues/{}",
            self.api_base, identity.repository, identity.key
        );
        let response = self.request(&url, etag)?;
        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(ExternalReadResult::NotModified);
        }
        if response.status() == StatusCode::NOT_FOUND {
            if self.credential_configured && self.token.is_none() {
                return Err(ExternalSourceError::new(
                    ExternalSourceErrorKind::Authentication,
                    "configured GitHub credentials are unavailable",
                ));
            }
            return Ok(ExternalReadResult::Deleted);
        }
        let response = checked_response(response)?;
        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let version = github_version(&response);
        let issue: GithubIssue = response.json().map_err(|error| {
            ExternalSourceError::new(
                ExternalSourceErrorKind::MalformedResponse,
                format!("GitHub returned an invalid issue record: {error}"),
            )
        })?;
        Ok(ExternalReadResult::Record {
            record: Box::new(issue.into_record(&self.host, &identity.repository)),
            etag,
            provider_version: version,
        })
    }

    fn search_repository(
        &self,
        repository: &str,
        etag: Option<&str>,
    ) -> Result<ExternalSearchResult, ExternalSourceError> {
        let url = format!(
            "{}/repos/{repository}/issues?state=all&per_page=100&sort=updated&direction=desc",
            self.api_base
        );
        let response = self.request(&url, etag)?;
        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(ExternalSearchResult {
                records: Vec::new(),
                etag: etag.map(str::to_owned),
                provider_version: None,
                not_modified: true,
            });
        }
        if response.status() == StatusCode::NOT_FOUND
            && self.credential_configured
            && self.token.is_none()
        {
            return Err(ExternalSourceError::new(
                ExternalSourceErrorKind::Authentication,
                "configured GitHub credentials are unavailable",
            ));
        }
        let response = checked_response(response)?;
        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let version = github_version(&response);
        let issues: Vec<GithubIssue> = response.json().map_err(|error| {
            ExternalSourceError::new(
                ExternalSourceErrorKind::MalformedResponse,
                format!("GitHub returned an invalid issue list: {error}"),
            )
        })?;
        Ok(ExternalSearchResult {
            records: issues
                .into_iter()
                .map(|issue| issue.into_record(&self.host, repository))
                .collect(),
            etag,
            provider_version: version,
            not_modified: false,
        })
    }
}

#[derive(Deserialize)]
struct GithubIssue {
    number: u64,
    title: String,
    #[serde(default)]
    body: Option<String>,
    state: String,
    html_url: String,
    user: Option<GithubUser>,
    created_at: Option<String>,
    updated_at: Option<String>,
    #[serde(default)]
    labels: Vec<GithubLabel>,
    #[serde(default)]
    assignees: Vec<GithubUser>,
    pull_request: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct GithubUser {
    login: String,
}

#[derive(Deserialize)]
struct GithubLabel {
    name: String,
}

impl GithubIssue {
    fn into_record(self, host: &str, repository: &str) -> ExternalEntityRecord {
        let remote_kind = if self.pull_request.is_some() {
            "pull_request"
        } else {
            "issue"
        };
        let mut attributes = BTreeMap::new();
        attributes.insert("host".to_owned(), host.to_owned());
        attributes.insert("repository".to_owned(), repository.to_owned());
        attributes.insert(
            "labels".to_owned(),
            self.labels
                .into_iter()
                .map(|label| label.name)
                .collect::<Vec<_>>()
                .join(","),
        );
        attributes.insert(
            "assignees".to_owned(),
            self.assignees
                .into_iter()
                .map(|user| user.login)
                .collect::<Vec<_>>()
                .join(","),
        );
        ExternalEntityRecord {
            identity: format!("github:{remote_kind}:{host}/{repository}:{}", self.number),
            provider: "github".to_owned(),
            remote_kind: remote_kind.to_owned(),
            title: self.title,
            body: self.body.unwrap_or_default(),
            state: self.state,
            author: self.user.map(|user| user.login),
            created_at: self.created_at,
            updated_at: self.updated_at,
            url: self.html_url,
            attributes,
        }
    }
}

fn github_version(response: &reqwest::blocking::Response) -> Option<String> {
    response
        .headers()
        .get("x-github-api-version-selected")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn map_transport_error(error: reqwest::Error) -> ExternalSourceError {
    let kind = if error.is_timeout() {
        ExternalSourceErrorKind::Timeout
    } else {
        ExternalSourceErrorKind::Unavailable
    };
    ExternalSourceError::new(kind, format!("external source request failed: {error}"))
}

fn checked_response(
    response: reqwest::blocking::Response,
) -> Result<reqwest::blocking::Response, ExternalSourceError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let kind = match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            if response
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|value| value.to_str().ok())
                == Some("0")
            {
                ExternalSourceErrorKind::RateLimited
            } else {
                ExternalSourceErrorKind::Authentication
            }
        }
        StatusCode::TOO_MANY_REQUESTS => ExternalSourceErrorKind::RateLimited,
        StatusCode::NOT_FOUND => ExternalSourceErrorKind::NotFound,
        _ => ExternalSourceErrorKind::Unavailable,
    };
    Err(ExternalSourceError::new(
        kind,
        format!("external source returned HTTP {status}"),
    ))
}

const EXTERNAL_SCHEMA: &str = r#"
PRAGMA user_version = 1;
CREATE TABLE IF NOT EXISTS external_records (
    identity TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    remote_kind TEXT NOT NULL,
    payload TEXT NOT NULL,
    fetched_at INTEGER NOT NULL,
    provider_version TEXT,
    etag TEXT
) STRICT;
CREATE TABLE IF NOT EXISTS external_queries (
    provider TEXT NOT NULL,
    host TEXT NOT NULL,
    repository TEXT NOT NULL,
    fetched_at INTEGER NOT NULL,
    provider_version TEXT,
    etag TEXT,
    identities TEXT NOT NULL,
    PRIMARY KEY(provider, host, repository)
) STRICT;
"#;

#[derive(Clone, Debug)]
pub struct ExternalEntityCache {
    path: PathBuf,
}

impl ExternalEntityCache {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn connection(&self) -> Result<Connection, ExternalCacheError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ExternalCacheError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let connection =
            Connection::open(&self.path).map_err(|source| ExternalCacheError::Sqlite {
                path: self.path.clone(),
                source,
            })?;
        connection
            .execute_batch(EXTERNAL_SCHEMA)
            .map_err(|source| ExternalCacheError::Sqlite {
                path: self.path.clone(),
                source,
            })?;
        Ok(connection)
    }

    pub fn get(
        &self,
        identity: &str,
        now: u64,
        ttl_seconds: u64,
    ) -> Result<Option<DerivedExternalEntity>, ExternalCacheError> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT payload, fetched_at, provider_version FROM external_records WHERE identity = ?1",
                [identity],
                |row| {
                    let payload: String = row.get(0)?;
                    let fetched_at = row.get::<_, i64>(1)? as u64;
                    let record = serde_json::from_str(&payload).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            payload.len(),
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                    Ok(DerivedExternalEntity {
                        record,
                        fetched_at,
                        freshness: freshness(now, fetched_at, ttl_seconds),
                        provider_version: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(|source| ExternalCacheError::Sqlite {
                path: self.path.clone(),
                source,
            })
    }

    pub fn all(
        &self,
        now: u64,
        ttl_seconds: u64,
    ) -> Result<Vec<DerivedExternalEntity>, ExternalCacheError> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare("SELECT payload, fetched_at, provider_version FROM external_records ORDER BY identity")
            .map_err(|source| ExternalCacheError::Sqlite { path: self.path.clone(), source })?;
        let rows = statement
            .query_map([], |row| {
                let payload: String = row.get(0)?;
                let fetched_at = row.get::<_, i64>(1)? as u64;
                let record = serde_json::from_str(&payload).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        payload.len(),
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(DerivedExternalEntity {
                    record,
                    fetched_at,
                    freshness: freshness(now, fetched_at, ttl_seconds),
                    provider_version: row.get(2)?,
                })
            })
            .map_err(|source| ExternalCacheError::Sqlite {
                path: self.path.clone(),
                source,
            })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|source| ExternalCacheError::Sqlite {
                path: self.path.clone(),
                source,
            })
    }

    fn etag(&self, identity: &str) -> Result<Option<String>, ExternalCacheError> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT etag FROM external_records WHERE identity = ?1",
                [identity],
                |row| row.get(0),
            )
            .optional()
            .map(|value| value.flatten())
            .map_err(|source| ExternalCacheError::Sqlite {
                path: self.path.clone(),
                source,
            })
    }

    fn put(
        &self,
        record: &ExternalEntityRecord,
        fetched_at: u64,
        provider_version: Option<&str>,
        etag: Option<&str>,
    ) -> Result<(), ExternalCacheError> {
        let payload = serde_json::to_string(record).expect("external records serialize");
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT OR REPLACE INTO external_records(identity, provider, remote_kind, payload, fetched_at, provider_version, etag) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![record.identity, record.provider, record.remote_kind, payload, fetched_at as i64, provider_version, etag],
            )
            .map_err(|source| ExternalCacheError::Sqlite { path: self.path.clone(), source })?;
        Ok(())
    }

    fn touch(&self, identity: &str, fetched_at: u64) -> Result<(), ExternalCacheError> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE external_records SET fetched_at = ?2 WHERE identity = ?1",
                params![identity, fetched_at as i64],
            )
            .map_err(|source| ExternalCacheError::Sqlite {
                path: self.path.clone(),
                source,
            })?;
        Ok(())
    }

    fn delete(&self, identity: &str) -> Result<(), ExternalCacheError> {
        let connection = self.connection()?;
        connection
            .execute(
                "DELETE FROM external_records WHERE identity = ?1",
                [identity],
            )
            .map_err(|source| ExternalCacheError::Sqlite {
                path: self.path.clone(),
                source,
            })?;
        Ok(())
    }

    fn query_metadata(
        &self,
        reference: &GitReferenceConfig,
    ) -> Result<Option<QueryCacheMetadata>, ExternalCacheError> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT fetched_at, provider_version, etag, identities FROM external_queries WHERE provider = ?1 AND host = ?2 AND repository = ?3",
                params![reference.provider, reference.host, reference.repository],
                |row| {
                    let identities: String = row.get(3)?;
                    Ok(QueryCacheMetadata {
                        fetched_at: row.get::<_, i64>(0)? as u64,
                        provider_version: row.get(1)?,
                        etag: row.get(2)?,
                        identities: serde_json::from_str(&identities).map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                identities.len(),
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })?,
                    })
                },
            )
            .optional()
            .map_err(|source| ExternalCacheError::Sqlite { path: self.path.clone(), source })
    }

    fn put_query(
        &self,
        reference: &GitReferenceConfig,
        metadata: &QueryCacheMetadata,
    ) -> Result<(), ExternalCacheError> {
        let identities = serde_json::to_string(&metadata.identities).expect("identities serialize");
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT OR REPLACE INTO external_queries(provider, host, repository, fetched_at, provider_version, etag, identities) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![reference.provider, reference.host, reference.repository, metadata.fetched_at as i64, metadata.provider_version, metadata.etag, identities],
            )
            .map_err(|source| ExternalCacheError::Sqlite { path: self.path.clone(), source })?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct QueryCacheMetadata {
    fetched_at: u64,
    provider_version: Option<String>,
    etag: Option<String>,
    identities: Vec<String>,
}

#[derive(Debug)]
pub enum ExternalCacheError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Sqlite {
        path: PathBuf,
        source: rusqlite::Error,
    },
}

impl fmt::Display for ExternalCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "cannot access {}: {source}", path.display())
            }
            Self::Sqlite { path, source } => write!(
                formatter,
                "cannot use external cache {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for ExternalCacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Sqlite { source, .. } => Some(source),
        }
    }
}

pub struct ExternalEntityService {
    cache: ExternalEntityCache,
    ttl_seconds: u64,
}

impl ExternalEntityService {
    pub fn new(cache: ExternalEntityCache, ttl_seconds: u64) -> Self {
        Self { cache, ttl_seconds }
    }

    pub fn cached(
        &self,
        now: SystemTime,
    ) -> Result<Vec<DerivedExternalEntity>, ExternalCacheError> {
        self.cache.all(epoch_seconds(now), self.ttl_seconds)
    }

    pub fn resolve(
        &self,
        identity: &str,
        source: Option<&dyn ExternalEntitySource>,
        now: SystemTime,
    ) -> ExternalEntityView {
        let now = epoch_seconds(now);
        let cached = self
            .cache
            .get(identity, now, self.ttl_seconds)
            .ok()
            .flatten();
        if cached
            .as_ref()
            .is_some_and(|record| record.freshness == ExternalFreshness::Fresh)
        {
            return ExternalEntityView {
                identity: identity.to_owned(),
                record: cached,
                fallback: ExternalFallback::Cached,
                error: None,
            };
        }
        let Some(source) = source else {
            return cached.map_or_else(
                || identity_only(identity, None),
                |record| stale_view(identity, record, None),
            );
        };
        let Some(parsed) = ExternalIdentity::parse(identity) else {
            return identity_only(
                identity,
                Some(ExternalSourceError::new(
                    ExternalSourceErrorKind::Unsupported,
                    "identity is not a supported external entity identity",
                )),
            );
        };
        let etag = self.cache.etag(identity).ok().flatten();
        match source.read(&parsed, etag.as_deref()) {
            Ok(ExternalReadResult::Record {
                record,
                etag,
                provider_version,
            }) => {
                if let Err(error) =
                    self.cache
                        .put(&record, now, provider_version.as_deref(), etag.as_deref())
                {
                    let error = cache_source_error(error);
                    return match cached {
                        Some(record) => stale_view(identity, record, Some(error)),
                        None => identity_only(identity, Some(error)),
                    };
                }
                ExternalEntityView {
                    identity: identity.to_owned(),
                    record: self
                        .cache
                        .get(identity, now, self.ttl_seconds)
                        .ok()
                        .flatten(),
                    fallback: ExternalFallback::Live,
                    error: None,
                }
            }
            Ok(ExternalReadResult::NotModified) => {
                let _ = self.cache.touch(identity, now);
                ExternalEntityView {
                    identity: identity.to_owned(),
                    record: self
                        .cache
                        .get(identity, now, self.ttl_seconds)
                        .ok()
                        .flatten(),
                    fallback: ExternalFallback::Cached,
                    error: None,
                }
            }
            Ok(ExternalReadResult::Deleted) => {
                let _ = self.cache.delete(identity);
                ExternalEntityView {
                    identity: identity.to_owned(),
                    record: None,
                    fallback: ExternalFallback::Deleted,
                    error: None,
                }
            }
            Err(error) => match cached {
                Some(record) => stale_view(identity, record, Some(error)),
                None => identity_only(identity, Some(error)),
            },
        }
    }

    pub fn refresh_repository(
        &self,
        reference: &GitReferenceConfig,
        source: &dyn ExternalEntitySource,
        now: SystemTime,
    ) -> Result<Vec<DerivedExternalEntity>, ExternalSourceError> {
        let now = epoch_seconds(now);
        let previous = self
            .cache
            .query_metadata(reference)
            .map_err(cache_source_error)?;
        if previous.as_ref().is_some_and(|metadata| {
            freshness(now, metadata.fetched_at, self.ttl_seconds) == ExternalFreshness::Fresh
        }) {
            let metadata = previous.expect("fresh metadata exists");
            for identity in &metadata.identities {
                self.cache
                    .touch(identity, metadata.fetched_at)
                    .map_err(cache_source_error)?;
            }
            return self.records_for_identities(&metadata.identities, now);
        }
        let result = source.search_repository(
            &reference.repository,
            previous.as_ref().and_then(|value| value.etag.as_deref()),
        );
        match result {
            Ok(result) if result.not_modified => {
                let mut metadata = previous.unwrap_or(QueryCacheMetadata {
                    fetched_at: now,
                    provider_version: result.provider_version,
                    etag: result.etag,
                    identities: Vec::new(),
                });
                metadata.fetched_at = now;
                for identity in &metadata.identities {
                    self.cache
                        .touch(identity, now)
                        .map_err(cache_source_error)?;
                }
                self.cache
                    .put_query(reference, &metadata)
                    .map_err(cache_source_error)?;
                self.records_for_identities(&metadata.identities, now)
            }
            Ok(result) => {
                let identities: Vec<_> = result
                    .records
                    .iter()
                    .map(|record| record.identity.clone())
                    .collect();
                for record in &result.records {
                    self.cache
                        .put(record, now, result.provider_version.as_deref(), None)
                        .map_err(cache_source_error)?;
                }
                if let Some(previous) = &previous {
                    for removed in previous
                        .identities
                        .iter()
                        .filter(|id| !identities.contains(id))
                    {
                        self.cache.delete(removed).map_err(cache_source_error)?;
                    }
                }
                let metadata = QueryCacheMetadata {
                    fetched_at: now,
                    provider_version: result.provider_version,
                    etag: result.etag,
                    identities: identities.clone(),
                };
                self.cache
                    .put_query(reference, &metadata)
                    .map_err(cache_source_error)?;
                self.records_for_identities(&identities, now)
            }
            Err(error) => {
                if let Some(previous) = previous {
                    self.records_for_identities(&previous.identities, now)
                } else {
                    Err(error)
                }
            }
        }
    }

    fn records_for_identities(
        &self,
        identities: &[String],
        now: u64,
    ) -> Result<Vec<DerivedExternalEntity>, ExternalSourceError> {
        identities
            .iter()
            .filter_map(|identity| self.cache.get(identity, now, self.ttl_seconds).transpose())
            .collect::<Result<Vec<_>, _>>()
            .map_err(cache_source_error)
    }
}

fn identity_only(identity: &str, error: Option<ExternalSourceError>) -> ExternalEntityView {
    ExternalEntityView {
        identity: identity.to_owned(),
        record: None,
        fallback: ExternalFallback::IdentityOnly,
        error,
    }
}

fn stale_view(
    identity: &str,
    mut record: DerivedExternalEntity,
    error: Option<ExternalSourceError>,
) -> ExternalEntityView {
    record.freshness = ExternalFreshness::Stale;
    ExternalEntityView {
        identity: identity.to_owned(),
        record: Some(record),
        fallback: ExternalFallback::Stale,
        error,
    }
}

fn cache_source_error(error: ExternalCacheError) -> ExternalSourceError {
    ExternalSourceError::new(ExternalSourceErrorKind::Unavailable, error.to_string())
}

fn freshness(now: u64, fetched_at: u64, ttl_seconds: u64) -> ExternalFreshness {
    if now.saturating_sub(fetched_at) <= ttl_seconds {
        ExternalFreshness::Fresh
    } else {
        ExternalFreshness::Stale
    }
}

fn epoch_seconds(now: SystemTime) -> u64 {
    now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::mpsc::{self, Receiver};
    use std::thread;

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct FakeSource {
        result: RefCell<Result<ExternalReadResult, ExternalSourceError>>,
    }

    impl ExternalEntitySource for FakeSource {
        fn provider(&self) -> &str {
            "fake"
        }
        fn host(&self) -> &str {
            "example.test"
        }
        fn capabilities(&self) -> ExternalSourceCapabilities {
            ExternalSourceCapabilities {
                read: true,
                search: false,
                mutate: false,
            }
        }
        fn read(
            &self,
            _: &ExternalIdentity,
            _: Option<&str>,
        ) -> Result<ExternalReadResult, ExternalSourceError> {
            self.result.borrow().clone()
        }
        fn search_repository(
            &self,
            _: &str,
            _: Option<&str>,
        ) -> Result<ExternalSearchResult, ExternalSourceError> {
            Err(ExternalSourceError::new(
                ExternalSourceErrorKind::Unsupported,
                "search unsupported",
            ))
        }
    }

    fn record() -> ExternalEntityRecord {
        ExternalEntityRecord {
            identity: "fake:issue:example.test/owner/repo:7".to_owned(),
            provider: "fake".to_owned(),
            remote_kind: "issue".to_owned(),
            title: "Remote issue".to_owned(),
            body: "Details".to_owned(),
            state: "open".to_owned(),
            author: Some("octo".to_owned()),
            created_at: None,
            updated_at: None,
            url: "https://example.test/owner/repo/issues/7".to_owned(),
            attributes: BTreeMap::new(),
        }
    }

    fn service() -> (PathBuf, ExternalEntityService) {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "docgraph-external-test-{}-{sequence}",
            std::process::id()
        ));
        let service =
            ExternalEntityService::new(ExternalEntityCache::new(root.join("external.sqlite")), 60);
        (root, service)
    }

    fn source_config(api_url: String) -> ExternalSourceConfig {
        ExternalSourceConfig {
            provider: "github".to_owned(),
            host: "github.test".to_owned(),
            api_url: Some(api_url),
            token_env: None,
            token_command: Vec::new(),
            timeout_seconds: 2,
        }
    }

    fn respond_once(
        status: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> (String, Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (sender, receiver) = mpsc::channel();
        let status = status.to_owned();
        let headers: Vec<_> = headers
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();
        let body = body.to_owned();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 1024];
            loop {
                let count = stream.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..count]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let _ = sender.send(String::from_utf8(request).unwrap());
            let mut response = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
                body.len()
            );
            for (name, value) in headers {
                response.push_str(&format!("{name}: {value}\r\n"));
            }
            response.push_str("\r\n");
            response.push_str(&body);
            stream.write_all(response.as_bytes()).unwrap();
        });
        (format!("http://{address}"), receiver)
    }

    #[test]
    fn identity_parsing_is_offline_and_deterministic() {
        let identity = ExternalIdentity::parse("github:issue:github.com/owner/repo:123").unwrap();
        assert_eq!(identity.provider, "github");
        assert_eq!(identity.repository, "owner/repo");
        assert_eq!(
            identity.canonical(),
            "github:issue:github.com/owner/repo:123"
        );
    }

    #[test]
    fn live_cached_stale_and_identity_only_fallbacks_are_distinct() {
        let (root, service) = service();
        let source = FakeSource {
            result: RefCell::new(Ok(ExternalReadResult::Record {
                record: Box::new(record()),
                etag: Some("v1".to_owned()),
                provider_version: Some("test-v1".to_owned()),
            })),
        };
        let at = UNIX_EPOCH + Duration::from_secs(100);
        let live = service.resolve(&record().identity, Some(&source), at);
        assert_eq!(live.fallback, ExternalFallback::Live);
        let cached = service.resolve(&record().identity, None, at + Duration::from_secs(30));
        assert_eq!(cached.fallback, ExternalFallback::Cached);
        let stale = service.resolve(&record().identity, None, at + Duration::from_secs(61));
        assert_eq!(stale.fallback, ExternalFallback::Stale);
        let missing = service.resolve("fake:issue:example.test/owner/repo:8", None, at);
        assert_eq!(missing.fallback, ExternalFallback::IdentityOnly);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn source_failures_preserve_stale_cached_records() {
        let (root, service) = service();
        let source = FakeSource {
            result: RefCell::new(Ok(ExternalReadResult::Record {
                record: Box::new(record()),
                etag: None,
                provider_version: None,
            })),
        };
        let at = UNIX_EPOCH + Duration::from_secs(100);
        service.resolve(&record().identity, Some(&source), at);
        *source.result.borrow_mut() = Err(ExternalSourceError::new(
            ExternalSourceErrorKind::Timeout,
            "timed out",
        ));
        let stale = service.resolve(
            &record().identity,
            Some(&source),
            at + Duration::from_secs(61),
        );
        assert_eq!(stale.fallback, ExternalFallback::Stale);
        assert_eq!(stale.error.unwrap().kind, ExternalSourceErrorKind::Timeout);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn github_source_normalizes_public_issue_reads_and_conditional_results() {
        let body = r#"{
            "number": 7,
            "title": "Remote issue",
            "body": "Details",
            "state": "open",
            "html_url": "https://github.test/owner/repo/issues/7",
            "user": {"login": "octo"},
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z",
            "labels": [{"name": "bug"}],
            "assignees": [{"login": "hubot"}],
            "pull_request": null
        }"#;
        let (api_url, request) = respond_once(
            "200 OK",
            &[
                ("ETag", "issue-v1"),
                ("X-GitHub-Api-Version-Selected", "2022-11-28"),
            ],
            body,
        );
        let source = GithubExternalEntitySource::new(&source_config(api_url)).unwrap();
        let identity = ExternalIdentity::parse("github:issue:github.test/owner/repo:7").unwrap();

        let result = source.read(&identity, None).unwrap();

        let ExternalReadResult::Record {
            record,
            etag,
            provider_version,
        } = result
        else {
            panic!("expected a record");
        };
        assert_eq!(record.identity, identity.canonical());
        assert_eq!(record.attributes["labels"], "bug");
        assert_eq!(record.attributes["assignees"], "hubot");
        assert_eq!(etag.as_deref(), Some("issue-v1"));
        assert_eq!(provider_version.as_deref(), Some("2022-11-28"));
        let request = request.recv().unwrap();
        assert!(request.starts_with("GET /repos/owner/repo/issues/7 "));
        assert!(!request.to_ascii_lowercase().contains("authorization:"));

        let (api_url, request) = respond_once("304 Not Modified", &[], "");
        let source = GithubExternalEntitySource::new(&source_config(api_url)).unwrap();
        assert_eq!(
            source.read(&identity, Some("issue-v1")).unwrap(),
            ExternalReadResult::NotModified
        );
        assert!(request.recv().unwrap().contains("if-none-match: issue-v1"));
    }

    #[test]
    fn github_repository_search_distinguishes_pull_requests() {
        let body = r#"[{
            "number": 4,
            "title": "Change",
            "body": null,
            "state": "closed",
            "html_url": "https://github.test/owner/repo/pull/4",
            "user": {"login": "octo"},
            "created_at": null,
            "updated_at": null,
            "labels": [],
            "assignees": [],
            "pull_request": {"url": "https://api.github.test/pulls/4"}
        }]"#;
        let (api_url, _) = respond_once("200 OK", &[], body);
        let source = GithubExternalEntitySource::new(&source_config(api_url)).unwrap();

        let result = source.search_repository("owner/repo", None).unwrap();

        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].remote_kind, "pull_request");
        assert_eq!(
            result.records[0].identity,
            "github:pull_request:github.test/owner/repo:4"
        );
    }

    #[test]
    fn github_source_classifies_failures_and_confirmed_deletion() {
        for (status, headers, expected) in [
            (
                "401 Unauthorized",
                vec![],
                ExternalSourceErrorKind::Authentication,
            ),
            (
                "403 Forbidden",
                vec![("X-RateLimit-Remaining", "0")],
                ExternalSourceErrorKind::RateLimited,
            ),
            (
                "429 Too Many Requests",
                vec![],
                ExternalSourceErrorKind::RateLimited,
            ),
            (
                "500 Internal Server Error",
                vec![],
                ExternalSourceErrorKind::Unavailable,
            ),
        ] {
            let (api_url, _) = respond_once(status, &headers, "{}");
            let source = GithubExternalEntitySource::new(&source_config(api_url)).unwrap();
            let error = source.search_repository("owner/repo", None).unwrap_err();
            assert_eq!(error.kind, expected);
        }

        let (api_url, _) = respond_once("404 Not Found", &[], "{}");
        let source = GithubExternalEntitySource::new(&source_config(api_url)).unwrap();
        let identity = ExternalIdentity::parse("github:issue:github.test/owner/repo:7").unwrap();
        assert_eq!(
            source.read(&identity, None).unwrap(),
            ExternalReadResult::Deleted
        );

        let (api_url, _) = respond_once("404 Not Found", &[], "{}");
        let mut config = source_config(api_url);
        config.token_env = Some("DOCGRAPH_TEST_MISSING_GITHUB_TOKEN".to_owned());
        let source = GithubExternalEntitySource::new(&config).unwrap();
        let error = source.read(&identity, None).unwrap_err();
        assert_eq!(error.kind, ExternalSourceErrorKind::Authentication);

        let (api_url, _) = respond_once("200 OK", &[], "{");
        let source = GithubExternalEntitySource::new(&source_config(api_url)).unwrap();
        let error = source.search_repository("owner/repo", None).unwrap_err();
        assert_eq!(error.kind, ExternalSourceErrorKind::MalformedResponse);
    }
}
