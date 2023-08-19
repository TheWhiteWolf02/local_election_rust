#!/bin/bash
if [[ "$1" == "--lib" ]]; then
    echo "Building library..."
    cargo build --release
else
    echo "Building and running executable..."
    cargo run --release
fi