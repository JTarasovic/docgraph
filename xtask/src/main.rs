use std::{
    env,
    ffi::OsString,
    fs, io,
    path::{Component, Path, PathBuf},
    process::Command,
};

use clap::{Args, Parser, Subcommand};
use flate2::read::GzDecoder;
use quick_xml::{Reader, events::Event};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tar::Archive;
use tempfile::TempDir;

#[derive(Parser)]
#[command(name = "cargo xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Release(Release),
}

#[derive(Args)]
struct Release {
    #[command(subcommand)]
    command: ReleaseCommands,
}

#[derive(Subcommand)]
enum ReleaseCommands {
    Stage {
        #[arg(long)]
        verify_attestations: bool,
    },
    Smoke(SmokeArgs),
    VerifyInputs(VerifyInputsArgs),
    Changelog,
}

#[derive(Args)]
struct SmokeArgs {
    #[arg(long)]
    target: String,
    #[arg(long)]
    version: String,
    #[arg(long)]
    archive: PathBuf,
}

#[derive(Args)]
struct VerifyInputsArgs {
    #[arg(long)]
    manifest: PathBuf,
    #[arg(long)]
    artifacts: PathBuf,
}

#[derive(Deserialize)]
struct Sources {
    artifact: std::collections::BTreeMap<String, Artifact>,
}
#[derive(Deserialize)]
struct Artifact {
    name: String,
    release: String,
    url: String,
    archive_sha256: String,
    checksum_sha256: String,
    sbom_sha256: String,
    binary_sha256: String,
    producer_revision: String,
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("xtask: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::Release(release) => match release.command {
            ReleaseCommands::Stage {
                verify_attestations,
            } => stage(verify_attestations),
            ReleaseCommands::Smoke(args) => smoke(&args),
            ReleaseCommands::VerifyInputs(args) => verify_inputs(&args),
            ReleaseCommands::Changelog => changelog(),
        },
    }
}

fn root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(display)?;
    if !output.status.success() {
        return Err("could not determine repository root".into());
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}
fn display(error: impl std::fmt::Display) -> String {
    error.to_string()
}
fn command(program: &str, args: &[OsString]) -> Result<(), String> {
    let status = Command::new(program).args(args).status().map_err(display)?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited with {status}"))
    }
}
fn sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(display)?;
    let mut digest = Sha256::new();
    io::copy(&mut file, &mut digest).map_err(display)?;
    Ok(format!("{:x}", digest.finalize()))
}
fn require_hash(path: &Path, expected: &str) -> Result<(), String> {
    let actual = sha256(path)?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!("SHA-256 mismatch for {}", path.display()))
    }
}
fn platform() -> Result<&'static str, String> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Ok("linux-x86_64"),
        ("windows", "x86_64") => Ok("windows-x86_64"),
        value => Err(format!(
            "unsupported dist-input host {}/{}",
            value.0, value.1
        )),
    }
}

fn stage(verify_attestations: bool) -> Result<(), String> {
    let repository = root()?;
    let source: Sources = toml_edit::de::from_str(
        &fs::read_to_string(repository.join("tools/logic-runtime/sources.toml"))
            .map_err(display)?,
    )
    .map_err(display)?;
    let artifact = source
        .artifact
        .get(platform()?)
        .ok_or("missing host artifact source")?;
    let asset = artifact
        .url
        .rsplit('/')
        .next()
        .ok_or("invalid artifact URL")?;
    let scratch = TempDir::new().map_err(display)?;
    let archive = scratch.path().join(asset);
    let checksum = scratch.path().join(format!("{asset}.sha256"));
    let sbom = scratch.path().join(format!("{asset}.cdx.json"));
    let args = vec![
        OsString::from("release"),
        OsString::from("download"),
        OsString::from(&artifact.release),
        OsString::from("--repo"),
        OsString::from("JTarasovic/docgraph"),
        OsString::from("--pattern"),
        OsString::from(asset),
        OsString::from("--pattern"),
        OsString::from(format!("{asset}.sha256")),
        OsString::from("--pattern"),
        OsString::from(format!("{asset}.cdx.json")),
        OsString::from("--dir"),
        scratch.path().as_os_str().to_os_string(),
    ];
    command("gh", &args)?;
    for (path, expected) in [
        (&archive, &artifact.archive_sha256),
        (&checksum, &artifact.checksum_sha256),
        (&sbom, &artifact.sbom_sha256),
    ] {
        require_hash(path, expected)?;
    }
    verify_checksum(&checksum, scratch.path())?;
    verify_sbom(&sbom, &artifact.name)?;
    if verify_attestations {
        for subject in [&archive, &checksum, &sbom] {
            command(
                "gh",
                &[
                    OsString::from("attestation"),
                    OsString::from("verify"),
                    subject.as_os_str().to_os_string(),
                    OsString::from("--repo"),
                    OsString::from("JTarasovic/docgraph"),
                    OsString::from("--signer-workflow"),
                    OsString::from("JTarasovic/docgraph/.github/workflows/logic-runtime.yml"),
                    OsString::from("--source-digest"),
                    OsString::from(&artifact.producer_revision),
                    OsString::from("--source-ref"),
                    OsString::from("refs/heads/main"),
                    OsString::from("--deny-self-hosted-runners"),
                ],
            )?;
        }
    }
    let extracted = scratch.path().join("extracted");
    fs::create_dir(&extracted).map_err(display)?;
    extract(&archive, &extracted)?;
    let runtime = find_named_file(&extracted, &artifact.name)?
        .ok_or_else(|| format!("archive lacks {}", artifact.name))?;
    require_hash(&runtime, &artifact.binary_sha256)?;
    let licenses = runtime
        .parent()
        .ok_or("runtime has no parent")?
        .join("licenses");
    if !licenses.is_dir() {
        return Err("archive lacks runtime licenses".into());
    }
    let staging = repository.join("target/release-inputs");
    let replacement = repository.join("target/release-inputs.next");
    if replacement.exists() {
        fs::remove_dir_all(&replacement).map_err(display)?;
    }
    fs::create_dir_all(replacement.join("skills")).map_err(display)?;
    fs::create_dir_all(replacement.join("THIRD_PARTY_LICENSES/souffle")).map_err(display)?;
    fs::copy(&runtime, replacement.join("docgraph-logic-runtime")).map_err(display)?;
    copy_dir(
        &repository.join("skills/docgraph"),
        &replacement.join("skills/docgraph"),
    )?;
    copy_dir(&licenses, &replacement.join("THIRD_PARTY_LICENSES/souffle"))?;
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(display)?;
    }
    fs::rename(&replacement, &staging).map_err(display)?;
    Ok(())
}

fn extract(archive: &Path, destination: &Path) -> Result<(), String> {
    if archive.extension().is_some_and(|x| x == "zip") {
        let file = fs::File::open(archive).map_err(display)?;
        let mut zip = zip::ZipArchive::new(file).map_err(display)?;
        for index in 0..zip.len() {
            let mut entry = zip.by_index(index).map_err(display)?;
            let enclosed = entry
                .enclosed_name()
                .ok_or("zip entry escapes scratch directory")?
                .to_path_buf();
            let output = destination.join(enclosed);
            if entry.is_dir() {
                fs::create_dir_all(output).map_err(display)?;
            } else {
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent).map_err(display)?;
                }
                let mut file = fs::File::create(output).map_err(display)?;
                io::copy(&mut entry, &mut file).map_err(display)?;
            }
        }
    } else {
        let file = fs::File::open(archive).map_err(display)?;
        let mut tar = Archive::new(GzDecoder::new(file));
        for entry in tar.entries().map_err(display)? {
            let mut entry = entry.map_err(display)?;
            let path = entry.path().map_err(display)?;
            if path.components().any(|x| {
                matches!(
                    x,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            }) {
                return Err("tar entry escapes scratch directory".into());
            }
            entry.unpack(destination.join(path)).map_err(display)?;
        }
    }
    Ok(())
}
fn find_named_file(root: &Path, name: &str) -> Result<Option<PathBuf>, String> {
    for entry in fs::read_dir(root).map_err(display)? {
        let path = entry.map_err(display)?.path();
        if path.is_dir() {
            if let Some(found) = find_named_file(&path, name)? {
                return Ok(Some(found));
            }
        } else if path.file_name().is_some_and(|file| file == name) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}
fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(display)?;
    for entry in fs::read_dir(from).map_err(display)? {
        let entry = entry.map_err(display)?;
        let target = to.join(entry.file_name());
        if entry.file_type().map_err(display)?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target).map_err(display)?;
        }
    }
    Ok(())
}
fn verify_checksum(checksum: &Path, directory: &Path) -> Result<(), String> {
    let line = fs::read_to_string(checksum).map_err(display)?;
    let (expected, name) = line
        .split_once(char::is_whitespace)
        .ok_or("malformed checksum")?;
    let name = name.trim().trim_start_matches('*');
    require_hash(&directory.join(name), expected)
}
fn verify_sbom(path: &Path, runtime: &str) -> Result<(), String> {
    if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        let sbom: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).map_err(display)?).map_err(display)?;
        let components = sbom["components"]
            .as_array()
            .ok_or("SBOM has no components")?;
        let runtime_stem = runtime.strip_suffix(".exe").unwrap_or(runtime);
        let found = components
            .iter()
            .filter_map(|component| component["name"].as_str())
            .any(|name| {
                name == runtime
                    || name.ends_with(&format!("/{runtime}"))
                    || name == runtime_stem
                    || name.ends_with(&format!("/{runtime_stem}"))
            });
        return if found {
            Ok(())
        } else {
            Err(format!("CycloneDX SBOM does not identify {runtime}"))
        };
    }
    let mut reader = Reader::from_file(path).map_err(display)?;
    let mut found = false;
    let mut in_name = false;
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer).map_err(display)? {
            Event::Start(node) if node.name().as_ref() == "name" => in_name = true,
            Event::End(node) if node.name().as_ref() == "name" => in_name = false,
            Event::Text(text) if in_name => {
                let name = text.as_ref();
                let runtime_stem = runtime.strip_suffix(".exe").unwrap_or(runtime);
                found |= name == runtime
                    || name.ends_with(&format!("/{runtime}"))
                    || name == runtime_stem
                    || name.ends_with(&format!("/{runtime_stem}"));
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if found {
        Ok(())
    } else {
        Err(format!("CycloneDX SBOM does not identify {runtime}"))
    }
}

fn smoke(args: &SmokeArgs) -> Result<(), String> {
    let archive = fs::canonicalize(&args.archive).map_err(display)?;
    let executable = match args.target.as_str() {
        "linux-x86_64" => "docgraph",
        "windows-x86_64" => "docgraph.exe",
        _ => return Err("unsupported smoke target".into()),
    };
    let scratch = TempDir::new().map_err(display)?;
    extract(&archive, scratch.path())?;
    let binary = find_named_file(scratch.path(), executable)?.ok_or("archive lacks CLI")?;
    let directory = binary.parent().ok_or("CLI lacks parent")?;
    for required in [
        "docgraph-logic-runtime",
        "LICENSE",
        "README.md",
        "THIRD_PARTY_LICENSES",
        "skills",
    ] {
        if !directory.join(required).exists() {
            return Err(format!("archive lacks {required}"));
        }
    }
    let version = args.version.trim_start_matches('v');
    run_program(
        &binary,
        directory,
        &["--version"],
        Some(&format!("docgraph {version}")),
    )?;
    run_program(&binary, directory, &["--help"], None)?;
    let workspace = scratch.path().join("workspace");
    copy_dir(&root()?.join("fixtures/synthetic"), &workspace)?;
    run_program(
        &binary,
        &workspace,
        &["instructions", "sync", "--dry-run"],
        None,
    )?;
    run_program(&binary, &workspace, &["instructions", "sync"], None)?;
    for command_args in [
        ["instructions", "check"].as_slice(),
        ["validate"].as_slice(),
        ["query", "scalar_values"].as_slice(),
        ["search", "florp"].as_slice(),
    ] {
        run_program(&binary, &workspace, command_args, None)?;
    }
    Ok(())
}
fn run_program(
    binary: &Path,
    cwd: &Path,
    args: &[&str],
    expected: Option<&str>,
) -> Result<(), String> {
    let output = Command::new(binary)
        .args(args)
        .current_dir(cwd)
        .env_remove("DOCGRAPH_LOGIC_RUNTIME")
        .output()
        .map_err(display)?;
    if !output.status.success() {
        return Err(format!("{} {:?} failed", binary.display(), args));
    }
    if let Some(expected) = expected
        && String::from_utf8_lossy(&output.stdout).trim() != expected
    {
        return Err("packaged version differs from requested version".into());
    }
    Ok(())
}
fn verify_inputs(args: &VerifyInputsArgs) -> Result<(), String> {
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&args.manifest).map_err(display)?)
            .map_err(display)?;
    let version = manifest
        .pointer("/announcement_tag")
        .and_then(|v| v.as_str())
        .ok_or("manifest has no announcement_tag")?
        .trim_start_matches('v');
    let mut archives = Vec::new();
    for entry in fs::read_dir(&args.artifacts).map_err(display)? {
        let path = entry.map_err(display)?.path();
        let name = path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        if name.ends_with(".tar.gz") || name.ends_with(".zip") {
            archives.push(path);
        }
    }
    if archives.len() != 2 {
        return Err("expected exactly two product archives".into());
    }
    let unified = args.artifacts.join("sha256.sum");
    if !unified.is_file() {
        return Err("missing sha256.sum".into());
    }
    for archive in &archives {
        let adjacent = PathBuf::from(format!("{}.sha256", archive.display()));
        verify_checksum(&adjacent, &args.artifacts)?;
        verify_checksum(&unified, &args.artifacts)?;
        let target = if archive.extension().is_some_and(|x| x == "zip") {
            "windows-x86_64"
        } else {
            "linux-x86_64"
        };
        smoke(&SmokeArgs {
            target: target.into(),
            version: version.into(),
            archive: archive.clone(),
        })?;
    }
    let sbom = fs::read_dir(&args.artifacts)
        .map_err(display)?
        .filter_map(Result::ok)
        .map(|x| x.path())
        .find(|x| x.extension().is_some_and(|e| e == "xml"))
        .ok_or("missing workspace SBOM")?;
    verify_sbom(&sbom, "docgraph-cli")
}

fn changelog() -> Result<(), String> {
    let repository = root()?;
    let dry_run = env::var("DRY_RUN").map_err(|_| "cargo-release did not provide DRY_RUN")?;
    let previous =
        env::var("PREV_VERSION").map_err(|_| "cargo-release did not provide PREV_VERSION")?;
    let next = env::var("NEW_VERSION").map_err(|_| "cargo-release did not provide NEW_VERSION")?;
    if !previous.starts_with(|c: char| c.is_ascii_digit())
        || !next.starts_with(|c: char| c.is_ascii_digit())
    {
        return Err("cargo-release supplied an invalid release version".into());
    }
    let tag = format!("v{previous}");
    let status = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", &tag])
        .current_dir(&repository)
        .status()
        .map_err(display)?;
    if !status.success() {
        return Err(format!("previous product tag {tag} is absent"));
    }
    let mut command = Command::new("git-cliff");
    command.args([
        "--config",
        "cliff.toml",
        "--repository",
        ".",
        "--offline",
        "--tag",
        &format!("v{next}"),
    ]);
    if dry_run == "true" {
        command.args(["--strip", "header"]);
    } else if dry_run == "false" {
        command.args(["--prepend", "CHANGELOG.md"]);
    } else {
        return Err("cargo-release supplied invalid DRY_RUN".into());
    }
    command.arg(format!("{tag}..HEAD"));
    let status = command.current_dir(repository).status().map_err(display)?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git-cliff exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_requires_the_named_file_to_match() {
        let directory = TempDir::new().unwrap();
        let artifact = directory.path().join("archive.tar.gz");
        fs::write(&artifact, b"release artifact").unwrap();
        let checksum = directory.path().join("archive.tar.gz.sha256");
        fs::write(
            &checksum,
            format!("{}  archive.tar.gz\n", sha256(&artifact).unwrap()),
        )
        .unwrap();
        verify_checksum(&checksum, directory.path()).unwrap();
        fs::write(&artifact, b"corrupt artifact").unwrap();
        assert!(verify_checksum(&checksum, directory.path()).is_err());
    }

    #[test]
    fn malformed_checksum_is_rejected() {
        let directory = TempDir::new().unwrap();
        let checksum = directory.path().join("checksum");
        fs::write(&checksum, "not a checksum").unwrap();
        assert!(verify_checksum(&checksum, directory.path()).is_err());
    }
}
