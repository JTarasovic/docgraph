#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/release.sh"

tag=${DOCGRAPH_VERSION:?An exact docgraph release tag is required}
if [[ ! $tag =~ ^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
    echo 'version must be an exact stable tag: vMAJOR.MINOR.PATCH' >&2
    exit 1
fi
archive="docgraph-cli-$target.$extension"
subjects=("$archive" "$archive.sha256")
# cargo-dist uses a flat ZIP and a named top-level directory in tar archives.
payload=${archive%.$extension}
if [[ $extension == zip ]]; then payload=.; fi
workflow=release.yml
source_ref="refs/tags/$tag"
producer_args=()
install_release

binary="$installation/docgraph$exe"
runtime="$installation/docgraph-logic-runtime"
test -f "$binary"
test -f "$runtime"
chmod +x "$binary" "$runtime"
[[ $("$binary" --version) == "docgraph ${tag#v}" ]]
native_path "$installation" >> "$GITHUB_PATH"
printf 'DOCGRAPH_EXECUTABLE=%s\n' "$(native_path "$binary")" >> "$GITHUB_ENV"
# The bundled runtime matches this CLI. install-runtime can explicitly replace it.
printf 'DOCGRAPH_LOGIC_RUNTIME=%s\n' "$(native_path "$runtime")" >> "$GITHUB_ENV"
printf 'version=%s\nexecutable=%s\n' "${tag#v}" "$(native_path "$binary")" >> "$GITHUB_OUTPUT"
