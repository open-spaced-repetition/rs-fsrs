#!/bin/bash

set -eux -o pipefail

rustup default nightly

rustup update

cargo fmt --check

cargo clippy -- -D clippy::nursery

cargo install cargo-llvm-cov --locked
SKIP_TRAINING=1 cargo llvm-cov --release
