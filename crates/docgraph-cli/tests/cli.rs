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

fn logic_runtime_configured() -> bool {
    std::env::var_os("DOCGRAPH_LOGIC_RUNTIME").is_some_and(|value| !value.is_empty())
}

#[test]
fn adopt_previews_then_manages_an_existing_document() {
    let fixture = Fixture::copy("synthetic");
    let document = fixture.0.join("docs/adopt-me.md");
    let original = "# Adopt me\n\nKeep this prose.\n";
    fs::write(&document, original).unwrap();

    let preview = fixture.run(&[
        "adopt",
        "docs/adopt-me.md",
        "--id",
        "florp:adopted",
        "--type",
        "florp",
        "--property",
        "title=Adopted florp",
        "--dry-run",
    ]);
    assert!(
        preview.status.success(),
        "{}",
        String::from_utf8_lossy(&preview.stderr)
    );
    assert_eq!(fs::read_to_string(&document).unwrap(), original);
    assert!(String::from_utf8_lossy(&preview.stdout).contains("florp:adopted"));

    let apply = fixture.run(&[
        "adopt",
        "docs/adopt-me.md",
        "--id",
        "florp:adopted",
        "--type",
        "florp",
        "--property",
        "title=Adopted florp",
    ]);
    assert!(
        apply.status.success(),
        "{}",
        String::from_utf8_lossy(&apply.stderr)
    );
    let adopted = fs::read_to_string(&document).unwrap();
    assert!(adopted.contains("id = \"florp:adopted\""));
    assert!(adopted.contains("# docgraph:generated:v1:begin"));
    assert!(adopted.contains("# Adopt me\n\nKeep this prose.\n"));
    assert!(adopted.contains("<a id=\"s-"));

    let get = fixture.run(&["--json", "get", "florp:adopted"]);
    assert!(get.status.success());
    let get: Value = serde_json::from_slice(&get.stdout).unwrap();
    assert_eq!(get["id"], "florp:adopted");
    assert_eq!(get["properties"]["title"], "Adopted florp");
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

    let get = fixture.run(&["--json", "get", "florp:1"]);
    assert!(get.status.success());
    let get: Value = serde_json::from_slice(&get.stdout).unwrap();
    assert_eq!(get["properties"]["title"], "Florp one");
    assert_eq!(get["properties"]["count"], 7);
    assert_eq!(get["properties"]["score"], 2.5);
    assert_eq!(get["properties"]["enabled"], true);
    assert_eq!(
        get["properties"]["labels"],
        serde_json::json!(["odd", "novel"])
    );

    if logic_runtime_configured() {
        let validate = fixture.run(&["validate"]);
        assert!(
            validate.status.success(),
            "stdout: {} stderr: {}",
            String::from_utf8_lossy(&validate.stdout),
            String::from_utf8_lossy(&validate.stderr)
        );
    }
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
    if !logic_runtime_configured() {
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

    let document = fixture.0.join("docs/florp.md");
    let before = fs::read_to_string(&document).unwrap();
    let preview = fixture.run(&["property", "set", "florp:1", "count", "8", "--dry-run"]);
    assert!(preview.status.success());
    assert_eq!(fs::read_to_string(&document).unwrap(), before);
    assert!(
        fixture
            .run(&["property", "set", "florp:1", "count", "8"])
            .status
            .success()
    );
    let get = fixture.run(&["--json", "get", "florp:1"]);
    let get: Value = serde_json::from_slice(&get.stdout).unwrap();
    assert_eq!(get["properties"]["count"], 8);
    assert!(
        !fixture
            .run(&["property", "set", "florp:1", "score", "not-a-float"])
            .status
            .success()
    );
    assert!(
        fixture
            .run(&["property", "unset", "florp:1", "labels"])
            .status
            .success()
    );
    let details = fixture.run(&["--json", "query", "florp_details", "--arg", "florp=florp:1"]);
    let details: Value = serde_json::from_slice(&details.stdout).unwrap();
    assert_eq!(details["rows"], serde_json::json!([]));
    assert!(
        fixture
            .run(&[
                "property",
                "set",
                "florp:1",
                "labels",
                "[\"odd\", \"novel\"]",
            ])
            .status
            .success()
    );

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
    if !logic_runtime_configured() {
        return;
    }
    let fixture = Fixture::copy("adr");
    let path = fixture.0.join("docs/0001-first.md");
    let before = fs::read_to_string(&path).unwrap();
    let query = fixture.run(&["--json", "query", "accepted_adrs"]);
    let query: Value = serde_json::from_slice(&query.stdout).unwrap();
    assert_eq!(query["rows"], serde_json::json!([{ "adr": "adr:2" }]));

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
    let query = fixture.run(&["--json", "query", "accepted_adrs"]);
    let query: Value = serde_json::from_slice(&query.stdout).unwrap();
    assert_eq!(query["rows"].as_array().unwrap().len(), 2);
    assert!(
        query["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["adr"] == "adr:1")
    );
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

#[test]
fn historical_research_mutation_updates_context_and_query_results() {
    if !logic_runtime_configured() {
        return;
    }
    let fixture = Fixture::copy("historical-research");
    let target = "finding:retry-memory#s-5D6F7G8H9J";

    let query = fixture.run(&[
        "--json",
        "query",
        "supported_findings",
        "--arg",
        "research=research:retry-history",
    ]);
    assert!(query.status.success());
    let query: Value = serde_json::from_slice(&query.stdout).unwrap();
    assert_eq!(query["rows"][0]["finding"], target);

    let section = fixture.run(&["--json", "get", target]);
    assert!(section.status.success());
    let section: Value = serde_json::from_slice(&section.stdout).unwrap();
    assert_eq!(section["kind"], "section");
    assert_eq!(section["document"], "docs/finding.md");
    assert_eq!(section["span"]["start_line"], 21);
    assert_eq!(section["span"]["line_count"], 3);
    assert!(
        section["content"]
            .as_str()
            .unwrap()
            .contains("retry policy was introduced")
    );
    assert!(
        section["relations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|relation| {
                relation["predicate"] == "supports" && relation["origin"] == "explicit"
            })
    );

    let preview = fixture.run(&[
        "unrelate",
        "research:retry-history",
        "supports",
        target,
        "--dry-run",
    ]);
    assert!(preview.status.success());
    assert!(String::from_utf8_lossy(&preview.stdout).contains("finding.md"));

    assert!(
        fixture
            .run(&["unrelate", "research:retry-history", "supports", target,])
            .status
            .success()
    );
    assert!(fixture.run(&["frontmatter", "check"]).status.success());
    let query = fixture.run(&[
        "--json",
        "query",
        "supported_findings",
        "--arg",
        "research=research:retry-history",
    ]);
    let query: Value = serde_json::from_slice(&query.stdout).unwrap();
    assert_eq!(query["rows"], serde_json::json!([]));

    let relate = fixture.run(&["relate", "research:retry-history", "supports", target]);
    assert!(
        relate.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&relate.stdout),
        String::from_utf8_lossy(&relate.stderr)
    );
    assert!(fixture.run(&["frontmatter", "check"]).status.success());
    assert!(fixture.run(&["validate"]).status.success());
    let query = fixture.run(&[
        "--json",
        "query",
        "supported_findings",
        "--arg",
        "research=research:retry-history",
    ]);
    let query: Value = serde_json::from_slice(&query.stdout).unwrap();
    assert_eq!(query["rows"][0]["finding"], target);
}

#[test]
fn v0_fixtures_exercise_exact_graph_search_and_named_query_retrieval() {
    if !logic_runtime_configured() {
        return;
    }
    for (fixture_name, entity, search_term, query_name, query_argument) in [
        ("adr", "adr:2", "stable anchors", "accepted_adrs", None),
        (
            "historical-research",
            "research:retry-history",
            "2019 outage",
            "supported_findings",
            Some("research=research:retry-history"),
        ),
        (
            "synthetic",
            "florp:1",
            "novel ontology",
            "grommit_targets",
            Some("florp=florp:1"),
        ),
    ] {
        let fixture = Fixture::copy(fixture_name);
        assert!(fixture.run(&["--json", "get", entity]).status.success());
        let search = fixture.run(&["--json", "search", search_term]);
        assert!(search.status.success());
        let search: Value = serde_json::from_slice(&search.stdout).unwrap();
        assert!(!search["rows"].as_array().unwrap().is_empty());
        let neighbors = fixture.run(&["--json", "neighbors", entity, "--all"]);
        assert!(neighbors.status.success());
        let neighbors: Value = serde_json::from_slice(&neighbors.stdout).unwrap();
        assert!(!neighbors["rows"].as_array().unwrap().is_empty());

        let mut arguments = vec!["--json", "query", query_name];
        if let Some(argument) = query_argument {
            arguments.extend(["--arg", argument]);
        }
        let query = fixture.run(&arguments);
        assert!(
            query.status.success(),
            "{fixture_name}: {}",
            String::from_utf8_lossy(&query.stderr)
        );
        let query: Value = serde_json::from_slice(&query.stdout).unwrap();
        assert!(!query["rows"].as_array().unwrap().is_empty());
    }
}

#[test]
fn normalization_dry_run_apply_and_reindex_complete_the_fixture_loop() {
    let fixture = Fixture::copy("synthetic");
    let document = fixture.0.join("docs/florp.md");
    let mut source = fs::read_to_string(&document).unwrap();
    source.push_str("\n## A newly authored section\n\nUnnormalized prose.\n");
    fs::write(&document, &source).unwrap();

    let preview = fixture.run(&["normalize", "--dry-run"]);
    assert!(preview.status.success());
    assert!(String::from_utf8_lossy(&preview.stdout).contains("+<a id=\"s-"));
    assert_eq!(fs::read_to_string(&document).unwrap(), source);

    assert!(fixture.run(&["normalize"]).status.success());
    let normalized = fs::read_to_string(&document).unwrap();
    assert_eq!(normalized.matches("<a id=\"s-").count(), 2);
    assert!(fixture.run(&["frontmatter", "check"]).status.success());
    if logic_runtime_configured() {
        assert!(fixture.run(&["validate"]).status.success());
    }
    assert!(String::from_utf8_lossy(&fixture.run(&["normalize"]).stdout).contains("no changes"));
}
