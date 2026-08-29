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

fn commit_fixture(fixture: &Fixture) {
    for arguments in [
        &["init"][..],
        &["config", "user.email", "docgraph@example.invalid"],
        &["config", "user.name", "Docgraph Test"],
        &["add", "."],
        &["commit", "-m", "fixture"],
    ] {
        let output = Command::new("git")
            .current_dir(&fixture.0)
            .args(arguments)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
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
    assert!(adopted.contains("[docgraph_generated]\nschema_version = 1"));
    assert!(adopted.contains("# Adopt me\n\nKeep this prose.\n"));
    assert!(adopted.contains("<a id=\"s-"));

    let get = fixture.run(&["--json", "get", "florp:adopted"]);
    assert!(get.status.success());
    let get: Value = serde_json::from_slice(&get.stdout).unwrap();
    assert_eq!(get["id"], "florp:adopted");
    assert_eq!(get["properties"]["title"], "Adopted florp");
}

#[test]
fn document_commands_create_move_and_safely_delete() {
    let fixture = Fixture::copy("synthetic");
    let created = fixture.0.join("docs/created.md");
    let moved = fixture.0.join("docs/archive/created.md");

    let preview = fixture.run(&[
        "document",
        "create",
        "docs/created.md",
        "--id",
        "florp:created",
        "--type",
        "florp",
        "--title",
        "Created florp",
        "--dry-run",
    ]);
    assert!(
        preview.status.success(),
        "{}",
        String::from_utf8_lossy(&preview.stderr)
    );
    assert!(!created.exists());
    assert!(String::from_utf8_lossy(&preview.stdout).contains("florp:created"));

    let create = fixture.run(&[
        "document",
        "create",
        "docs/created.md",
        "--id",
        "florp:created",
        "--type",
        "florp",
        "--title",
        "Created florp",
    ]);
    assert!(
        create.status.success(),
        "{}",
        String::from_utf8_lossy(&create.stderr)
    );
    assert!(created.exists());
    let created_source = fs::read_to_string(&created).unwrap();
    assert!(created_source.contains("<a id=\"s-"));
    assert!(created_source.contains("title = \"Created florp\""));

    let conflicting = fixture.run(&[
        "document",
        "create",
        "docs/conflicting.md",
        "--id",
        "florp:conflicting",
        "--type",
        "florp",
        "--title",
        "Heading title",
        "--property",
        "title=Metadata title",
    ]);
    assert!(!conflicting.status.success());
    assert!(
        String::from_utf8_lossy(&conflicting.stderr)
            .contains("--title conflicts with --property title=")
    );
    assert!(!fixture.0.join("docs/conflicting.md").exists());

    let move_document = fixture.run(&[
        "document",
        "move",
        "florp:created",
        "docs/archive/created.md",
    ]);
    assert!(
        move_document.status.success(),
        "{}",
        String::from_utf8_lossy(&move_document.stderr)
    );
    assert!(!created.exists());
    assert!(moved.exists());

    assert!(
        fixture
            .run(&["relate", "florp:1", "precedes", "florp:created"])
            .status
            .success()
    );
    let blocked = fixture.run(&["document", "delete", "florp:created"]);
    assert!(!blocked.status.success());
    assert!(String::from_utf8_lossy(&blocked.stderr).contains("inbound references remain"));
    assert!(moved.exists());

    assert!(
        fixture
            .run(&["unrelate", "florp:1", "precedes", "florp:created"])
            .status
            .success()
    );
    let delete = fixture.run(&["document", "delete", "florp:created"]);
    assert!(
        delete.status.success(),
        "{}",
        String::from_utf8_lossy(&delete.stderr)
    );
    assert!(!moved.exists());
    assert!(fixture.run(&["validate"]).status.success());
}

#[test]
fn section_commands_split_merge_and_delete() {
    let fixture = Fixture::copy("synthetic");
    let document = fixture.0.join("docs/sections.md");
    let create = fixture.run(&[
        "document",
        "create",
        "docs/sections.md",
        "--id",
        "florp:sections",
        "--type",
        "florp",
        "--title",
        "Section lifecycle",
        "--property",
        "title=Section lifecycle",
    ]);
    assert!(
        create.status.success(),
        "{}",
        String::from_utf8_lossy(&create.stderr)
    );
    let mut source = fs::read_to_string(&document).unwrap();
    let root_id = source
        .split("<a id=\"")
        .nth(1)
        .unwrap()
        .split('\"')
        .next()
        .unwrap()
        .to_owned();
    source.push_str("\nAlpha body.\n\nBeta body.\n");
    fs::write(&document, &source).unwrap();
    let beta_line = source[..source.find("Beta body.").unwrap()]
        .matches('\n')
        .count()
        + 1;
    let beta_line = beta_line.to_string();
    let root = format!("florp:sections#{root_id}");

    let split = fixture.run(&[
        "section",
        "split",
        &root,
        "--at-line",
        &beta_line,
        "--title",
        "Beta",
    ]);
    assert!(
        split.status.success(),
        "{}",
        String::from_utf8_lossy(&split.stderr)
    );
    let split_source = fs::read_to_string(&document).unwrap();
    let new_id = split_source
        .split("<a id=\"")
        .skip(1)
        .filter_map(|tail| tail.split('\"').next())
        .find(|id| *id != root_id)
        .unwrap()
        .to_owned();
    let new_section = format!("florp:sections#{new_id}");
    assert!(split_source.contains("# Beta"));

    let merge = fixture.run(&["section", "merge", &new_section, "--into", &root]);
    assert!(
        merge.status.success(),
        "{}",
        String::from_utf8_lossy(&merge.stderr)
    );
    let merged = fs::read_to_string(&document).unwrap();
    assert!(!merged.contains(&new_id));
    assert!(merged.contains("Beta body."));

    let beta_line = (merged[..merged.find("Beta body.").unwrap()]
        .matches('\n')
        .count()
        + 1)
    .to_string();
    assert!(
        fixture
            .run(&[
                "section",
                "split",
                &root,
                "--at-line",
                &beta_line,
                "--title",
                "Disposable",
            ])
            .status
            .success()
    );
    let split_source = fs::read_to_string(&document).unwrap();
    let disposable_id = split_source
        .split("<a id=\"")
        .skip(1)
        .filter_map(|tail| tail.split('\"').next())
        .find(|id| *id != root_id)
        .unwrap();
    let disposable = format!("florp:sections#{disposable_id}");
    assert!(
        fixture
            .run(&["section", "delete", &disposable])
            .status
            .success()
    );
    assert!(
        !fs::read_to_string(&document)
            .unwrap()
            .contains("Beta body.")
    );
    assert!(fixture.run(&["validate"]).status.success());
}

#[test]
fn workflow_initialize_materializes_missing_states() {
    let fixture = Fixture::copy("synthetic");
    let path = fixture.0.join("docs/florp.md");
    let source = fs::read_to_string(&path).unwrap();
    fs::write(&path, source.replace("state = \"queued\"\n", "")).unwrap();

    let result = fixture.run(&["workflow", "initialize", "florp"]);

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        fs::read_to_string(path)
            .unwrap()
            .contains("state = \"queued\"")
    );
}

#[test]
fn change_validation_allows_prose_and_rejects_illegal_state_jumps() {
    let fixture = Fixture::copy("synthetic");
    let path = fixture.0.join("docs/florp.md");
    let initial = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        initial.replace("state = \"queued\"", "state = \"done\""),
    )
    .unwrap();
    commit_fixture(&fixture);
    let original = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("{original}\nAdditional prose.\n")).unwrap();

    let prose = fixture.run(&["validate", "--changes", "HEAD"]);
    assert!(
        prose.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&prose.stdout),
        String::from_utf8_lossy(&prose.stderr)
    );

    fs::write(
        &path,
        original.replace("state = \"done\"", "state = \"queued\""),
    )
    .unwrap();
    let illegal = fixture.run(&["validate", "--changes", "HEAD"]);
    assert!(!illegal.status.success());
    assert!(String::from_utf8_lossy(&illegal.stdout).contains("unsupported-workflow-state-change"));
}

#[test]
fn semantic_review_reports_text_and_structured_graph_changes() {
    let fixture = Fixture::copy("synthetic");
    commit_fixture(&fixture);

    assert!(
        fixture
            .run(&["transition", "florp:2", "active"])
            .status
            .success()
    );
    assert!(
        fixture
            .run(&["property", "set", "florp:1", "count", "8"])
            .status
            .success()
    );
    assert!(
        fixture
            .run(&[
                "relate",
                "florp:1",
                "grommits",
                "https://example.com/review",
            ])
            .status
            .success()
    );
    let document = fixture.0.join("docs/florp.md");
    let source = fs::read_to_string(&document).unwrap();
    fs::write(
        &document,
        source.replace("# Florp one", "# Renamed florp one"),
    )
    .unwrap();

    let structured = fixture.run(&["--json", "review", "HEAD"]);
    assert!(
        structured.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&structured.stdout),
        String::from_utf8_lossy(&structured.stderr)
    );
    let structured: Value = serde_json::from_slice(&structured.stdout).unwrap();
    assert_eq!(structured["base"], "HEAD");
    assert_eq!(structured["valid"], true);
    let changes = structured["changes"].as_array().unwrap();
    for kind in [
        "workflow_state_changed",
        "property_changed",
        "section_changed",
        "relation_added",
    ] {
        assert!(
            changes.iter().any(|change| change["kind"] == kind),
            "{kind}"
        );
    }

    let text = fixture.run(&["review", "HEAD"]);
    assert!(text.status.success());
    let text = String::from_utf8_lossy(&text.stdout);
    assert!(text.contains("semantic changes from HEAD"));
    assert!(text.contains("~ workflow florp:2 queued -> active"));
    assert!(text.contains("~ property florp:1.count 7 -> 8"));
    assert!(text.contains("+ relation florp:1 --grommits--> https://example.com/review"));
}

#[test]
fn adopt_batch_manages_multiple_unnormalized_documents() {
    let fixture = Fixture::copy("synthetic");
    fs::write(fixture.0.join("docs/batch-one.md"), "# Batch one\n").unwrap();
    fs::write(fixture.0.join("docs/batch-two.md"), "# Batch two\n").unwrap();
    fs::write(
        fixture.0.join("adopt.toml"),
        "[[document]]\npath = \"docs/batch-one.md\"\nid = \"florp:batch-one\"\ntype = \"florp\"\nproperty = [\"title=Batch one\"]\n\n[[document]]\npath = \"docs/batch-two.md\"\nid = \"florp:batch-two\"\ntype = \"florp\"\nproperty = [\"title=Batch two\"]\n",
    )
    .unwrap();

    let preview = fixture.run(&["adopt", "--batch", "adopt.toml", "--dry-run"]);
    assert!(
        preview.status.success(),
        "{}",
        String::from_utf8_lossy(&preview.stderr)
    );
    assert!(
        !fs::read_to_string(fixture.0.join("docs/batch-one.md"))
            .unwrap()
            .contains("type = \"florp\"")
    );

    let result = fixture.run(&["adopt", "--batch", "adopt.toml"]);

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    for name in ["batch-one.md", "batch-two.md"] {
        let adopted = fs::read_to_string(fixture.0.join("docs").join(name)).unwrap();
        assert!(adopted.contains("type = \"florp\""));
        assert!(adopted.contains("state = \"queued\""));
        assert!(adopted.contains("<a id=\"s-"));
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

    let complete = fixture.run(&["--json", "describe", "--all"]);
    assert!(
        complete.status.success(),
        "{}",
        String::from_utf8_lossy(&complete.stderr)
    );
    let complete: Value = serde_json::from_slice(&complete.stdout).unwrap();
    assert_eq!(complete["schema_version"], 1);
    assert_eq!(
        complete["project"]["name"],
        "Synthetic ontology conformance"
    );
    assert_eq!(complete["project"]["documents"]["root"], "docs");
    assert_eq!(
        complete["project"]["agent_instructions"]["targets"],
        serde_json::json!(["AGENTS.md", "CLAUDE.md"])
    );
    assert_eq!(
        complete["entity_types"]["grommit"]["properties"]["mode"]["values"],
        serde_json::json!(["soft", "hard"])
    );
    assert_eq!(
        complete["entity_types"]["florp"]["properties"]["labels"]["items"],
        "string"
    );
    assert_eq!(
        complete["relations"]["grommits"]["inverse"],
        "grommitted_by"
    );
    assert_eq!(
        complete["workflows"]["florp"]["states"]["queued"]["transitions"],
        serde_json::json!(["active"])
    );
    assert_eq!(
        complete["queries"]["grommit_targets"]["arguments"][0]["type"],
        "entity"
    );
    assert_eq!(
        complete["commands"]["florp.activate"]["operation"]["type"],
        "transition"
    );
    assert_eq!(complete["project"]["logic_configured"], true);

    let readable = fixture.run(&["describe", "--all"]);
    assert!(readable.status.success());
    let readable = String::from_utf8_lossy(&readable.stdout);
    assert!(readable.contains("\"entity_types\""));
    assert!(readable.contains("\"grommit_targets\""));

    let conflicting = fixture.run(&["describe", "--all", "type", "florp"]);
    assert!(!conflicting.status.success());
    assert!(
        String::from_utf8_lossy(&conflicting.stderr)
            .contains("describe --all does not accept a kind or name")
    );

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
fn directional_traversal_and_expanded_context_are_structured() {
    let fixture = Fixture::copy("synthetic");

    let incoming = fixture.run(&["--json", "incoming", "florp:1"]);
    assert!(incoming.status.success());
    let incoming: Value = serde_json::from_slice(&incoming.stdout).unwrap();
    assert!(
        incoming["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| { row["direction"] == "incoming" && row["origin"] == "explicit" })
    );
    assert!(
        incoming["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["node"] == "florp:2")
    );

    let outgoing = fixture.run(&["--json", "outgoing", "florp:1"]);
    assert!(outgoing.status.success());
    let outgoing: Value = serde_json::from_slice(&outgoing.stdout).unwrap();
    assert!(
        outgoing["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| { row["direction"] == "outgoing" && row["origin"] == "explicit" })
    );

    let section = fixture.run(&["--json", "outgoing", "florp:1#s-9K8J7H6G5F"]);
    assert!(section.status.success());
    let section: Value = serde_json::from_slice(&section.stdout).unwrap();
    assert_eq!(section["rows"][0]["node"], "florp:2#s-9D9KQWAJ82");

    let traverse = fixture.run(&[
        "--json",
        "traverse",
        "florp:1",
        "--direction",
        "outgoing",
        "--depth",
        "2",
    ]);
    assert!(traverse.status.success());
    let traverse: Value = serde_json::from_slice(&traverse.stdout).unwrap();
    assert!(
        traverse["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["node"] == "florp:3" && row["depth"] == 2)
    );

    let context = fixture.run(&["--json", "context", "florp:1", "--depth", "1"]);
    assert!(context.status.success());
    let context: Value = serde_json::from_slice(&context.stdout).unwrap();
    assert_eq!(context["reference"], "florp:1");
    assert_eq!(context["nodes"][0]["depth"], 0);
    assert_eq!(
        context["nodes"][0]["node"]["properties"]["title"],
        "Florp one"
    );
    assert!(
        context["relations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|relation| relation["origin"] == "explicit")
    );
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
    assert_eq!(
        query["rows"][0]["target"],
        "github:issue:github.com/owner/repo:123"
    );

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
fn string_properties_accept_bare_and_toml_quoted_values() {
    for raw in ["mode=hard", "mode=\"hard\""] {
        let fixture = Fixture::copy("synthetic");
        let document = fixture.0.join("docs/adopt-me.md");
        fs::write(&document, "# Adopt me\n\nKeep this prose.\n").unwrap();
        let adopt = fixture.run(&[
            "adopt",
            "docs/adopt-me.md",
            "--id",
            "grommit:adopted",
            "--type",
            "grommit",
            "--property",
            "title=Adopted grommit",
            "--property",
            raw,
        ]);
        assert!(
            adopt.status.success(),
            "{raw}: {}",
            String::from_utf8_lossy(&adopt.stderr)
        );
        let adopted = fs::read_to_string(&document).unwrap();
        assert!(adopted.contains("mode = \"hard\""), "{raw}: {adopted}");

        let set = fixture.run(&["property", "set", "grommit:adopted", "mode", "\"soft\""]);
        assert!(
            set.status.success(),
            "{}",
            String::from_utf8_lossy(&set.stderr)
        );
        let updated = fs::read_to_string(&document).unwrap();
        assert!(updated.contains("mode = \"soft\""), "{updated}");
    }
}

#[test]
fn repository_commands_appear_in_help_and_dispatch_named_queries() {
    let fixture = Fixture::copy("synthetic");
    for help_argument in ["--help", "-h", "help"] {
        let help = fixture.run(&[help_argument]);
        assert!(help.status.success());
        let help = String::from_utf8_lossy(&help.stdout);
        assert!(help.contains("Repository commands:"));
        assert!(help.contains("florp ready"));
        assert!(
            help.find("Repository commands:").unwrap() < help.find("Usage:").unwrap(),
            "repository commands should be presented before generic CLI usage and commands"
        );
    }

    let command_help = fixture.run(&["florp", "ready", "--help"]);
    assert!(command_help.status.success());
    assert!(
        String::from_utf8_lossy(&command_help.stdout)
            .contains("List florps with no incoming precedence edge.")
    );

    let group_help = fixture.run(&["florp", "--help"]);
    assert!(group_help.status.success());
    let group_help = String::from_utf8_lossy(&group_help.stdout);
    assert!(group_help.contains("ready"));
    assert!(group_help.contains("targets"));

    if !logic_runtime_configured() {
        return;
    }
    let output = fixture.run(&["--json", "florp", "ready"]);
    assert!(
        output.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output["query"], "ready_florps_command");
    assert_eq!(output["rows"][0]["florp"], "florp:1");

    let filtered = fixture.run(&["--json", "florp", "ready", "--label", "odd"]);
    assert!(filtered.status.success());
    let filtered: Value = serde_json::from_slice(&filtered.stdout).unwrap();
    assert_eq!(filtered["rows"].as_array().unwrap().len(), 1);

    let targets = fixture.run(&["--json", "florp", "targets", "florp:1"]);
    assert!(targets.status.success());
    let targets: Value = serde_json::from_slice(&targets.stdout).unwrap();
    assert_eq!(
        targets["rows"][0]["target"],
        "github:issue:github.com/owner/repo:123"
    );

    let transition = fixture.run(&["florp", "activate", "florp:2", "--dry-run"]);
    assert!(transition.status.success());
    assert!(String::from_utf8_lossy(&transition.stdout).contains("state = \"active\""));

    let relation = fixture.run(&[
        "florp",
        "grommit",
        "florp:1",
        "https://example.com/new",
        "--dry-run",
    ]);
    assert!(relation.status.success());
    assert!(String::from_utf8_lossy(&relation.stdout).contains("https://example.com/new"));
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
fn read_commands_recover_an_interrupted_mutation_before_loading_the_graph() {
    let fixture = Fixture::copy("adr");
    let preview = fixture.run(&["--json", "transition", "adr:1", "accepted", "--dry-run"]);
    assert!(preview.status.success());
    let preview: Value = serde_json::from_slice(&preview.stdout).unwrap();
    let files: Vec<_> = preview["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|change| {
            serde_json::json!({
                "path": change["path"],
                "original": change["original"],
                "intended": change["intended"],
            })
        })
        .collect();
    let journal = serde_json::json!({
        "fingerprint": preview["fingerprint"],
        "file": files,
    });
    let state = fixture.0.join(".docgraph/.state");
    fs::create_dir_all(&state).unwrap();
    fs::write(
        state.join("recovery.toml"),
        toml_edit::ser::to_string(&journal).unwrap(),
    )
    .unwrap();

    let get = fixture.run(&["--json", "get", "adr:1"]);
    assert!(
        get.status.success(),
        "{}",
        String::from_utf8_lossy(&get.stderr)
    );
    let get: Value = serde_json::from_slice(&get.stdout).unwrap();

    assert_eq!(get["state"], "accepted");
    assert!(!state.join("recovery.toml").exists());
    assert!(!state.join("index.sqlite").exists());
}

#[test]
fn generated_agent_guidance_is_checked_and_safely_synchronized() {
    let fixture = Fixture::copy("synthetic");
    assert!(fixture.run(&["instructions", "check"]).status.success());

    let agents = fixture.0.join("AGENTS.md");
    let stale = fs::read_to_string(&agents).unwrap().replace(
        "A deliberately unfamiliar entity type.",
        "A stale entity description.",
    );
    fs::write(&agents, &stale).unwrap();
    assert!(!fixture.run(&["instructions", "check"]).status.success());

    let preview = fixture.run(&["instructions", "sync", "--dry-run"]);
    assert!(preview.status.success());
    assert!(
        String::from_utf8_lossy(&preview.stdout)
            .contains("`florp` — A deliberately unfamiliar entity type.")
    );
    assert_eq!(fs::read_to_string(&agents).unwrap(), stale);

    assert!(fixture.run(&["instructions", "sync"]).status.success());
    let synchronized = fs::read_to_string(&agents).unwrap();
    assert!(synchronized.contains("This user-authored text must survive"));
    assert!(synchronized.contains("## Docgraph repository model"));
    assert!(synchronized.contains("`florp` — A deliberately unfamiliar entity type."));
    assert!(synchronized.contains("`grommits`: `florp` → `external`"));
    assert!(synchronized.contains("- `florp`; initial `queued`"));
    assert!(synchronized.contains("- `grommit`; initial `idle`"));
    assert!(synchronized.contains("docgraph query grommit_targets --arg florp=<value>"));
    assert!(synchronized.contains("- Maintain: `docgraph validate`"));
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
    assert_eq!(section["span"]["start_line"], 22);
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
fn graph_paths_accept_entities_and_stable_sections_as_endpoints() {
    let fixture = Fixture::copy("historical-research");
    let entity = "research:retry-history";
    let section = "finding:retry-memory#s-5D6F7G8H9J";

    let forward = fixture.run(&["--json", "path", entity, section]);
    assert!(
        forward.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&forward.stdout),
        String::from_utf8_lossy(&forward.stderr)
    );
    let forward: Value = serde_json::from_slice(&forward.stdout).unwrap();
    assert_eq!(forward["path"], serde_json::json!([entity, section]));

    let reverse = fixture.run(&["--json", "path", section, entity]);
    assert!(
        reverse.status.success(),
        "stdout: {} stderr: {}",
        String::from_utf8_lossy(&reverse.stdout),
        String::from_utf8_lossy(&reverse.stderr)
    );
    let reverse: Value = serde_json::from_slice(&reverse.stdout).unwrap();
    assert_eq!(reverse["path"], serde_json::json!([section, entity]));

    let missing = fixture.run(&["path", entity, "finding:retry-memory#s-0000000000"]);
    assert!(!missing.status.success());
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains(
            "entity or stable section \"finding:retry-memory#s-0000000000\" does not exist"
        )
    );
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
fn semantic_search_labels_full_text_fallback_without_a_provider() {
    let fixture = Fixture::copy("synthetic");
    let output = fixture.run(&["--json", "semantic-search", "novel ontology"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["mode"], "full_text_fallback");
    assert_eq!(result["reason"], "no embedding provider is configured");
    assert!(!result["rows"].as_array().unwrap().is_empty());
}

#[test]
fn synthetic_fixture_exercises_generic_workflows_sections_cycles_and_recursive_logic() {
    if !logic_runtime_configured() {
        return;
    }
    let fixture = Fixture::copy("synthetic");

    assert!(fixture.run(&["validate"]).status.success());
    assert!(fixture.run(&["frontmatter", "check"]).status.success());
    assert!(fixture.run(&["instructions", "check"]).status.success());
    let reachable = fixture.run(&[
        "--json",
        "query",
        "reachable_florps",
        "--arg",
        "source=florp:1",
    ]);
    assert!(
        reachable.status.success(),
        "{}",
        String::from_utf8_lossy(&reachable.stderr)
    );
    let reachable: Value = serde_json::from_slice(&reachable.stdout).unwrap();
    assert_eq!(
        reachable["rows"],
        serde_json::json!([
            { "target": "florp:2" },
            { "target": "florp:3" },
        ])
    );

    let ready = fixture.run(&["--json", "query", "ready_florps"]);
    assert!(ready.status.success());
    let ready: Value = serde_json::from_slice(&ready.stdout).unwrap();
    assert_eq!(ready["rows"], serde_json::json!([{ "florp": "florp:1" }]));

    let section = fixture.run(&["--json", "get", "florp:1#s-9K8J7H6G5F"]);
    assert!(section.status.success());
    let section: Value = serde_json::from_slice(&section.stdout).unwrap();
    assert!(
        section["relations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|relation| {
                relation["direction"] == "outgoing"
                    && relation["predicate"] == "annotates"
                    && relation["target"] == "florp:2#s-9D9KQWAJ82"
            })
    );

    assert!(
        fixture
            .run(&["transition", "grommit:1", "running"])
            .status
            .success()
    );
    let grommit = fixture.run(&["--json", "get", "grommit:1"]);
    let grommit: Value = serde_json::from_slice(&grommit.stdout).unwrap();
    assert_eq!(grommit["state"], "running");

    let cycle = fixture.run(&["relate", "florp:3", "precedes", "florp:1"]);
    assert!(!cycle.status.success());
    assert!(String::from_utf8_lossy(&cycle.stderr).contains("cycle"));
    assert!(fixture.run(&["validate"]).status.success());
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

#[test]
fn derived_index_is_sqlite_reused_when_fresh_and_refreshed_when_stale() {
    let fixture = Fixture::copy("synthetic");
    let index = fixture.0.join(".docgraph/.state/index.sqlite");
    let fingerprint = fixture.0.join(".docgraph/.state/fingerprint");

    assert!(fixture.run(&["get", "florp:1"]).status.success());
    assert!(!index.exists());
    assert!(!fingerprint.exists());

    assert!(fixture.run(&["search", "florp"]).status.success());
    let fresh_index = fs::read(&index).unwrap();
    let fresh_fingerprint = fs::read_to_string(&fingerprint).unwrap();
    assert_eq!(&fresh_index[..16], b"SQLite format 3\0");

    assert!(fixture.run(&["search", "florp"]).status.success());
    assert_eq!(fs::read(&index).unwrap(), fresh_index);
    assert_eq!(fs::read_to_string(&fingerprint).unwrap(), fresh_fingerprint);

    let document = fixture.0.join("docs/florp.md");
    let mut source = fs::read_to_string(&document).unwrap();
    source.push_str("\nPersistent-index-refresh-sentinel.\n");
    fs::write(document, source).unwrap();
    let search = fixture.run(&["--json", "search", "refresh sentinel"]);
    assert!(
        search.status.success(),
        "{}",
        String::from_utf8_lossy(&search.stderr)
    );
    let search: Value = serde_json::from_slice(&search.stdout).unwrap();

    assert!(!search["rows"].as_array().unwrap().is_empty());
    assert_ne!(fs::read_to_string(fingerprint).unwrap(), fresh_fingerprint);
    assert_ne!(fs::read(index).unwrap(), fresh_index);
}
