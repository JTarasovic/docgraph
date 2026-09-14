#!/usr/bin/env bash
set -euo pipefail
cd "${WORKSPACE_ROOT:?cargo-release must provide WORKSPACE_ROOT}"
previous="v${PREV_VERSION:?cargo-release must provide PREV_VERSION}"
next="v${NEW_VERSION:?cargo-release must provide NEW_VERSION}"
git rev-parse --verify --quiet "$previous"
case "${DRY_RUN:?cargo-release must provide DRY_RUN}" in
    true) output=(--strip header) ;;
    false) output=(--prepend CHANGELOG.md) ;;
    *) exit 1 ;;
esac
git-cliff --config cliff.toml --repository . --offline --tag "$next" "${output[@]}" "$previous..HEAD"
