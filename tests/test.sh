#!/bin/bash

# Run the Cargo Tests
cargo test

# Clean up the Test Library, since we can't do this safely in the RUST unit tests.
rm -rf library
