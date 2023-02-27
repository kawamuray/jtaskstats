#!/bin/bash
set -e

rustup target add x86_64-unknown-linux-musl

./jthreadinfo/gradlew --build-file ./jthreadinfo/build.gradle build
cargo build --target x86_64-unknown-linux-musl --release
gzip -c target/x86_64-unknown-linux-musl/release/jtaskstats > jtaskstats-x86_64-linux-musl-$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version').gz
