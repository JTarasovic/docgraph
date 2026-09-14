#!/usr/bin/env bash
set -euo pipefail
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
archive=$(realpath "${1:?Pass a native release archive}")
version=${2:?Pass the expected release tag}
scratch=$(mktemp -d)
case "$archive" in
    *.zip) unzip -q "$archive" -d "$scratch"; binary="$scratch/docgraph.exe" ;;
    *.tar.gz) tar -xzf "$archive" -C "$scratch"; binary="$scratch/$(basename "${archive%.tar.gz}")/docgraph" ;;
    *) exit 1 ;;
esac
for file in docgraph-logic-runtime LICENSE README.md THIRD_PARTY_LICENSES skills; do
    test -e "$(dirname "$binary")/$file"
done
unset DOCGRAPH_LOGIC_RUNTIME
[[ $("$binary" --version) == "docgraph ${version#v}" ]]
"$binary" --help >/dev/null
cp -R "$repository/fixtures/synthetic" "$scratch/workspace"
cd "$scratch/workspace"
"$binary" instructions sync --dry-run
"$binary" instructions sync
"$binary" instructions check
"$binary" validate
"$binary" query scalar_values
"$binary" search florp
