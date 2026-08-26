use crate::frontmatter::{anchor_on_line, parse as parse_frontmatter};
use crate::{Frontmatter, FrontmatterError, SourceSpan, StableSectionId};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

#[derive(Clone, Debug)]
pub struct ParsedDocument {
    pub frontmatter: Option<Frontmatter>,
    pub body_offset: usize,
    pub headings: Vec<Heading>,
    pub links: Vec<MarkdownLink>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Heading {
    pub id: Option<StableSectionId>,
    pub level: u8,
    pub title: String,
    pub heading_span: SourceSpan,
    pub section_span: SourceSpan,
    pub anchor_span: Option<SourceSpan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownLink {
    pub destination: String,
    pub title: String,
    pub span: SourceSpan,
    pub containing_section: Option<usize>,
}

struct BuildingHeading {
    level: u8,
    title: String,
    start: usize,
}

impl ParsedDocument {
    pub fn parse(source: &str) -> Result<Self, FrontmatterError> {
        let (frontmatter, body_offset) = parse_frontmatter(source)?;
        let mut options = Options::empty();
        options.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        let mut headings = Vec::new();
        let mut links = Vec::new();
        let mut building_heading: Option<BuildingHeading> = None;
        let mut current_section = None;

        for (event, range) in Parser::new_ext(source, options).into_offset_iter() {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    let index = headings.len();
                    current_section = Some(index);
                    building_heading = Some(BuildingHeading {
                        level: level as u8,
                        title: String::new(),
                        start: range.start,
                    });
                }
                Event::End(TagEnd::Heading(_)) => {
                    let building = building_heading
                        .take()
                        .expect("pulldown-cmark balances heading events");
                    let bytes = building.start..range.end;
                    let (id, anchor_span) = adjacent_anchor(source, bytes.start);
                    headings.push(Heading {
                        id,
                        level: building.level,
                        title: building.title.trim().to_owned(),
                        heading_span: SourceSpan::new(source, bytes.clone()),
                        section_span: SourceSpan::new(source, bytes),
                        anchor_span,
                    });
                }
                Event::Start(Tag::Link {
                    dest_url, title, ..
                }) => links.push(MarkdownLink {
                    destination: dest_url.into_string(),
                    title: title.into_string(),
                    span: SourceSpan::new(source, range),
                    containing_section: current_section,
                }),
                Event::Text(text) | Event::Code(text) if building_heading.is_some() => {
                    let heading = building_heading.as_mut().unwrap();
                    heading.title.push_str(&text);
                }
                Event::SoftBreak | Event::HardBreak if building_heading.is_some() => {
                    building_heading.as_mut().unwrap().title.push(' ');
                }
                _ => {}
            }
        }

        for index in 0..headings.len() {
            let end = headings[(index + 1)..]
                .iter()
                .find(|candidate| candidate.level <= headings[index].level)
                .map_or(source.len(), |candidate| {
                    candidate
                        .anchor_span
                        .as_ref()
                        .map_or(candidate.heading_span.bytes.start, |anchor| {
                            line_start(source, anchor.bytes.start)
                        })
                });
            headings[index].section_span =
                SourceSpan::new(source, headings[index].heading_span.bytes.start..end);
        }

        Ok(Self {
            frontmatter,
            body_offset,
            headings,
            links,
        })
    }
}

fn adjacent_anchor(
    source: &str,
    heading_start: usize,
) -> (Option<StableSectionId>, Option<SourceSpan>) {
    let heading_line_start = line_start(source, heading_start);
    if heading_line_start == 0 {
        return (None, None);
    }
    let previous_end = heading_line_start - 1;
    let previous_end = if previous_end > 0 && source.as_bytes()[previous_end - 1] == b'\r' {
        previous_end - 1
    } else {
        previous_end
    };
    let previous_start = line_start(source, previous_end);
    let line = &source[previous_start..previous_end];
    let Some((raw_id, local_span)) = anchor_on_line(line) else {
        return (None, None);
    };
    let Some(id) = StableSectionId::parse(raw_id) else {
        return (None, None);
    };
    let bytes = (previous_start + local_span.start)..(previous_start + local_span.end);
    (Some(id), Some(SourceSpan::new(source, bytes)))
}

fn line_start(source: &str, offset: usize) -> usize {
    source[..offset]
        .rfind('\n')
        .map_or(0, |newline| newline + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_headings_links_and_section_spans() {
        let source = r#"+++
id = "spec:retry"
type = "spec"
+++
<a id="s-83JRT4K2P6"></a>
## Retry *semantics*

See [ADR](../adr/42.md#s-7K3M9Q2W).

### Details

More.

## Next
"#;

        let document = ParsedDocument::parse(source).unwrap();

        let frontmatter = document.frontmatter.as_ref().unwrap();
        assert_eq!(frontmatter.item("id").unwrap().as_str(), Some("spec:retry"));
        assert_eq!(frontmatter.item_span(source, "id").unwrap().start_line, 2);
        assert_eq!(document.headings.len(), 3);
        assert_eq!(document.headings[0].title, "Retry semantics");
        assert_eq!(
            document.headings[0].id.as_ref().unwrap().as_str(),
            "s-83JRT4K2P6"
        );
        assert_eq!(document.headings[0].section_span.start_line, 6);
        assert_eq!(document.headings[0].section_span.end_line, 14);
        assert_eq!(document.headings[0].section_span.line_count(), 8);
        assert_eq!(document.links[0].containing_section, Some(0));
        assert_eq!(document.links[0].span.start_line, 8);
        assert_eq!(
            &source[document.links[0].span.bytes.clone()],
            "[ADR](../adr/42.md#s-7K3M9Q2W)"
        );
    }

    #[test]
    fn includes_nested_headings_but_not_heading_like_code() {
        let source = "> <a id=\"s-7K3M9Q2W\"></a>\n> ## Quoted\n\n- item\n  ### Listed\n\nSetext\n------\n\n```md\n## Not code\n```\n\n<div>\n## Not HTML\n</div>\n";

        let document = ParsedDocument::parse(source).unwrap();

        assert_eq!(
            document
                .headings
                .iter()
                .map(|heading| heading.title.as_str())
                .collect::<Vec<_>>(),
            ["Quoted", "Listed", "Setext"]
        );
        assert_eq!(
            document.headings[0].id.as_ref().unwrap().as_str(),
            "s-7K3M9Q2W"
        );
    }

    #[test]
    fn reports_frontmatter_errors_in_document_coordinates() {
        let source = "+++\nid = [\n+++\n# Heading\n";

        let error = ParsedDocument::parse(source).unwrap_err();

        assert_eq!(error.span.start_line, 2);
    }

    #[test]
    fn frontmatter_can_be_edited_without_reformatting_unrelated_content() {
        let source = "+++\n# retained comment\nid   =   \"adr:42\"\n+++\n# Heading\n";

        let document = ParsedDocument::parse(source).unwrap();
        let frontmatter = document.frontmatter.unwrap();

        assert_eq!(
            frontmatter.to_mut().to_string(),
            "# retained comment\nid   =   \"adr:42\"\n"
        );
    }

    #[test]
    fn a_section_does_not_claim_the_next_sections_anchor() {
        let source = "# One\nText.\n<a id=\"s-83JRT4K2P6\"></a>\n# Two\n";

        let document = ParsedDocument::parse(source).unwrap();
        let first = &document.headings[0].section_span;

        assert_eq!(&source[first.bytes.clone()], "# One\nText.\n");
        assert_eq!(first.start_line, 1);
        assert_eq!(first.line_count(), 2);
    }
}
