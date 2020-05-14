#!/bin/bash
set -e

root_dir=$(cd $(dirname $0)/..; pwd)

if [ ! -d $root_dir/docker-build/vendor ]; then
    echo "Running cargo vendor ..."
    docker run --rm -v $root_dir:/jtaskstats -v $root_dir/docker-build/.cargo:/jtaskstats/.cargo jtaskstats-build:latest cargo vendor docker-build/vendor
fi

echo "Running ..."
docker run --rm --network host --cap-add SYS_PTRACE --cap-add NET_ADMIN -v $root_dir:/jtaskstats -v $root_dir/docker-build/.cargo:/jtaskstats/.cargo jtaskstats-build:latest /bin/bash docker-build/run-test.sh
