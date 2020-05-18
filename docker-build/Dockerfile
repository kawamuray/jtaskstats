FROM jthreadinfo-build:latest as jthreadinfo

FROM rust:buster as rust

FROM debian:buster

MAINTAINER Yuto Kawamura

RUN apt-get update && apt-get install -y curl llvm-dev libclang-dev gcc

WORKDIR /jtaskstats

COPY --from=jthreadinfo /usr/local/openjdk-8 /usr/local/openjdk-8
COPY --from=jthreadinfo /root/.gradle /root/.gradle
ENV JAVA_HOME /usr/local/openjdk-8
ENV LD_LIBRARY_PATH=${JAVA_HOME}/jre/lib/amd64/server

COPY --from=rust /usr/local/rustup /usr/local/rustup
COPY --from=rust /usr/local/cargo /usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.43.1

CMD cargo test
