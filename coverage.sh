#! /usr/bin/env bash

set -euo pipefail

# Install cargo-llvm-cov if it is not already installed
if ! cargo llvm-cov --version &>/dev/null; then
    cargo install cargo-llvm-cov
    rustup component add llvm-tools-preview
fi

cargo test --all
cargo llvm-cov --html --open
