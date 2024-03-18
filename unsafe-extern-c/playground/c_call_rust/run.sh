#!/usr/bin/env bash

export LD_LIBRARY_PATH=./target/debug
if [ "$1" == "v" ]; then
    valgrind ./main
else
    ./main
fi

