#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repository=$(cd -- "$script_dir/../.." && pwd)
cache=${1:-"$repository/.tools/logic-runtime/linux"}
output=${2:-"$repository/target/logic-runtime/linux-x86_64"}
souffle_revision=a1303be3c0166400dee3d1f36f0d96abe03e6901
source_dir="$cache/souffle"
build_dir="$cache/build"
artifact="$output/docgraph-logic-runtime"
stamp="$output/build-inputs.sha256"

for command in cmake ninja g++ flex bison git sha256sum; do
    command -v "$command" >/dev/null || {
        echo "required build tool is unavailable: $command" >&2
        exit 1
    }
done

mkdir -p -- "$cache" "$output"
input_hash=$(
    sha256sum "$script_dir/build-linux.sh" "$script_dir/sources.toml" |
        sha256sum |
        cut -d ' ' -f 1
)
if [[ -x "$artifact" && -f "$stamp" && $(<"$stamp") == "$input_hash" ]]; then
    sha256sum "$artifact"
    exit 0
fi

if [[ ! -d "$source_dir/.git" ]]; then
    git clone --filter=blob:none https://github.com/souffle-lang/souffle.git "$source_dir"
fi
git -C "$source_dir" fetch --depth 1 origin "$souffle_revision"
git -C "$source_dir" checkout --detach "$souffle_revision"

sqlite_archive=$(g++ -print-file-name=libsqlite3.a)
if [[ $sqlite_archive == libsqlite3.a || ! -f $sqlite_archive ]]; then
    sqlite_archive=$(find /usr/lib -path '*/libsqlite3.a' -print -quit)
fi
if [[ -z $sqlite_archive || ! -f $sqlite_archive ]]; then
    echo "static SQLite library is unavailable; install libsqlite3-dev" >&2
    exit 1
fi

cmake -S "$source_dir" -B "$build_dir" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_CXX_FLAGS='-static-libgcc -static-libstdc++' \
    -DSQLite3_LIBRARY="$sqlite_archive" \
    -DSOUFFLE_DOMAIN_64BIT=ON \
    -DSOUFFLE_USE_SQLITE=ON \
    -DSOUFFLE_USE_CURSES=OFF \
    -DSOUFFLE_USE_ZLIB=OFF \
    -DSOUFFLE_USE_LIBFFI=OFF \
    -DSOUFFLE_USE_OPENMP=OFF \
    -DSOUFFLE_ENABLE_TESTING=OFF \
    -DSOUFFLE_BASH_COMPLETION=OFF \
    -DSOUFFLE_GIT=OFF
cmake --build "$build_dir" --target souffle -- -j2

install -m 0755 "$build_dir/src/souffle" "$artifact"
rm -rf -- "$output/licenses"
cp -R -- "$source_dir/licenses" "$output/licenses"
printf '%s' "$input_hash" >"$stamp"

dependencies=$(ldd "$artifact")
printf '%s\n' "$dependencies"
if grep -Eq 'not found|lib(ncurses|ffi|gomp|z)\.so' <<<"$dependencies"; then
    echo "logic runtime retains an unexpected shared dependency" >&2
    exit 1
fi
"$artifact" --version >/dev/null
sha256sum "$artifact"
