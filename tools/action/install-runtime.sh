#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$script_dir/release.sh"
pins="$script_dir/../logic-runtime/artifacts.json"
tag=$(jq -r --arg platform "$platform" '.[$platform].release' "$pins")
archive=$(jq -r --arg platform "$platform" '.[$platform].url | split("/") | last' "$pins")
pinned_checksums=$(jq -r --arg platform "$platform" '
    .[$platform] | (.url | split("/") | last) as $archive |
    "\(.archive_sha256)  \($archive)\n\(.checksum_sha256)  \($archive).sha256\n\(.sbom_sha256)  \($archive).cdx.json"
' "$pins" | tr -d '\r')
producer=$(jq -r --arg platform "$platform" '.[$platform].producer_revision' "$pins")
binary_sha256=$(jq -r --arg platform "$platform" '.[$platform].binary_sha256' "$pins")
subjects=("$archive" "$archive.sha256" "$archive.cdx.json")
payload=${archive%.$extension}
workflow=logic-runtime.yml
source_ref=refs/heads/main
producer_args=(--source-digest "$producer")
install_release

binary="$installation/docgraph-logic-runtime$exe"
printf '%s  %s\n' "$binary_sha256" "$binary" | sha256sum --check --strict
test -d "$installation/licenses"
jq --exit-status '.bomFormat == "CycloneDX" and (.components | length > 0)' "$archive.cdx.json" >/dev/null
chmod +x "$binary"
mkdir smoke
"$binary" --no-preprocessor -D smoke "$script_dir/../logic-runtime/smoke-test.dl"
[[ $(tr -d '\r\n' < smoke/successor.csv) == 42 ]]
native_path "$installation" >> "$GITHUB_PATH"
printf 'DOCGRAPH_LOGIC_RUNTIME=%s\n' "$(native_path "$binary")" >> "$GITHUB_ENV"
printf 'executable=%s\ndirectory=%s\n' "$(native_path "$binary")" "$(native_path "$installation")" >> "$GITHUB_OUTPUT"
