#!/usr/bin/env bash
# Exercise the actual installer with local release fixtures and a fake gh transport.
set -euo pipefail
scripts=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
scratch=$(mktemp -d)
export RUNNER_OS=Linux RUNNER_ARCH=X64 RUNNER_TEMP="$scratch"
export DOCGRAPH_VERSION=v1.2.3
export TEST_FIXTURE="$scratch/fixture" TEST_EXECUTION_LOG="$scratch/executed"
export GITHUB_PATH="$scratch/path" GITHUB_ENV="$scratch/env" GITHUB_OUTPUT="$scratch/output"
archive=docgraph-cli-x86_64-unknown-linux-gnu.tar.gz
payload=${archive%.tar.gz}
mkdir -p "$TEST_FIXTURE/$payload"

gh() {
    case "$1 $2" in
        'release download')
            [[ $3 == v1.2.3 && $4 == --repo && $5 == JTarasovic/docgraph ]] || return 91
            [[ ${TEST_FAILURE:-} != download ]] || return 92
            cp "$TEST_FIXTURE/$archive" "$TEST_FIXTURE/$archive.sha256" .
            ;;
        'attestation verify')
            [[ $4 == --repo && $5 == JTarasovic/docgraph ]] || return 93
            [[ $6 == --signer-workflow && $7 == JTarasovic/docgraph/.github/workflows/release.yml ]] || return 94
            [[ $8 == --source-ref && $9 == refs/tags/v1.2.3 && ${10} == --deny-self-hosted-runners ]] || return 95
            [[ ${TEST_FAILURE:-} != attestation ]] || return 96
            ;;
        *) return 97 ;;
    esac
}
export -f gh
export archive

make_release() {
    printf '#!/usr/bin/env bash\nprintf "executed\\n" >> "$TEST_EXECUTION_LOG"\nprintf "docgraph %s\\n"\n' "$1" > "$TEST_FIXTURE/$payload/docgraph"
    printf 'runtime\n' > "$TEST_FIXTURE/$payload/docgraph-logic-runtime"
    chmod +x "$TEST_FIXTURE/$payload/docgraph"
    tar -czf "$TEST_FIXTURE/$archive" -C "$TEST_FIXTURE" "$payload"
    (cd "$TEST_FIXTURE" && sha256sum "$archive" && printf '\n') > "$TEST_FIXTURE/$archive.sha256"
}

expect_failure() {
    # Reset only this test's output files, never an installation or user data.
    : > "$GITHUB_OUTPUT"
    : > "$GITHUB_ENV"
    : > "$GITHUB_PATH"
    : > "$TEST_EXECUTION_LOG"
    if bash "$scripts/install.sh" > "$scratch/failure.log" 2>&1; then
        echo "Expected installation failure: $1" >&2
        exit 1
    fi
    test ! -s "$GITHUB_OUTPUT"
    test ! -s "$GITHUB_ENV"
    test ! -s "$GITHUB_PATH"
}

make_release 1.2.3
bash "$scripts/install.sh"
grep -qx 'version=1.2.3' "$GITHUB_OUTPUT"
grep -q '^DOCGRAPH_LOGIC_RUNTIME=' "$GITHUB_ENV"
test -s "$TEST_EXECUTION_LOG"

for TEST_FAILURE in download attestation; do
    export TEST_FAILURE
    expect_failure "$TEST_FAILURE"
    test ! -s "$TEST_EXECUTION_LOG"
done
unset TEST_FAILURE
printf 'corruption' >> "$TEST_FIXTURE/$archive"
expect_failure checksum
test ! -s "$TEST_EXECUTION_LOG"

make_release 9.9.9
expect_failure version
make_release 1.2.3
# Create archives omitting each required executable, with otherwise valid checksums.
for missing in docgraph docgraph-logic-runtime; do
    tar --exclude="$payload/$missing" -czf "$TEST_FIXTURE/$archive" -C "$TEST_FIXTURE" "$payload"
    (cd "$TEST_FIXTURE" && sha256sum "$archive") > "$TEST_FIXTURE/$archive.sha256"
    expect_failure "missing $missing"
    test ! -s "$TEST_EXECUTION_LOG"
done
for DOCGRAPH_VERSION in latest 1.2.3 v01.2.3 'v1.2.3; echo injected'; do
    export DOCGRAPH_VERSION
    expect_failure 'invalid version'
    test ! -s "$TEST_EXECUTION_LOG"
done
export DOCGRAPH_VERSION=v1.2.3 RUNNER_ARCH=ARM64
expect_failure 'unsupported runner'
test ! -s "$TEST_EXECUTION_LOG"
printf 'Installer success and failure tests passed.\n'
