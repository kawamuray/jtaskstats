#!/bin/bash

opts=""
while [[ "$1" == '-'* ]]; do
    opts="$opts $1"
    shift
done

root_dir=$(cd $(dirname $0)/..; pwd)
exec docker run --rm \
     --network host --cap-add NET_ADMIN --cap-add SYS_PTRACE  \
     -v $root_dir:/jtaskstats \
     -v $root_dir/docker-build/.cargo:/jtaskstats/.cargo \
     $opts \
     jtaskstats-build:latest "$@"
