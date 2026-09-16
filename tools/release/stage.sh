#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
runtime=${1:?Pass the verified runtime executable}
staging=target/release-inputs
test -f "$runtime"
test -d "$(dirname "$runtime")/licenses"
mkdir -p "$staging/skills" "$staging/THIRD_PARTY_LICENSES"
cp "$runtime" "$staging/docgraph-logic-runtime"
cp -R .agents/skills/docgraph "$staging/skills/"
cp -R "$(dirname "$runtime")/licenses/." "$staging/THIRD_PARTY_LICENSES/"
