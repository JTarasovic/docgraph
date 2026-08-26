use crate::document::ParsedDocument;
use crate::frontmatter::{anchor_on_line, list_markers, structural_prefix_len};
use crate::{SourceSpan, StableSectionId};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

const TOKEN_LENGTH: usize = 10;
const MAX_GENERATION_ATTEMPTS: usize = 128;
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Normalization {
    pub content: String,
    pub inserted: Vec<SectionInsertion>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SectionInsertion {
    pub id: StableSectionId,
    pub before: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizeError {
    Parse(String),
    Generator(String),
    InvalidGeneratedToken(String),
    Exhausted,
}

impl fmt::Display for NormalizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(message) => write!(formatter, "cannot parse document: {message}"),
            Self::Generator(message) => {
                write!(formatter, "cannot generate a section ID: {message}")
            }
            Self::InvalidGeneratedToken(token) => {
                write!(
                    formatter,
                    "section ID generator returned invalid token {token:?}"
                )
            }
            Self::Exhausted => formatter.write_str("could not generate a unique section ID"),
        }
    }
}

impl Error for NormalizeError {}

pub fn normalize_sections(source: &str) -> Result<Normalization, NormalizeError> {
    normalize_sections_with_reserved_random(source, std::iter::empty())
}

pub fn normalize_sections_with_reserved_random(
    source: &str,
    reserved: impl IntoIterator<Item = StableSectionId>,
) -> Result<Normalization, NormalizeError> {
    normalize_sections_with_reserved(source, reserved, || {
        let mut bytes = [0_u8; TOKEN_LENGTH];
        getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
        Ok(bytes
            .into_iter()
            .map(|byte| CROCKFORD[(byte & 31) as usize] as char)
            .collect())
    })
}

pub fn normalize_sections_with(
    source: &str,
    generate_token: impl FnMut() -> Result<String, String>,
) -> Result<Normalization, NormalizeError> {
    normalize_sections_with_reserved(source, std::iter::empty(), generate_token)
}

pub fn normalize_sections_with_reserved(
    source: &str,
    reserved: impl IntoIterator<Item = StableSectionId>,
    mut generate_token: impl FnMut() -> Result<String, String>,
) -> Result<Normalization, NormalizeError> {
    let document =
        ParsedDocument::parse(source).map_err(|error| NormalizeError::Parse(error.to_string()))?;
    let mut used = collect_stable_ids(source);
    used.extend(reserved);
    let mut edits = Vec::new();
    let mut inserted = Vec::new();

    for heading in document
        .headings
        .iter()
        .filter(|heading| heading.id.is_none())
    {
        let id = (0..MAX_GENERATION_ATTEMPTS)
            .find_map(|_| {
                let token = match generate_token() {
                    Ok(token) => token,
                    Err(message) => return Some(Err(NormalizeError::Generator(message))),
                };
                let candidate = format!("s-{token}");
                let Some(id) = StableSectionId::parse(&candidate) else {
                    return Some(Err(NormalizeError::InvalidGeneratedToken(token)));
                };
                if used.insert(id.clone()) {
                    Some(Ok(id))
                } else {
                    None
                }
            })
            .transpose()?
            .ok_or(NormalizeError::Exhausted)?;
        let offset = line_start(source, heading.heading_span.bytes.start);
        let (prefix_end, replacement) = insertion(source, offset, &id);
        edits.push((offset, prefix_end, replacement));
        inserted.push(SectionInsertion {
            id,
            before: SourceSpan::new(source, offset..offset),
        });
    }

    let mut content = source.to_owned();
    for (start, end, replacement) in edits.into_iter().rev() {
        content.replace_range(start..end, &replacement);
    }
    Ok(Normalization { content, inserted })
}

fn collect_stable_ids(source: &str) -> BTreeSet<StableSectionId> {
    source
        .lines()
        .filter_map(|line| anchor_on_line(line).map(|(id, _)| id))
        .filter_map(StableSectionId::parse)
        .collect()
}

fn insertion(source: &str, line_start: usize, id: &StableSectionId) -> (usize, String) {
    let line_end = source[line_start..]
        .find(['\r', '\n'])
        .map_or(source.len(), |relative| line_start + relative);
    let line = &source[line_start..line_end];
    let prefix_len = structural_prefix_len(line);
    let prefix = &line[..prefix_len];
    let continuation = continuation_prefix(prefix);
    let newline = if source[line_end..].starts_with("\r\n") {
        "\r\n"
    } else if source[line_end..].starts_with('\n') {
        "\n"
    } else if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    (
        line_start + prefix_len,
        format!("{prefix}<a id=\"{id}\"></a>{newline}{continuation}"),
    )
}

fn continuation_prefix(prefix: &str) -> String {
    let mut continuation = prefix.to_owned();
    for marker in list_markers(prefix).into_iter().rev() {
        continuation.replace_range(marker.clone(), &" ".repeat(marker.len()));
    }
    continuation
}

fn line_start(source: &str, offset: usize) -> usize {
    source[..offset]
        .rfind('\n')
        .map_or(0, |newline| newline + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens<'a>(values: &'a [&'a str]) -> impl FnMut() -> Result<String, String> + 'a {
        let mut values = values.iter();
        move || Ok(values.next().unwrap().to_string())
    }

    #[test]
    fn inserts_missing_ids_and_is_idempotent() {
        let source = "# One\n\n<a id=\"s-7K3M9Q2W\"></a>\n## Two\n";

        let normalized = normalize_sections_with(source, tokens(&["83JRT4K2P6"])).unwrap();

        assert_eq!(
            normalized.content,
            "<a id=\"s-83JRT4K2P6\"></a>\n# One\n\n<a id=\"s-7K3M9Q2W\"></a>\n## Two\n"
        );
        assert_eq!(normalized.inserted.len(), 1);
        let second = normalize_sections_with(&normalized.content, tokens(&[])).unwrap();
        assert!(second.inserted.is_empty());
        assert_eq!(second.content, normalized.content);
    }

    #[test]
    fn preserves_block_quotes_and_list_structure() {
        let source = "> ## Quoted\n\n- ## Listed\n\n- item\n  ### Nested\n";

        let normalized =
            normalize_sections_with(source, tokens(&["83JRT4K2P6", "7K3M9Q2W", "0123456789"]))
                .unwrap();

        assert_eq!(
            normalized.content,
            "> <a id=\"s-83JRT4K2P6\"></a>\n> ## Quoted\n\n- <a id=\"s-7K3M9Q2W\"></a>\n  ## Listed\n\n- item\n  <a id=\"s-0123456789\"></a>\n  ### Nested\n"
        );
        let reparsed = ParsedDocument::parse(&normalized.content).unwrap();
        assert_eq!(reparsed.headings.len(), 3);
        assert!(reparsed.headings.iter().all(|heading| heading.id.is_some()));
    }

    #[test]
    fn retries_collisions() {
        let source = "<a id=\"s-83JRT4K2P6\"></a>\n# Existing\n\n## Missing\n";

        let normalized =
            normalize_sections_with(source, tokens(&["83JRT4K2P6", "7K3M9Q2W"])).unwrap();

        assert_eq!(normalized.inserted[0].id.as_str(), "s-7K3M9Q2W");
    }

    #[test]
    fn retries_ids_reserved_by_other_documents() {
        let source = "# Missing\n";
        let reserved = [StableSectionId::parse("s-83JRT4K2P6").unwrap()];

        let normalized =
            normalize_sections_with_reserved(source, reserved, tokens(&["83JRT4K2P6", "7K3M9Q2W"]))
                .unwrap();

        assert_eq!(normalized.inserted[0].id.as_str(), "s-7K3M9Q2W");
    }

    #[test]
    fn preserves_interleaved_list_and_quote_containers() {
        let source = "- > ## Nested\n";

        let normalized = normalize_sections_with(source, tokens(&["83JRT4K2P6"])).unwrap();

        assert_eq!(
            normalized.content,
            "- > <a id=\"s-83JRT4K2P6\"></a>\n  > ## Nested\n"
        );
        let reparsed = ParsedDocument::parse(&normalized.content).unwrap();
        assert_eq!(reparsed.headings.len(), 1);
        assert!(reparsed.headings[0].id.is_some());
    }

    #[test]
    fn preserves_crlf_line_endings() {
        let source = "# One\r\n\r\n## Two\r\n";

        let normalized =
            normalize_sections_with(source, tokens(&["83JRT4K2P6", "7K3M9Q2W"])).unwrap();

        assert!(!normalized.content.replace("\r\n", "").contains('\n'));
        assert_eq!(
            ParsedDocument::parse(&normalized.content)
                .unwrap()
                .headings
                .len(),
            2
        );
    }
}
