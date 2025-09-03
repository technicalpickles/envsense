#!/usr/bin/env bash

set -euo pipefail

echo "Testing version change detection script..."

# Capture the output from check-version-change.sh
# The diagnostic messages go to stderr, variable assignments to stdout or GITHUB_OUTPUT
if [ -n "${GITHUB_OUTPUT:-}" ]; then
    # In CI environment, check-version-change.sh writes to GITHUB_OUTPUT file
    # We need to run it and then read from the file
    ./scripts/check-version-change.sh 2>/dev/null
    if [ -f "$GITHUB_OUTPUT" ]; then
        script_output=$(cat "$GITHUB_OUTPUT")
    else
        echo "GITHUB_OUTPUT file not found: $GITHUB_OUTPUT"
        exit 1
    fi
else
    # In local environment, check-version-change.sh writes to stdout
    script_output=$(./scripts/check-version-change.sh 2>/dev/null)
fi

# Source the variable assignments
eval "$script_output"

if [ -z "$VERSION_CHANGED" ]; then
    echo "VERSION_CHANGED is not set"
    exit 1
fi

if [ "$VERSION_CHANGED" = "true" ]; then
    if [ -z "$NEW_VERSION" ]; then
        echo "NEW_VERSION should be set when VERSION_CHANGED=true"
        exit 1
    fi
    
    if [ -z "$TAG_NAME" ]; then
        echo "TAG_NAME should be set when VERSION_CHANGED=true"
        exit 1
    fi
    
    echo "Version changed: NEW_VERSION=$NEW_VERSION, TAG_NAME=$TAG_NAME"
else
    if [ "$NEW_VERSION" != "false" ]; then
        echo "NEW_VERSION should be 'false' when VERSION_CHANGED=false, got: '$NEW_VERSION'"
        exit 1
    fi
    
    if [ "$TAG_NAME" != "false" ]; then
        echo "TAG_NAME should be 'false' when VERSION_CHANGED=false, got: '$TAG_NAME'"
        exit 1
    fi
    
    echo "Version unchanged: NEW_VERSION and TAG_NAME are correctly set to 'false'"
fi

echo "Version change detection script works!"