#!/bin/bash
set -e

cargo build

$JAVA_HOME/bin/java -cp jthreadinfo/build/libs/jthreadinfo.jar jthreadinfo.SampleMain &
sleep 5

export RUST_BACKTRACE=full

cp ./target/debug/jtaskstats /tmp/jtaskstats
/sbin/setcap cap_net_admin+ep /tmp/jtaskstats
/tmp/jtaskstats $(jobs -p)
