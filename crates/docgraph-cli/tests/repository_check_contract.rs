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
const MISE_CACHE_KEY: &str = "{{cache_key_prefix}}-{{platform}}-{{file_hash}}";

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

fn assert_actions_are_pinned(source: &str) {
    for line in source.lines() {
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
        linux_install["with"]["cache_key"].as_str(),
        Some(MISE_CACHE_KEY)
    );
    assert_eq!(linux_install["with"]["cache_save"].as_bool(), Some(true));
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
    assert_eq!(install["with"]["cache_key"].as_str(), Some(MISE_CACHE_KEY));
    assert_eq!(install["with"]["cache_save"].as_bool(), Some(true));
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
    assert_actions_are_pinned(&source);
    assert_actions_are_pinned(&smoke_source);

    let workflow = serde_yaml_ng::from_str::<YamlValue>(&source).unwrap();
    assert!(
        workflow["on"]["pull_request"].is_null(),
        "release planning is already covered by the required CI checks"
    );
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
    let local_steps = steps(&workflow, "build-local-artifacts");
    assert!(
        local_steps
            .iter()
            .all(|step| step["name"].as_str() != Some("Attest")),
        "release evidence must be attested together after global artifacts exist"
    );
    let host_steps = steps(&workflow, "host");
    let attest = named_step(host_steps, "Attest");
    assert_eq!(
        attest["with"]["subject-path"].as_str(),
        Some(
            "artifacts/*.tar.gz\nartifacts/*.tar.gz.sha256\nartifacts/*.zip\nartifacts/*.zip.sha256\nartifacts/*.cdx.xml\nartifacts/sha256.sum\n"
        )
    );
    assert_eq!(
        workflow["jobs"]["host"]["permissions"]["attestations"].as_str(),
        Some("write")
    );
    assert_eq!(
        workflow["jobs"]["host"]["permissions"]["id-token"].as_str(),
        Some("write")
    );
}

#[test]
fn logic_runtime_companions_are_manual_native_builds_with_evidence() {
    let root = repository_root();
    let source = fs::read_to_string(root.join(".github/workflows/logic-runtime.yml")).unwrap();
    assert_actions_are_pinned(&source);

    let workflow = serde_yaml_ng::from_str::<YamlValue>(&source).unwrap();
    let triggers = workflow["on"]
        .as_mapping()
        .unwrap()
        .keys()
        .map(|event| event.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(triggers, ["workflow_dispatch"].into_iter().collect());
    assert_eq!(
        workflow["on"]["workflow_dispatch"]["inputs"]["publish"]["default"].as_bool(),
        Some(false)
    );

    let jobs = workflow["jobs"]
        .as_mapping()
        .unwrap()
        .keys()
        .map(|job| job.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        jobs,
        ["evidence", "linux", "windows"].into_iter().collect(),
        "runtime builds should fan in from explicit native jobs"
    );
    assert!(!source.contains("matrix:"));

    for job in ["linux", "windows"] {
        let runtime_steps = steps(&workflow, job);
        named_step(runtime_steps, "Build runtime");
        let smoke = named_step(runtime_steps, "Smoke-test runtime")["run"]
            .as_str()
            .unwrap();
        assert!(smoke.contains("tools/logic-runtime/smoke-test.dl"));
        assert!(smoke.contains("successor.csv"));
        let attest = named_step(runtime_steps, "Attest native runtime binary");
        assert!(
            attest["with"]["subject-path"]
                .as_str()
                .unwrap()
                .contains("docgraph-logic-runtime")
        );
        assert_eq!(
            named_step(runtime_steps, "Upload runtime")["with"]["retention-days"].as_i64(),
            Some(1)
        );
    }

    let evidence_needs = workflow["jobs"]["evidence"]["needs"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|job| job.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(evidence_needs, ["linux", "windows"].into_iter().collect());
    let evidence_steps = steps(&workflow, "evidence");
    assert_eq!(
        named_step(evidence_steps, "Download native runtimes")["with"]["pattern"].as_str(),
        Some("logic-runtime-*-x86_64")
    );
    assert_eq!(
        named_step(evidence_steps, "Install Syft")["with"]["syft-version"].as_str(),
        Some("v1.51.1")
    );
    let generate = named_step(evidence_steps, "Generate CycloneDX SBOMs")["run"]
        .as_str()
        .unwrap();
    assert!(generate.contains("for platform in linux windows"));
    assert!(generate.contains("${GITHUB_SHA:0:8}"));
    assert!(generate.contains("--source-version \"$SOUFFLE_REVISION\""));
    assert!(generate.contains("cyclonedx-json="));
    let attest = named_step(
        evidence_steps,
        "Attest companion archives, checksums, and SBOMs",
    );
    assert_eq!(
        attest["with"]["subject-path"].as_str(),
        Some("target/logic-runtime/release/*")
    );
    let publish_step = named_step(evidence_steps, "Publish immutable companions");
    assert_eq!(publish_step["if"].as_str(), Some("${{ inputs.publish }}"));
    let publish = publish_step["run"].as_str().unwrap();
    assert!(publish.contains("${SOUFFLE_REVISION:0:8}-${GITHUB_SHA:0:8}"));
    assert!(publish.contains("Refusing to replace existing companion release"));
    assert!(!publish.contains("--clobber"));

    let runtime_sources = fs::read_to_string(root.join("tools/logic-runtime/sources.toml"))
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();
    assert_eq!(
        workflow["env"]["SOUFFLE_REVISION"].as_str(),
        runtime_sources["souffle"]["revision"].as_str()
    );
    let revision = runtime_sources["souffle"]["revision"].as_str().unwrap();
    let short = &revision[..8];
    for (platform, operating_system, extension) in [
        ("linux-x86_64", "linux", "tar.gz"),
        ("windows-x86_64", "windows", "zip"),
    ] {
        let release = runtime_sources["artifact"][platform]["release"]
            .as_str()
            .unwrap();
        assert!(release.starts_with(&format!("logic-runtime-{operating_system}-{short}")));
        let url = runtime_sources["artifact"][platform]["url"]
            .as_str()
            .unwrap();
        assert!(url.contains(&format!("/releases/download/{release}/")));
        let file = url.rsplit('/').next().unwrap();
        assert!(file.starts_with(&format!(
            "docgraph-logic-runtime-{operating_system}-x86_64-{short}"
        )));
        assert!(file.ends_with(extension));
    }
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
    let install = named_step(title_steps, "Install pinned commit validator");
    assert_eq!(install["with"]["cache_key"].as_str(), Some(MISE_CACHE_KEY));
    assert_eq!(install["with"]["cache_save"].as_bool(), Some(false));
    let validation = named_step(title_steps, "Validate pull-request title");
    assert!(validation["env"]["DOCGRAPH_COMMIT_MESSAGE"].is_string());
    assert!(
        validation["run"]
            .as_str()
            .unwrap()
            .contains("committed --commit-file -")
    );
}
