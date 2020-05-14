#!/bin/bash
exec docker build $(dirname $0) -t jtaskstats-build:latest
