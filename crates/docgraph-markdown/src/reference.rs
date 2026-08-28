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
    providers: Vec<ProviderRepository>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRepository {
    pub provider: String,
    pub host: String,
    pub repository: String,
}

pub trait ReferenceAdapter: Sync {
    fn name(&self) -> &'static str;
    fn normalize(&self, raw: &str, context: &ProviderRepository) -> Option<String>;
}

struct GithubAdapter;
struct GitlabAdapter;

static GITHUB_ADAPTER: GithubAdapter = GithubAdapter;
static GITLAB_ADAPTER: GitlabAdapter = GitlabAdapter;

pub fn reference_adapter(name: &str) -> Option<&'static dyn ReferenceAdapter> {
    match name {
        "github" => Some(&GITHUB_ADAPTER),
        "gitlab" => Some(&GITLAB_ADAPTER),
        _ => None,
    }
}

impl ReferenceClassifier {
    pub fn new(entity_types: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            entity_types: entity_types.into_iter().map(Into::into).collect(),
            providers: Vec::new(),
        }
    }

    pub fn with_providers(
        mut self,
        providers: impl IntoIterator<Item = ProviderRepository>,
    ) -> Self {
        self.providers = providers.into_iter().collect();
        self
    }

    pub fn classify(&self, raw: &str) -> ReferenceTarget {
        if let Some(fragment) = raw.strip_prefix('#')
            && let Some(section) = StableSectionId::parse(fragment)
        {
            return ReferenceTarget::CurrentDocumentSection(section);
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

        if let Some(identity) = self.provider_identity(raw) {
            return ReferenceTarget::ExternalUri(identity);
        }

        if has_uri_scheme(raw) {
            return ReferenceTarget::ExternalUri(raw.to_owned());
        }

        ReferenceTarget::Unresolved(raw.to_owned())
    }

    fn provider_identity(&self, raw: &str) -> Option<String> {
        self.providers
            .iter()
            .find_map(|context| reference_adapter(&context.provider)?.normalize(raw, context))
    }
}

fn digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn external_identity(context: &ProviderRepository, kind: &str, key: &str) -> String {
    format!(
        "{}:{kind}:{}/{}:{key}",
        context.provider, context.host, context.repository
    )
}

fn qualified<'a>(raw: &'a str, repository: &str, delimiter: char) -> Option<&'a str> {
    raw.strip_prefix(&format!("{repository}{delimiter}"))
}

impl ReferenceAdapter for GithubAdapter {
    fn name(&self) -> &'static str {
        "github"
    }

    fn normalize(&self, raw: &str, context: &ProviderRepository) -> Option<String> {
        let issue = raw
            .strip_prefix("GH-")
            .or_else(|| raw.strip_prefix('#'))
            .or_else(|| qualified(raw, &context.repository, '#'))
            .filter(|value| digits(value));
        if let Some(issue) = issue {
            return Some(external_identity(context, "issue", issue));
        }
        qualified(raw, &context.repository, '@')
            .filter(|value| value.len() >= 7 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .map(|commit| external_identity(context, "commit", commit))
    }
}

impl ReferenceAdapter for GitlabAdapter {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    fn normalize(&self, raw: &str, context: &ProviderRepository) -> Option<String> {
        let issue = raw
            .strip_prefix('#')
            .or_else(|| qualified(raw, &context.repository, '#'))
            .filter(|value| digits(value));
        if let Some(issue) = issue {
            return Some(external_identity(context, "issue", issue));
        }
        let change = raw
            .strip_prefix('!')
            .or_else(|| qualified(raw, &context.repository, '!'))
            .filter(|value| digits(value));
        if let Some(change) = change {
            return Some(external_identity(context, "merge_request", change));
        }
        qualified(raw, &context.repository, '@')
            .filter(|value| value.len() >= 7 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .map(|commit| external_identity(context, "commit", commit))
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

    #[test]
    fn configured_provider_shorthand_normalizes_offline() {
        let classifier = ReferenceClassifier::new(["task"]).with_providers([
            ProviderRepository {
                provider: "github".to_owned(),
                host: "github.com".to_owned(),
                repository: "owner/repo".to_owned(),
            },
            ProviderRepository {
                provider: "gitlab".to_owned(),
                host: "git.example.com".to_owned(),
                repository: "group/project".to_owned(),
            },
        ]);

        assert_eq!(
            classifier.classify("#123"),
            ReferenceTarget::ExternalUri("github:issue:github.com/owner/repo:123".to_owned())
        );
        assert_eq!(
            classifier.classify("GH-47"),
            ReferenceTarget::ExternalUri("github:issue:github.com/owner/repo:47".to_owned())
        );
        assert_eq!(
            classifier.classify("group/project!9"),
            ReferenceTarget::ExternalUri(
                "gitlab:merge_request:git.example.com/group/project:9".to_owned()
            )
        );
        assert_eq!(
            classifier.classify("owner/repo@a5c3785"),
            ReferenceTarget::ExternalUri("github:commit:github.com/owner/repo:a5c3785".to_owned())
        );
        assert_eq!(
            classifier.classify("a5c3785"),
            ReferenceTarget::Unresolved("a5c3785".to_owned())
        );
        assert!(matches!(
            classifier.classify("task:123"),
            ReferenceTarget::CanonicalEntity { .. }
        ));
    }
}
