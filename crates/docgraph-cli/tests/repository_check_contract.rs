use serde_yaml_ng::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;

const REQUIRED_CHECKS: [&str; 6] = ["deny", "fmt", "lint", "machete", "managed-changes", "test"];
const CI_JOBS: [&str; 2] = ["rust", "windows-e2e"];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

    let workflow_source = fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    let workflow = serde_yaml_ng::from_str::<Value>(&workflow_source).unwrap();
    for job in CI_JOBS {
        let steps = workflow["jobs"][job]["steps"].as_sequence().unwrap();
        let check_steps = steps
            .iter()
            .filter(|step| step["name"].as_str() == Some("Run implementation checks"))
            .collect::<Vec<_>>();
        assert_eq!(check_steps.len(), 1, "{job} must have one check step");
        let check_step = check_steps[0];
        assert_eq!(check_step["run"].as_str(), Some("mise run check"));
        assert!(
            check_step["env"]["DOCGRAPH_CHANGE_BASE"].is_string(),
            "{job} must select the managed-change base"
        );

        let checkout = steps
            .iter()
            .find(|step| step["name"].as_str() == Some("Check out repository"))
            .unwrap();
        assert_eq!(
            checkout["with"]["fetch-depth"].as_i64(),
            Some(0),
            "{job} must fetch the selected change base"
        );
    }

    for duplicated_command in [
        "mise run fmt",
        "mise run lint",
        "mise run deny",
        "mise run machete",
        "cargo nextest run",
        "validate --changes",
    ] {
        assert!(
            !workflow_source.contains(duplicated_command),
            "CI duplicates the shared contract with `{duplicated_command}`"
        );
    }
}
