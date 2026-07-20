#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -f "Cargo.toml" ]]; then
    echo "Error: Cargo.toml was not found in the repository root." >&2
    exit 1
fi

run_check() {
    local description="$1"
    shift

    echo
    echo "==> ${description}"

    if ! "$@"; then
        echo
        echo "Verification failed: ${description}" >&2
        exit 1
    fi
}

run_check \
    "Checking Rust formatting" \
    cargo fmt --all -- --check

run_check \
    "Running Clippy" \
    cargo clippy --workspace --all-targets --all-features -- -D warnings

run_check \
    "Running tests" \
    cargo test --workspace --all-features

run_check \
    "Building release binaries" \
    cargo build --workspace --all-features --release

echo
echo "All verification checks passed."
