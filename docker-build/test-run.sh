#!/bin/bash
set -e

./build.sh

cat <<'EOF' > SampleMain.java
public final class SampleMain {
    public static void main(String[] args) throws InterruptedException {
        start("Thread-A");
        start("Thread-B");
        start("Thread-C");
        start("PartitionProcessorThread-decaton-benchmark-decatonbench-6b97179f-e920-4c92-9e6b-f217345fd4e4-1-11");
        start("PartitionProcessorThread-decaton-benchmark-decatonbench-6b97179f-e920-4c92-9e6b-f217345fd4e4-0-10");
        Thread.sleep(10000);
    }

    private static void start(String name) {
        Thread th = new Thread(() -> {
            try {
                Thread.sleep(10000);
            } catch (InterruptedException e) {
                throw new RuntimeException(e);
            }
        });
        th.setName(name);
        th.start();
    }
}
EOF

$JAVA_HOME/bin/javac SampleMain.java
$JAVA_HOME/bin/java SampleMain &
sleep 5

export RUST_BACKTRACE=full
./docker-build/target/x86_64-unknown-linux-musl/debug/jtaskstats $(jobs -p)
