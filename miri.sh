#! /usr/bin/env bash

set -euo pipefail

# rustup +nightly component add miri

cargo +nightly miri test
