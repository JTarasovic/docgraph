#!/usr/bin/env bash
# Shared release transport. Callers provide the exact artifact and producer policy.
set -euo pipefail

repository=JTarasovic/docgraph
case "${RUNNER_OS:-}-${RUNNER_ARCH:-}" in
    Linux-X64) platform=linux-x86_64; target=x86_64-unknown-linux-gnu; extension=tar.gz; exe= ;;
    Windows-X64) platform=windows-x86_64; target=x86_64-pc-windows-msvc; extension=zip; exe=.exe ;;
    *) echo 'docgraph supports Linux and Windows X64 runners.' >&2; exit 1 ;;
esac

install_release() {
    installation=$(mktemp -d "${RUNNER_TEMP:?}/docgraph.XXXXXXXX")
    installation=$(cd "$installation" && pwd -P)
    cd "$installation"
    gh release download "$tag" --repo "$repository" "${subjects[@]/#/--pattern=}"
    for subject in "${subjects[@]}"; do
        gh attestation verify "$subject" --repo "$repository" \
            --signer-workflow "$repository/.github/workflows/$workflow" \
            --source-ref "$source_ref" --deny-self-hosted-runners "${producer_args[@]}"
    done
    # cargo-dist emits a trailing blank line in adjacent checksum files.
    sed '/^[[:space:]]*$/d' "$archive.sha256" | sha256sum --check --strict
    if [[ -n ${pinned_checksums:-} ]]; then
        printf '%s\n' "$pinned_checksums" | sha256sum --check --strict
    fi
    mkdir extracted
    if [[ $extension == zip ]]; then
        unzip -q "$archive" -d extracted
    else
        tar -xzf "$archive" -C extracted
    fi
    installation="$installation/extracted/$payload"
    test -d "$installation"
}

# GitHub environment files must contain native paths on Windows (also for pwsh).
native_path() {
    if [[ $RUNNER_OS == Windows ]]; then cygpath -am "$1"; else printf '%s\n' "$1"; fi
}
