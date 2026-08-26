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

    fn command(&self, arguments: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_docgraph"));
        command.current_dir(&self.0).args(arguments);
        command
    }

    fn run(&self, arguments: &[&str]) -> std::process::Output {
        self.command(arguments).output().unwrap()
    }

    fn run_without_logic_runtime(&self, arguments: &[&str]) -> std::process::Output {
        self.command(arguments)
            .env("DOCGRAPH_LOGIC_RUNTIME", "")
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
fn structured_describe_validate_and_unavailable_query_are_stable() {
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

    let validate = fixture.run(&["validate"]);
    assert!(
        validate.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&validate.stdout),
        String::from_utf8_lossy(&validate.stderr)
    );
    let query = fixture.run_without_logic_runtime(&[
        "--json",
        "query",
        "grommit_targets",
        "--arg",
        "florp=florp:1",
    ]);
    assert!(!query.status.success());
    assert!(String::from_utf8_lossy(&query.stderr).contains("logic runtime is unavailable"));
}

#[test]
fn configured_logic_runtime_executes_a_typed_query() {
    if std::env::var_os("DOCGRAPH_LOGIC_RUNTIME").is_none() {
        return;
    }
    let fixture = Fixture::copy("synthetic");
    let query = fixture.run(&[
        "--json",
        "query",
        "grommit_targets",
        "--arg",
        "florp=florp:1",
    ]);
    assert!(
        query.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&query.stdout),
        String::from_utf8_lossy(&query.stderr)
    );
    let query: Value = serde_json::from_slice(&query.stdout).unwrap();
    assert_eq!(query["query"], "grommit_targets");
    assert_eq!(query["rows"][0]["target"], "github:issue:owner/repo:123");

    let confidence = fixture.run(&[
        "--json",
        "query",
        "grommit_confidence",
        "--arg",
        "florp=florp:1",
    ]);
    assert!(confidence.status.success());
    let confidence: Value = serde_json::from_slice(&confidence.stdout).unwrap();
    assert_eq!(confidence["rows"][0]["confidence"], 0.75);

    let details = fixture.run(&["--json", "query", "florp_details", "--arg", "florp=florp:1"]);
    assert!(
        details.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&details.stdout),
        String::from_utf8_lossy(&details.stderr)
    );
    let details: Value = serde_json::from_slice(&details.stdout).unwrap();
    assert_eq!(details["rows"].as_array().unwrap().len(), 2);
    assert_eq!(details["rows"][0]["title"], "Florp one");
    assert_eq!(details["rows"][0]["count"], 7);
    assert_eq!(details["rows"][0]["score"], 2.5);
    assert_eq!(details["rows"][0]["enabled"], true);
    assert_eq!(details["rows"][0]["observed"], "2026-08-26T12:30:00Z");

    let scalars = fixture.run(&["--json", "query", "scalar_values"]);
    assert!(
        scalars.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&scalars.stdout),
        String::from_utf8_lossy(&scalars.stderr)
    );
    let scalars: Value = serde_json::from_slice(&scalars.stdout).unwrap();
    assert_eq!(scalars["rows"][0]["integer"], 42);
    assert_eq!(scalars["rows"][0]["float"], 3.5);
    assert_eq!(scalars["rows"][0]["boolean"], true);
    assert_eq!(scalars["rows"][0]["text"], "left\tright");

    let logic_path = fixture.0.join(".docgraph/logic.dl");
    let malformed = fs::read_to_string(&logic_path).unwrap().replace(
        "relation(florp, \"grommits\", target).",
        "relation(florp, \"grommits\", target), target = .",
    );
    fs::write(logic_path, malformed).unwrap();
    let validate = fixture.run(&["validate"]);
    assert!(!validate.status.success());
    assert!(String::from_utf8_lossy(&validate.stdout).contains("invalid-repository-logic"));
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

#[test]
fn generated_agent_guidance_is_checked_and_safely_synchronized() {
    let fixture = Fixture::copy("synthetic");
    assert!(fixture.run(&["instructions", "check"]).status.success());

    let agents = fixture.0.join("AGENTS.md");
    let stale = fs::read_to_string(&agents)
        .unwrap()
        .replace("Model: entities [florp]", "Model: entities [stale]");
    fs::write(&agents, &stale).unwrap();
    assert!(!fixture.run(&["instructions", "check"]).status.success());

    let preview = fixture.run(&["instructions", "sync", "--dry-run"]);
    assert!(preview.status.success());
    assert!(String::from_utf8_lossy(&preview.stdout).contains("entities [florp]"));
    assert_eq!(fs::read_to_string(&agents).unwrap(), stale);

    assert!(fixture.run(&["instructions", "sync"]).status.success());
    let synchronized = fs::read_to_string(&agents).unwrap();
    assert!(synchronized.contains("This user-authored text must survive"));
    assert!(synchronized.contains("Model: entities [florp]"));
    assert!(fixture.run(&["instructions", "check"]).status.success());
    assert!(fixture.run(&["instructions", "sync"]).stdout.is_empty());
}
