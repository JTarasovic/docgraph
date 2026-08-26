use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn copy(name: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let target = std::env::temp_dir().join(format!(
            "docgraph-cli-{name}-{}-{sequence}",
            std::process::id()
        ));
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("fixtures")
            .join(name);
        copy_directory(&source, &target);
        Self(target)
    }

    fn run(&self, arguments: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_docgraph"))
            .current_dir(&self.0)
            .args(arguments)
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_directory(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

#[test]
fn structured_describe_validate_and_query_are_stable() {
    let fixture = Fixture::copy("synthetic");

    let describe = fixture.run(&["--json", "describe"]);
    assert!(
        describe.status.success(),
        "{}",
        String::from_utf8_lossy(&describe.stderr)
    );
    let describe: Value = serde_json::from_slice(&describe.stdout).unwrap();
    assert_eq!(describe["project"], "Synthetic ontology conformance");
    assert_eq!(describe["entity_types"][0], "florp");

    assert!(fixture.run(&["validate"]).status.success());
    let query = fixture.run(&[
        "--json",
        "query",
        "grommit_targets",
        "--arg",
        "florp=florp:1",
    ]);
    assert!(
        query.status.success(),
        "{}",
        String::from_utf8_lossy(&query.stderr)
    );
    let query: Value = serde_json::from_slice(&query.stdout).unwrap();
    assert_eq!(query["columns"][0]["name"], "target");
    assert_eq!(query["rows"][0]["target"], "github:issue:owner/repo:123");
}

#[test]
fn transition_dry_run_then_apply_updates_the_fixture() {
    let fixture = Fixture::copy("adr");
    let path = fixture.0.join("docs/0001-first.md");
    let before = fs::read_to_string(&path).unwrap();

    let preview = fixture.run(&["transition", "adr:1", "accepted", "--dry-run"]);
    assert!(preview.status.success());
    assert!(String::from_utf8_lossy(&preview.stdout).contains("-state = \"proposed\""));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);

    assert!(
        fixture
            .run(&["transition", "adr:1", "accepted"])
            .status
            .success()
    );
    assert!(
        fs::read_to_string(path)
            .unwrap()
            .contains("state = \"accepted\"")
    );
    assert!(fixture.run(&["validate"]).status.success());
}
