#!/bin/bash

set -euo pipefail

export PATH=../target/debug:$PATH

pushd ..
cargo build
popd

mdbook serve
