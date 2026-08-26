use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceSpan {
    pub bytes: Range<usize>,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl SourceSpan {
    pub(crate) fn new(source: &str, bytes: Range<usize>) -> Self {
        let (start_line, start_column) = line_column(source, bytes.start);
        let (end_line, end_column) = line_column(source, bytes.end);
        Self {
            bytes,
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }

    pub fn line_count(&self) -> usize {
        if self.bytes.is_empty() {
            0
        } else if self.end_column == 1 && self.end_line > self.start_line {
            self.end_line - self.start_line
        } else {
            self.end_line.saturating_sub(self.start_line) + 1
        }
    }
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let prefix = &source[..offset.min(source.len())];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.chars().count() + 1, |(_, tail)| {
            tail.chars().count() + 1
        });
    (line, column)
}
