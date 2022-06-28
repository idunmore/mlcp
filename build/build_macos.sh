#!/bin/bash
echo "macOS Build"
echo "Building Apple Silicon binary ..."
cargo build --release --target=aarch64-apple-darwin
echo "Building x86_64 binary ..."
cargo build --release --target=x86_64-apple-darwin
echo "Creating Universal binary ..."
mkdir ../target/universal
mkdir ../target/universal/release
lipo ../target/aarch64-apple-darwin/release/mlcp ../target/x86_64-apple-darwin/release/mlcp -output ../target/universal/release/mlcp -create
echo "Universal binary can be found at: ../target/universal/release/mlcp"
echo "Done."
