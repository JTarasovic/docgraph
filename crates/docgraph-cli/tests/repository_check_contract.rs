use serde_yaml_ng::Value as YamlValue;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;

const REQUIRED_CHECKS: [&str; 8] = [
    "commit-messages",
    "deny",
    "fmt",
    "lint",
    "machete",
    "managed-changes",
    "release-check",
    "test",
];
const WINDOWS_PATHS: [&str; 11] = [
    ".cargo/**",
    ".github/workflows/windows-e2e.yml",
    "Cargo.lock",
    "Cargo.toml",
    "action.yml",
    "crates/**",
    "fixtures/**",
    "mise.toml",
    "skills/**",
    "tools/action/**",
    "tools/logic-runtime/**",
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn steps<'a>(workflow: &'a YamlValue, job: &str) -> &'a Vec<YamlValue> {
    workflow["jobs"][job]["steps"].as_sequence().unwrap()
}

fn named_step<'a>(steps: &'a [YamlValue], name: &str) -> &'a YamlValue {
    steps
        .iter()
        .find(|step| step["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("missing `{name}` step"))
}

fn path_set<'a>(workflow: &'a YamlValue, event: &str) -> BTreeSet<&'a str> {
    workflow["on"][event]["paths"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect()
}

#[test]
fn mise_and_ci_share_one_required_check_contract() {
    let root = repository_root();
    let mise_source = fs::read_to_string(root.join("mise.toml")).unwrap();
    let mise = mise_source.parse::<DocumentMut>().unwrap();
    let configured = mise["tasks"]["check"]["depends"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    let required = REQUIRED_CHECKS.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(configured, required, "mise check contract changed");

    let lint = mise["tasks"]["lint"]["run"].as_str().unwrap();
    assert!(
        lint.contains(" --no-deps "),
        "Clippy must not lint third-party dependencies"
    );
    let windows_overlay = mise["tasks"]["windows-e2e"]["run"]
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .as_inline_table()
        .unwrap();
    assert_eq!(
        windows_overlay.get("task").and_then(|task| task.as_str()),
        Some("test"),
        "Windows must reuse the shared locked test task"
    );
    assert!(
        mise["tasks"]["windows-e2e"]
            .as_table()
            .unwrap()
            .get("run_windows")
            .is_none(),
        "Windows overlay must not replace the shared test task"
    );

    let linux_source = fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    let linux = serde_yaml_ng::from_str::<YamlValue>(&linux_source).unwrap();
    let linux_steps = steps(&linux, "rust");
    let linux_check = named_step(linux_steps, "Run implementation checks");
    assert_eq!(linux_check["run"].as_str(), Some("mise run check"));
    assert_eq!(
        linux_check["env"]["DOCGRAPH_CHANGE_BASE"].as_str(),
        Some(
            "${{ github.event_name == 'pull_request' && github.event.pull_request.base.sha || github.event.before }}"
        )
    );
    assert_eq!(
        linux_check["env"]["DOCGRAPH_CHANGE_HEAD"].as_str(),
        Some(
            "${{ github.event_name == 'pull_request' && github.event.pull_request.head.sha || github.sha }}"
        )
    );
    let commit_messages = mise["tasks"]["commit-messages"]["run"].as_str().unwrap();
    assert!(commit_messages.contains("DOCGRAPH_CHANGE_BASE"));
    assert!(commit_messages.contains("DOCGRAPH_CHANGE_HEAD"));
    let linux_checkout = named_step(linux_steps, "Check out repository");
    assert_eq!(linux_checkout["with"]["fetch-depth"].as_i64(), Some(0));
    let linux_install = named_step(linux_steps, "Install pinned development tools");
    assert_eq!(
        linux_install["with"]["install_args"].as_str(),
        Some(
            "cargo-binstall rust cargo:cargo-deny cargo:cargo-machete cargo:cargo-nextest cargo:cargo-dist cargo:committed"
        )
    );
    assert_eq!(
        named_step(linux_steps, "Smoke-test released validation action")["with"]["version"]
            .as_str(),
        Some("${{ steps.released-docgraph.outputs.tag }}")
    );
    assert!(linux["jobs"]["windows-e2e"].is_null());

    for duplicated_command in [
        "mise run fmt",
        "mise run lint",
        "mise run deny",
        "mise run machete",
        "cargo nextest run",
        "validate --changes",
    ] {
        assert!(
            !linux_source.contains(duplicated_command),
            "Linux CI duplicates the shared contract with `{duplicated_command}`"
        );
    }

    let windows_source =
        fs::read_to_string(root.join(".github/workflows/windows-e2e.yml")).unwrap();
    let windows = serde_yaml_ng::from_str::<YamlValue>(&windows_source).unwrap();
    let expected_paths = WINDOWS_PATHS.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(path_set(&windows, "push"), expected_paths);
    assert_eq!(path_set(&windows, "pull_request"), expected_paths);

    let windows_steps = steps(&windows, "windows-e2e");
    let windows_checkout = named_step(windows_steps, "Check out repository");
    assert!(
        windows_checkout["with"]["fetch-depth"].is_null(),
        "Windows should keep the default shallow checkout"
    );
    assert_eq!(
        named_step(windows_steps, "Smoke-test released validation action")["with"]["version"]
            .as_str(),
        Some("${{ steps.released-docgraph.outputs.tag }}")
    );
    let install = named_step(windows_steps, "Install Windows test tools");
    assert_eq!(
        install["with"]["install_args"].as_str(),
        Some("cargo-binstall rust cargo:cargo-nextest")
    );
    let windows_test = named_step(windows_steps, "Run Windows end-to-end tests");
    assert_eq!(
        windows_test["run"].as_str(),
        Some("mise run --skip-tools windows-e2e")
    );

    for linux_only_command in [
        "mise run check",
        "DOCGRAPH_CHANGE_BASE",
        "cargo clippy",
        "cargo deny",
        "cargo machete",
        "validate --changes",
    ] {
        assert!(
            !windows_source.contains(linux_only_command),
            "Windows workflow contains Linux-only work `{linux_only_command}`"
        );
    }
}

#[test]
fn generated_release_workflow_is_pinned_and_smoke_gated() {
    let root = repository_root();
    let source = fs::read_to_string(root.join(".github/workflows/release.yml")).unwrap();
    assert!(
        source.starts_with("# This file was autogenerated by dist:"),
        "release workflow must remain dist-generated"
    );

    let smoke_source =
        fs::read_to_string(root.join(".github/workflows/release-smoke.yml")).unwrap();
    for line in source.lines().chain(smoke_source.lines()) {
        let Some(reference) = line.trim().strip_prefix("uses: ") else {
            continue;
        };
        let reference = reference.split_whitespace().next().unwrap();
        if reference.starts_with("./") {
            continue;
        }
        let revision = reference
            .rsplit_once('@')
            .unwrap_or_else(|| panic!("external action is not pinned: {reference}"))
            .1;
        assert!(
            revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "external action is not pinned to a full commit: {reference}"
        );
    }

    let workflow = serde_yaml_ng::from_str::<YamlValue>(&source).unwrap();
    let host_needs = workflow["jobs"]["host"]["needs"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert!(
        host_needs.contains("custom-release-smoke"),
        "publishing must wait for native clean-install smoke tests"
    );
}

#[test]
fn pull_request_titles_are_checked_on_every_relevant_change() {
    let root = repository_root();
    let source = fs::read_to_string(root.join(".github/workflows/pr-title.yml")).unwrap();
    let workflow = serde_yaml_ng::from_str::<YamlValue>(&source).unwrap();
    let activity_types = workflow["on"]["pull_request_target"]["types"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        activity_types,
        ["edited", "opened", "reopened", "synchronize"]
            .into_iter()
            .collect()
    );

    let title_steps = steps(&workflow, "conventional-title");
    let validation = named_step(title_steps, "Validate pull-request title");
    assert!(validation["env"]["DOCGRAPH_COMMIT_MESSAGE"].is_string());
    assert!(
        validation["run"]
            .as_str()
            .unwrap()
            .contains("committed --commit-file -")
    );
}
