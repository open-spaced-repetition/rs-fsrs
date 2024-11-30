#!/bin/bash

set -eux -o pipefail

rustup default nightly

rustup update

rustup component add rustfmt

cargo fmt --check

cargo clippy -- -D clippy::nursery

cargo install cargo-llvm-cov --locked
SKIP_TRAINING=1 cargo llvm-cov --release
