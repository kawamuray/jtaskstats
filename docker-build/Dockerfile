FROM openjdk:8 as jdk

# Make a gradle binary cache
COPY ./gradlew /jthreadinfo/gradlew
COPY ./gradle /jthreadinfo/gradle
RUN /jthreadinfo/gradlew --version

FROM rust:buster
MAINTAINER Yuto Kawamura

RUN apt-get update && apt-get install -y curl llvm-dev libclang-dev gcc musl-tools

COPY --from=jdk /usr/local/openjdk-8 /usr/local/openjdk-8
COPY --from=jdk /root/.gradle /root/.gradle
ENV JAVA_HOME /usr/local/openjdk-8

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /jtaskstats
CMD cargo test
