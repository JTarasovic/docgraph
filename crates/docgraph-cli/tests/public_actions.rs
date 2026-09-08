use serde_yaml_ng::Value;
use std::{fs, path::Path, process::Command};

#[test]
fn installer_verifies_before_execution_and_exports_only_complete_installations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("bash")
        .arg("tools/action/test.sh")
        .current_dir(root)
        .output()
        .expect("Bash is required to test the public actions");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn validation_preserves_change_ref_as_one_argument_and_propagates_failure() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let action: Value =
        serde_yaml_ng::from_str(&fs::read_to_string(root.join("action.yml")).unwrap()).unwrap();
    let validation = action["runs"]["steps"][0]["run"].as_str().unwrap();
    for changes in [
        "",
        "refs/remotes/origin/main",
        "ref with spaces; $(exit 99)",
    ] {
        let script = format!(
            r#"set -euo pipefail
docgraph() {{
    [[ $1 == validate ]] || return 90
    if [[ -n $DOCGRAPH_CHANGES ]]; then
        [[ $# == 3 && $2 == --changes && $3 == "$DOCGRAPH_CHANGES" ]] || return 91
    else
        [[ $# == 1 ]] || return 92
    fi
    return 42
}}
{validation}"#
        );
        let status = Command::new("bash")
            .args(["-c", &script])
            .env_remove("DOCGRAPH_EXECUTABLE")
            .env("DOCGRAPH_CHANGES", changes)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(42));
    }
}
