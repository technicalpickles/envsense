#!/usr/bin/env bash

# validate-simple.sh
#
# Purpose: Simple validation of envsense CLI functionality
# Created: 2025-09-02
# Used for: Basic validation testing

echo "Starting simple validation..."

# Test basic help
echo "Testing help command..."
cargo run -- --help > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✓ Help command works"
else
    echo "✗ Help command failed"
    exit 1
fi

# Test info command
echo "Testing info command..."
cargo run -- info --json > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✓ Info command works"
else
    echo "✗ Info command failed"
    exit 1
fi

# Test check command
echo "Testing check command..."
cargo run -- check agent > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✓ Check command works"
else
    echo "✗ Check command failed"
    exit 1
fi

echo "All basic tests passed!"
