#!/bin/bash

set -eux -o pipefail

cargo +nightly fmt --check

cargo +nightly clippy -- -D clippy::nursery

cargo install cargo-llvm-cov --locked
SKIP_TRAINING=1 cargo llvm-cov --release
