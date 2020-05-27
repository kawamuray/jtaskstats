jtaskstats
==========

Linux taskstats applied for JVM threads.
This binary helps you to inspect resource utilizations and performance metrics of JVM threads which are "tasks" from linux kernel viewpoint.
It would be particularly useful as a tool to inspect each thread's "delay" statistics available only in kernel, a.k.a [Delay Accounting](https://www.kernel.org/doc/html/latest/accounting/delay-accounting.html).

Much specifically, this command line program does 1. List up all threads information from the target JVM and 2. Obtain their task statistics from kernel via [taskstats](https://www.kernel.org/doc/Documentation/accounting/taskstats.txt) interface.


# Usage

```sh
$ jtaskstats JVMPID
$ jtaskstats -d JVMPID
$ jtaskstats -v JVMPID
```

# How to build

```sh
./build.sh
```

Or on platform other than linux:

```sh
./docker-build/build-docker-image.sh # Just once, creates a image `jtaskstats-build:latest`
./docker-build/build.sh
# The outputs will be created under docker-build/target
```

# License

[MIT](./LICENSE)
