#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "usage: update-changelog.sh <version>" >&2
    exit 2
fi
version_number=${1#v}
if [[ ! $version_number =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$ ]]; then
    echo "release version must be an exact semantic version, found '$1'" >&2
    exit 2
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repository=$(cd -- "$script_dir/../.." && pwd)
previous_tag=$(git -C "$repository" describe \
    --tags \
    --match 'v[0-9]*.[0-9]*.[0-9]*' \
    --abbrev=0 \
    HEAD)
if [[ ! $previous_tag =~ ^v[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$ ]]; then
    echo "could not determine the previous stable release tag" >&2
    exit 1
fi

checked_in_version=$(
    awk '
        $0 == "[workspace.package]" { selected = 1; next }
        selected && /^\[/ { exit }
        selected && /^[[:space:]]*version[[:space:]]*=/ {
            line = $0
            sub("^[^=]*=[[:space:]]*\"", "", line)
            sub("\"[[:space:]]*$", "", line)
            print line
            exit
        }
    ' "$repository/Cargo.toml"
)
if [[ -z $checked_in_version ]]; then
    echo "Cargo.toml does not define workspace.package.version" >&2
    exit 1
fi

cliff_arguments=(
    --config "$repository/cliff.toml"
    --repository "$repository"
    --offline
    --tag "v$version_number"
)
if [[ $checked_in_version == "$version_number" ]]; then
    cliff_arguments+=(--prepend "$repository/CHANGELOG.md")
else
    cliff_arguments+=(--strip header)
fi

git-cliff "${cliff_arguments[@]}" "$previous_tag..HEAD"
if [[ $checked_in_version == "$version_number" ]]; then
    echo "Prepared changelog entries for $previous_tag..v$version_number"
else
    echo "Previewed changelog entries for $previous_tag..v$version_number"
fi
