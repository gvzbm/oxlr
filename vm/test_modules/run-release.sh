#!/bin/bash

cargo build --workspace --release
ASM=../../target/release/asm
VM=../../target/release/vm

# assemble test modules
echo "==== Assembling test modules ===="
mkdir -p /tmp/oxlr_test_modules
find -type f -name "*.s" | xargs -I {} -- $ASM {} /tmp/oxlr_test_modules
echo

# run test modules
echo "==== Running test modules ======="
export OXLR_MODULE_PATH=/tmp/oxlr_test_modules
export RUST_LOG=info
find -type f -name "*.s" -printf "%f\n" | cut -d '.' -f -1 \
    | xargs -n 1 -- $VM
