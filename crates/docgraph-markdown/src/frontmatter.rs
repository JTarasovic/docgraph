use crate::SourceSpan;
use std::error::Error;
use std::fmt;
use std::ops::Range;
use toml_edit::{Document, DocumentMut, Item};

#[derive(Clone, Debug)]
pub struct Frontmatter {
    document: Document<String>,
    pub span: SourceSpan,
    pub content_span: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YamlFrontmatter {
    pub span: SourceSpan,
    pub content_span: SourceSpan,
}

impl Frontmatter {
    pub fn document(&self) -> &Document<String> {
        &self.document
    }

    pub fn to_mut(&self) -> DocumentMut {
        self.document.clone().into_mut()
    }

    pub fn item(&self, key: &str) -> Option<&Item> {
        self.document.get(key)
    }

    pub fn item_span(&self, source: &str, key: &str) -> Option<SourceSpan> {
        self.document
            .get(key)?
            .span()
            .map(|local| self.source_span(source, local))
    }

    pub fn source_span(&self, source: &str, local: Range<usize>) -> SourceSpan {
        let start = self.content_span.bytes.start;
        SourceSpan::new(source, (start + local.start)..(start + local.end))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontmatterError {
    pub span: SourceSpan,
    pub message: String,
}

impl fmt::Display for FrontmatterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}: {}",
            self.span.start_line, self.span.start_column, self.message
        )
    }
}

impl Error for FrontmatterError {}

pub(crate) fn parse(
    source: &str,
) -> Result<(Option<Frontmatter>, Option<YamlFrontmatter>, usize), FrontmatterError> {
    if let Some(open_end) = delimiter_end(source, 0, "+++") {
        return parse_toml(source, open_end);
    }
    if let Some(open_end) = delimiter_end(source, 0, "---") {
        return parse_yaml_region(source, open_end);
    }
    Ok((None, None, 0))
}

fn parse_toml(
    source: &str,
    open_end: usize,
) -> Result<(Option<Frontmatter>, Option<YamlFrontmatter>, usize), FrontmatterError> {
    let mut cursor = open_end;
    while cursor <= source.len() {
        let line_end = source[cursor..]
            .find('\n')
            .map_or(source.len(), |relative| cursor + relative + 1);
        let content_end = if line_end > cursor && source.as_bytes()[line_end - 1] == b'\n' {
            line_end - 1
        } else {
            line_end
        };
        let content_end = if content_end > cursor && source.as_bytes()[content_end - 1] == b'\r' {
            content_end - 1
        } else {
            content_end
        };

        if &source[cursor..content_end] == "+++" {
            let content = &source[open_end..cursor];
            let document = content.parse::<Document<String>>().map_err(|error| {
                let local = error.span().unwrap_or(0..content.len());
                let bytes = (open_end + local.start)..(open_end + local.end);
                FrontmatterError {
                    span: SourceSpan::new(source, bytes),
                    message: error.message().to_owned(),
                }
            })?;
            return Ok((
                Some(Frontmatter {
                    document,
                    span: SourceSpan::new(source, 0..line_end),
                    content_span: SourceSpan::new(source, open_end..cursor),
                }),
                None,
                line_end,
            ));
        }

        if line_end == source.len() {
            break;
        }
        cursor = line_end;
    }

    Err(FrontmatterError {
        span: SourceSpan::new(source, 0..open_end),
        message: "unclosed TOML frontmatter; expected a closing +++ line".to_owned(),
    })
}

fn parse_yaml_region(
    source: &str,
    open_end: usize,
) -> Result<(Option<Frontmatter>, Option<YamlFrontmatter>, usize), FrontmatterError> {
    let mut cursor = open_end;
    while cursor <= source.len() {
        let line_end = source[cursor..]
            .find('\n')
            .map_or(source.len(), |relative| cursor + relative + 1);
        let content_end = source[cursor..line_end]
            .trim_end_matches(['\r', '\n'])
            .len()
            + cursor;
        if matches!(&source[cursor..content_end], "---" | "...") {
            return Ok((
                None,
                Some(YamlFrontmatter {
                    span: SourceSpan::new(source, 0..line_end),
                    content_span: SourceSpan::new(source, open_end..cursor),
                }),
                line_end,
            ));
        }
        if line_end == source.len() {
            break;
        }
        cursor = line_end;
    }
    Err(FrontmatterError {
        span: SourceSpan::new(source, 0..open_end),
        message: "unclosed YAML frontmatter; expected a closing --- or ... line; docgraph frontmatter is TOML in +++ fences; run `docgraph frontmatter migrate`".to_owned(),
    })
}

/// Adds one blank line inside each frontmatter delimiter without changing the TOML body.
pub fn frame_content(content: &str, newline: &str) -> String {
    let body = content.trim_matches(['\r', '\n']);
    format!("{newline}{body}{newline}{newline}")
}

fn delimiter_end(source: &str, start: usize, delimiter: &str) -> Option<usize> {
    if source[start..].starts_with(&format!("{delimiter}\r\n")) {
        Some(start + 5)
    } else if source[start..].starts_with(&format!("{delimiter}\n")) {
        Some(start + 4)
    } else {
        None
    }
}

pub(crate) fn anchor_on_line(line: &str) -> Option<(&str, Range<usize>)> {
    let content_start = structural_prefix_len(line);
    let content = line[content_start..].trim();
    let id = content.strip_prefix("<a id=\"")?.strip_suffix("\"></a>")?;
    let leading = line[content_start..].len() - line[content_start..].trim_start().len();
    let start = content_start + leading;
    Some((id, start..(start + content.len())))
}

pub(crate) fn structural_prefix_len(line: &str) -> usize {
    structural_prefix(line).0
}

pub(crate) fn list_markers(prefix: &str) -> Vec<Range<usize>> {
    structural_prefix(prefix).1
}

fn structural_prefix(line: &str) -> (usize, Vec<Range<usize>>) {
    let bytes = line.as_bytes();
    let mut cursor = 0;
    let mut list_markers = Vec::new();

    while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
        cursor += 1;
    }

    loop {
        if cursor < bytes.len() && bytes[cursor] == b'>' {
            cursor += 1;
            if cursor < bytes.len() && bytes[cursor] == b' ' {
                cursor += 1;
            }
            while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            continue;
        }

        if cursor + 1 < bytes.len()
            && matches!(bytes[cursor], b'-' | b'+' | b'*')
            && matches!(bytes[cursor + 1], b' ' | b'\t')
        {
            list_markers.push(cursor..(cursor + 1));
            cursor += 2;
            while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            continue;
        }

        let digits_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor > digits_start
            && cursor + 1 < bytes.len()
            && matches!(bytes[cursor], b'.' | b')')
            && matches!(bytes[cursor + 1], b' ' | b'\t')
        {
            list_markers.push(digits_start..(cursor + 1));
            cursor += 2;
            while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            continue;
        }
        cursor = digits_start;
        break;
    }

    (cursor, list_markers)
}

#[cfg(test)]
mod tests {
    use super::frame_content;

    #[test]
    fn framing_is_idempotent_and_preserves_inner_toml() {
        let content = "\n\nid = \"task:1\"\n\n[properties]\ntitle = \"One\"\n\n";
        let framed = frame_content(content, "\n");
        assert_eq!(
            framed,
            "\nid = \"task:1\"\n\n[properties]\ntitle = \"One\"\n\n"
        );
        assert_eq!(frame_content(&framed, "\n"), framed);
    }

    #[test]
    fn framing_preserves_crlf() {
        assert_eq!(
            frame_content("id = \"task:1\"\r\n", "\r\n"),
            "\r\nid = \"task:1\"\r\n\r\n"
        );
    }
}
