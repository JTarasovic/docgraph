use crate::EmbeddingConfig;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub trait EmbeddingProvider {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
}

pub struct CommandEmbeddingProvider<'a> {
    config: &'a EmbeddingConfig,
}

impl<'a> CommandEmbeddingProvider<'a> {
    pub fn new(config: &'a EmbeddingConfig) -> Self {
        Self { config }
    }
}

#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    texts: &'a [String],
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    vectors: Vec<Vec<f32>>,
}

impl EmbeddingProvider for CommandEmbeddingProvider<'_> {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let (program, arguments) = self
            .config
            .command
            .split_first()
            .ok_or_else(|| EmbeddingError::Protocol("embedding command is empty".to_owned()))?;
        let mut child = Command::new(program)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| EmbeddingError::Unavailable(source.to_string()))?;
        serde_json::to_writer(
            child.stdin.as_mut().expect("piped stdin is available"),
            &EmbeddingRequest {
                model: &self.config.model,
                texts,
            },
        )
        .map_err(|source| EmbeddingError::Protocol(source.to_string()))?;
        let mut stdin = child.stdin.take().expect("piped stdin is available");
        stdin
            .flush()
            .map_err(|source| EmbeddingError::Unavailable(source.to_string()))?;
        drop(stdin);
        let stdout = child.stdout.take().expect("piped stdout is available");
        let stderr = child.stderr.take().expect("piped stderr is available");
        let stdout_reader = thread::spawn(move || read_all(stdout));
        let stderr_reader = thread::spawn(move || read_all(stderr));
        let deadline = Instant::now() + Duration::from_secs(self.config.timeout_seconds);
        let status = loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|source| EmbeddingError::Unavailable(source.to_string()))?
            {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(EmbeddingError::Unavailable(format!(
                    "provider timed out after {} seconds",
                    self.config.timeout_seconds
                )));
            }
            thread::sleep(Duration::from_millis(10));
        };
        let stdout = join_reader(stdout_reader)?;
        let stderr = join_reader(stderr_reader)?;
        if !status.success() {
            let stderr = String::from_utf8_lossy(&stderr).trim().to_owned();
            return Err(EmbeddingError::Unavailable(if stderr.is_empty() {
                format!("provider exited with {status}")
            } else {
                stderr
            }));
        }
        let response: EmbeddingResponse = serde_json::from_slice(&stdout)
            .map_err(|source| EmbeddingError::Protocol(source.to_string()))?;
        if response.vectors.len() != texts.len() {
            return Err(EmbeddingError::Protocol(format!(
                "provider returned {} vectors for {} texts",
                response.vectors.len(),
                texts.len()
            )));
        }
        if let Some(vector) = response
            .vectors
            .iter()
            .find(|vector| vector.len() != self.config.dimensions)
        {
            return Err(EmbeddingError::Protocol(format!(
                "provider returned a {}-dimension vector; expected {}",
                vector.len(),
                self.config.dimensions
            )));
        }
        Ok(response.vectors)
    }
}

fn read_all(mut reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn join_reader(
    reader: thread::JoinHandle<std::io::Result<Vec<u8>>>,
) -> Result<Vec<u8>, EmbeddingError> {
    reader
        .join()
        .map_err(|_| EmbeddingError::Unavailable("provider output reader panicked".to_owned()))?
        .map_err(|source| EmbeddingError::Unavailable(source.to_string()))
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticSearchHit {
    pub node: String,
    pub score: f64,
    pub snippet: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticSearchMode {
    Vector,
    FullTextFallback,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticSearchResult {
    pub mode: SemanticSearchMode,
    pub reason: Option<String>,
    pub hits: Vec<SemanticSearchHit>,
}

#[derive(Debug)]
pub enum EmbeddingError {
    Unavailable(String),
    Protocol(String),
}

impl fmt::Display for EmbeddingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) => {
                write!(formatter, "embedding provider unavailable: {message}")
            }
            Self::Protocol(message) => {
                write!(formatter, "invalid embedding provider response: {message}")
            }
        }
    }
}

impl Error for EmbeddingError {}
