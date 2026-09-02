#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repository=$(cd -- "$script_dir/../.." && pwd)
target_root="$repository/target"
staging="$target_root/release-inputs"
verify_attestations=false
if [[ ${1:-} == --verify-attestations ]]; then
    verify_attestations=true
    shift
fi
if [[ $# -ne 0 ]]; then
    echo "usage: stage-dist-inputs.sh [--verify-attestations]" >&2
    exit 2
fi

toml_string() {
    local section=$1
    local key=$2
    local value
    value=$(
        awk -v section="[$section]" -v key="$key" '
            $0 == section { selected = 1; next }
            selected && /^\[/ { exit }
            selected {
                line = $0
                if (line ~ "^[[:space:]]*" key "[[:space:]]*=[[:space:]]*\"") {
                    sub("^[^=]*=[[:space:]]*\"", "", line)
                    sub("\"[[:space:]]*$", "", line)
                    print line
                    exit
                }
            }
        ' "$repository/tools/logic-runtime/sources.toml"
    )
    if [[ -z $value ]]; then
        echo "missing $key in [$section] in tools/logic-runtime/sources.toml" >&2
        exit 1
    fi
    printf '%s\n' "$value"
}

case "$(uname -s)" in
    Linux*)
        platform=linux-x86_64
        ;;
    MINGW* | MSYS* | CYGWIN*)
        platform=windows-x86_64
        ;;
    *)
        echo "dist inputs support only Windows and Linux x86-64 hosts" >&2
        exit 1
        ;;
esac
if [[ $(uname -m) != x86_64 ]]; then
    echo "dist inputs do not support host architecture '$(uname -m)'" >&2
    exit 1
fi

commands=(gh jq sha256sum find)
if [[ $platform == windows-x86_64 ]]; then
    commands+=(unzip)
else
    commands+=(tar)
fi
for command in "${commands[@]}"; do
    command -v "$command" >/dev/null || {
        echo "required staging tool is unavailable: $command" >&2
        exit 1
    }
done

section="artifact.$platform"
runtime_name=$(toml_string "$section" name)
release=$(toml_string "$section" release)
url=$(toml_string "$section" url)
archive_sha256=$(toml_string "$section" archive_sha256)
checksum_sha256=$(toml_string "$section" checksum_sha256)
sbom_sha256=$(toml_string "$section" sbom_sha256)
binary_sha256=$(toml_string "$section" binary_sha256)
producer_revision=$(toml_string "$section" producer_revision)
asset_name=${url##*/}
checksum_name="$asset_name.sha256"
sbom_name="$asset_name.cdx.json"

scratch=$(mktemp -d "${TMPDIR:-/tmp}/docgraph-dist-inputs.XXXXXXXX")
trap 'rm -rf -- "$scratch"' EXIT
archive="$scratch/$asset_name"
checksum="$scratch/$checksum_name"
sbom="$scratch/$sbom_name"
extracted="$scratch/extracted"
mkdir -p -- "$extracted"

gh release download "$release" \
    --repo JTarasovic/docgraph \
    --pattern "$asset_name" \
    --pattern "$checksum_name" \
    --pattern "$sbom_name" \
    --dir "$scratch"
printf '%s  %s\n' "$archive_sha256" "$archive" | sha256sum --check --status
printf '%s  %s\n' "$checksum_sha256" "$checksum" | sha256sum --check --status
printf '%s  %s\n' "$sbom_sha256" "$sbom" | sha256sum --check --status
(cd -- "$scratch" && sha256sum --check --strict "$checksum_name")

jq --exit-status --arg runtime "$runtime_name" '
    .bomFormat == "CycloneDX" and
    (.specVersion | type == "string") and
    (.components | type == "array" and length > 0) and
    any(.components[];
        .name == $runtime or (.name | endswith("/" + $runtime))) and
    any(.components[]; .name | endswith("/licenses/SOUFFLE-UPL.txt"))
' "$sbom" >/dev/null

if $verify_attestations; then
    for subject in "$archive" "$checksum" "$sbom"; do
        gh attestation verify "$subject" \
            --repo JTarasovic/docgraph \
            --signer-workflow JTarasovic/docgraph/.github/workflows/logic-runtime.yml \
            --source-digest "$producer_revision" \
            --source-ref refs/heads/main \
            --deny-self-hosted-runners \
            --format json >/dev/null
    done
fi

case "$archive" in
    *.zip) unzip -q "$archive" -d "$extracted" ;;
    *.tar.gz) tar -xzf "$archive" -C "$extracted" ;;
    *)
        echo "unsupported logic runtime archive: $asset_name" >&2
        exit 1
        ;;
esac

runtime=$(find "$extracted" -type f -name "$runtime_name" -print -quit)
if [[ -z $runtime ]]; then
    echo "downloaded logic runtime archive does not contain $runtime_name" >&2
    exit 1
fi
printf '%s  %s\n' "$binary_sha256" "$runtime" | sha256sum --check --status
licenses="$(dirname -- "$runtime")/licenses"
if [[ ! -d $licenses ]]; then
    echo "downloaded logic runtime archive does not contain its licenses directory" >&2
    exit 1
fi

rm -rf -- "$staging"
mkdir -p -- "$staging/skills" "$staging/THIRD_PARTY_LICENSES/souffle"
cp -- "$runtime" "$staging/docgraph-logic-runtime"
cp -R -- "$repository/skills/docgraph" "$staging/skills/"
cp -R -- "$licenses/." "$staging/THIRD_PARTY_LICENSES/souffle/"
if [[ $platform == linux-x86_64 ]]; then
    chmod +x "$staging/docgraph-logic-runtime"
fi

echo "staged dist inputs for $platform at $staging"
