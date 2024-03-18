#!/usr/bin/env bash

cargo build
cbindgen --config cbindgen.toml --crate c_call_rust --output c_call_rust.h --lang c
gcc -g main.c -o main -lc_call_rust -L./target/debug
