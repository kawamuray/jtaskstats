#!/bin/bash
set -e

root_dir="$(dirname $0)"

$root_dir/jthreadinfo/gradlew --build-file $root_dir/jthreadinfo/build.gradle build

cargo build --manifest-path=$root_dir/Cargo.toml
cargo test --manifest-path=$root_dir/Cargo.toml
