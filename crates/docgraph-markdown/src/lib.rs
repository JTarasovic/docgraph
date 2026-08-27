//! Markdown, frontmatter, reference, and source-span support for docgraph.

mod document;
mod frontmatter;
mod normalize;
mod reference;
mod span;

pub use document::{Heading, MarkdownLink, ParsedDocument};
pub use frontmatter::{Frontmatter, FrontmatterError, frame_content};
pub use normalize::{
    Normalization, NormalizeError, SectionInsertion, normalize_sections, normalize_sections_with,
    normalize_sections_with_reserved, normalize_sections_with_reserved_random,
};
pub use reference::{ReferenceClassifier, ReferenceTarget};
pub use span::SourceSpan;

/// An opaque, web-renderable stable section identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableSectionId(String);

impl StableSectionId {
    pub fn parse(value: &str) -> Option<Self> {
        let token = value.strip_prefix("s-")?;
        if token.is_empty()
            || !token.bytes().all(|byte| {
                matches!(
                    byte,
                    b'0'..=b'9'
                        | b'A'..=b'H'
                        | b'J'..=b'K'
                        | b'M'..=b'N'
                        | b'P'..=b'T'
                        | b'V'..=b'Z'
                )
            })
        {
            return None;
        }
        Some(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for StableSectionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_ids_use_canonical_crockford_base32() {
        assert!(StableSectionId::parse("s-83JRT4K2P6").is_some());
        assert!(StableSectionId::parse("s-7K3M9Q2W").is_some());
        assert!(StableSectionId::parse("s-IAMLOWER").is_none());
        assert!(StableSectionId::parse("s-lower").is_none());
    }
}
