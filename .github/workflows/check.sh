#!/bin/bash

set -eux -o pipefail

cargo install cargo-llvm-cov --locked
SKIP_TRAINING=1 cargo llvm-cov --release
