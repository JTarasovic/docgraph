use crate::StableSectionId;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceTarget {
    CurrentDocumentSection(StableSectionId),
    RelativeDocument {
        path: String,
        section: Option<StableSectionId>,
    },
    CanonicalEntity {
        id: String,
        section: Option<StableSectionId>,
    },
    ExternalUri(String),
    Unresolved(String),
}

#[derive(Clone, Debug, Default)]
pub struct ReferenceClassifier {
    entity_types: BTreeSet<String>,
}

impl ReferenceClassifier {
    pub fn new(entity_types: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            entity_types: entity_types.into_iter().map(Into::into).collect(),
        }
    }

    pub fn classify(&self, raw: &str) -> ReferenceTarget {
        if let Some(fragment) = raw.strip_prefix('#') {
            return StableSectionId::parse(fragment).map_or_else(
                || ReferenceTarget::Unresolved(raw.to_owned()),
                ReferenceTarget::CurrentDocumentSection,
            );
        }

        let (base, section) = split_section(raw);
        if base.starts_with("./") || base.starts_with("../") {
            return parse_section(section).map_or_else(
                || {
                    if section.is_some() {
                        ReferenceTarget::Unresolved(raw.to_owned())
                    } else {
                        ReferenceTarget::RelativeDocument {
                            path: base.to_owned(),
                            section: None,
                        }
                    }
                },
                |section| ReferenceTarget::RelativeDocument {
                    path: base.to_owned(),
                    section: Some(section),
                },
            );
        }

        if let Some((entity_type, entity_id)) = base.split_once(':')
            && self.entity_types.contains(entity_type)
            && !entity_id.is_empty()
        {
            return match (section, parse_section(section)) {
                (Some(_), None) => ReferenceTarget::Unresolved(raw.to_owned()),
                (_, section) => ReferenceTarget::CanonicalEntity {
                    id: base.to_owned(),
                    section,
                },
            };
        }

        if has_uri_scheme(raw) {
            return ReferenceTarget::ExternalUri(raw.to_owned());
        }

        ReferenceTarget::Unresolved(raw.to_owned())
    }
}

fn split_section(raw: &str) -> (&str, Option<&str>) {
    raw.split_once('#')
        .map_or((raw, None), |(base, section)| (base, Some(section)))
}

fn parse_section(raw: Option<&str>) -> Option<StableSectionId> {
    raw.and_then(StableSectionId::parse)
}

fn has_uri_scheme(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    let mut bytes = scheme.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_the_v0_reference_grammar_without_guessing() {
        let classifier = ReferenceClassifier::new(["adr", "spec"]);

        assert!(matches!(
            classifier.classify("#s-83JRT4K2P6"),
            ReferenceTarget::CurrentDocumentSection(_)
        ));
        assert!(matches!(
            classifier.classify("../specs/retry.md#s-7K3M9Q2W"),
            ReferenceTarget::RelativeDocument {
                section: Some(_),
                ..
            }
        ));
        assert!(matches!(
            classifier.classify("adr:42"),
            ReferenceTarget::CanonicalEntity { section: None, .. }
        ));
        assert!(matches!(
            classifier.classify("spec:retry#s-7K3M9Q2W"),
            ReferenceTarget::CanonicalEntity {
                section: Some(_),
                ..
            }
        ));
        assert_eq!(
            classifier.classify("https://example.com/spec"),
            ReferenceTarget::ExternalUri("https://example.com/spec".to_owned())
        );
        assert_eq!(
            classifier.classify("https://example.com/spec#overview"),
            ReferenceTarget::ExternalUri("https://example.com/spec#overview".to_owned())
        );
        assert_eq!(
            classifier.classify("possibly-retry"),
            ReferenceTarget::Unresolved("possibly-retry".to_owned())
        );
    }

    #[test]
    fn repository_entity_types_win_over_uri_syntax() {
        let classifier = ReferenceClassifier::new(["adr"]);

        assert!(matches!(
            classifier.classify("adr:42"),
            ReferenceTarget::CanonicalEntity { .. }
        ));
        assert!(matches!(
            classifier.classify("mailto:docs@example.com"),
            ReferenceTarget::ExternalUri(_)
        ));
    }
}
