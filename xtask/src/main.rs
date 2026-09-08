use std::{
    env, fs, io,
    path::{Component, Path, PathBuf},
    process::Command,
};

use clap::{Args, Parser, Subcommand};
use flate2::read::GzDecoder;
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
        runtime: PathBuf,
    },
    Smoke(SmokeArgs),
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

#[derive(Deserialize)]
struct Artifact {
    binary_sha256: String,
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
            ReleaseCommands::Stage { runtime } => stage(&runtime),
            ReleaseCommands::Smoke(args) => smoke(&args),
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

fn stage(runtime: &Path) -> Result<(), String> {
    let repository = root()?;
    let sources: std::collections::BTreeMap<String, Artifact> = serde_json::from_str(
        &fs::read_to_string(repository.join("tools/logic-runtime/artifacts.json"))
            .map_err(display)?,
    )
    .map_err(display)?;
    let artifact = sources
        .get(platform()?)
        .ok_or("missing host artifact source")?;
    // Installation and producer verification belong to the public action. Staging
    // only accepts the pinned binary and copies the product-specific payload.
    require_hash(runtime, &artifact.binary_sha256)?;
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
    fs::copy(runtime, replacement.join("docgraph-logic-runtime")).map_err(display)?;
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
