#!/usr/bin/env bash

set -euo pipefail

echo "Testing version change detection script..."

eval "$(./scripts/check-version-change.sh)"

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