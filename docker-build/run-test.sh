#!/bin/bash
set -e

./build.sh

$JAVA_HOME/bin/java -cp jthreadinfo/build/libs/jthreadinfo.jar jthreadinfo.SampleMain &
sleep 5

export RUST_BACKTRACE=full

./docker-build/target/x86_64-unknown-linux-musl/debug/jtaskstats $(jobs -p)
