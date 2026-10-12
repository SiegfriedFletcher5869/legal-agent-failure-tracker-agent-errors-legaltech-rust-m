#!/bin/sh
set -eu

: "${INFRAI_API_KEY:?set INFRAI_API_KEY before running the worker}"
cargo run --bin deadline_worker -- "${1:-MAT-1042}"

