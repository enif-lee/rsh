#!/bin/sh
# Check source identity without trusting Cargo's shared target cache.
# The embedded marker works for binaries of every supported platform.
set -eu
if [ "$#" -lt 2 ]; then
    echo "usage: $0 SOURCE BINARY..." >&2
    exit 2
fi
check_source=$(cd "$1" && pwd)
shift
check_tmp=$(mktemp -d "${TMPDIR:-/tmp}/rsh-build-check.XXXXXX")
trap 'rm -rf "$check_tmp"' EXIT HUP INT TERM
rustc "$check_source/build.rs" -o "$check_tmp/build-id"
build_output=$(cd "$check_source" && "$check_tmp/build-id")
expected=$(printf '%s\n' "$build_output" | sed -n 's/^cargo:rustc-env=RSH_BUILD=//p')
case "$expected" in
    ""|*[!0-9a-f]*) echo "could not read source build id: $check_source" >&2; exit 2 ;;
esac
for binary do
    if ! LC_ALL=C grep -aFq -- "rsh-build-mark:$expected:end" "$binary"; then
        echo "build mismatch: $binary (expected $expected from $check_source)" >&2
        exit 1
    fi
    printf 'verified build %s: %s\n' "$expected" "$binary"
done
