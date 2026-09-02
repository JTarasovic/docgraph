#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "usage: smoke-test.sh <windows-x86_64|linux-x86_64> <version> <archive>" >&2
    exit 2
fi

target=$1
version_number=${2#v}
archive=$3
if [[ $archive != /* ]]; then
    archive="$PWD/$archive"
fi
if [[ ! -f $archive ]]; then
    echo "release archive does not exist: $archive" >&2
    exit 1
fi

case "$target" in
    windows-x86_64) executable_name=docgraph.exe ;;
    linux-x86_64) executable_name=docgraph ;;
    *)
        echo "unsupported release smoke-test target: $target" >&2
        exit 2
        ;;
esac

commands=(find)
if [[ $target == windows-x86_64 ]]; then
    commands+=(unzip)
else
    commands+=(tar)
fi
for command in "${commands[@]}"; do
    command -v "$command" >/dev/null || {
        echo "required smoke-test tool is unavailable: $command" >&2
        exit 1
    }
done

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repository=$(cd -- "$script_dir/../.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/docgraph-release-smoke.XXXXXXXX")
trap 'rm -rf -- "$scratch"' EXIT

case "$target" in
    windows-x86_64) unzip -q "$archive" -d "$scratch" ;;
    linux-x86_64) tar -xzf "$archive" -C "$scratch" ;;
esac

executable=$(find "$scratch" -type f -name "$executable_name" -print -quit)
if [[ -z $executable ]]; then
    echo "archive does not contain $executable_name" >&2
    exit 1
fi
executable_directory=$(dirname -- "$executable")
runtime="$executable_directory/docgraph-logic-runtime"
if [[ ! -f $runtime ]]; then
    echo "archive does not place docgraph-logic-runtime beside $executable_name" >&2
    exit 1
fi
for required in LICENSE README.md THIRD_PARTY_LICENSES skills/docgraph/skill.toml; do
    if [[ ! -e $executable_directory/$required ]]; then
        echo "archive is missing $required" >&2
        exit 1
    fi
done
if [[ $target == linux-x86_64 ]]; then
    chmod +x "$executable" "$runtime"
fi

export DOCGRAPH_LOGIC_RUNTIME=$runtime
reported_version=$("$executable" --version | tr -d '\r')
if [[ $reported_version != "docgraph $version_number" ]]; then
    echo "packaged CLI version '$reported_version' does not match '$version_number'" >&2
    exit 1
fi
"$executable" --help >/dev/null

workspace="$scratch/workspace"
cp -R -- "$repository/fixtures/synthetic" "$workspace"
pushd "$workspace" >/dev/null
if "$executable" instructions check >/dev/null 2>&1; then
    echo "a workspace without the portable skill unexpectedly passed instructions check" >&2
    exit 1
fi
"$executable" instructions sync --dry-run >/dev/null
if [[ -e skills/docgraph/SKILL.md ]]; then
    echo "instruction dry-run wrote the portable skill" >&2
    exit 1
fi
"$executable" instructions sync >/dev/null
"$executable" instructions check >/dev/null
"$executable" validate >/dev/null
"$executable" query scalar_values >/dev/null
"$executable" search florp >/dev/null
popd >/dev/null

echo "release smoke test passed: $target $version_number"
