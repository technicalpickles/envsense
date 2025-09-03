#!/usr/bin/env bash

set -euo pipefail

echo "Testing version change detection script..."

eval "$(./scripts/check-version-change.sh)"

if [ -z "$VERSION_CHANGED" ]; then
    echo "VERSION_CHANGED is not set"
    exit 1
fi

if [ -z "$NEW_VERSION" ]; then
    echo "NEW_VERSION is not set"
    exit 1
fi

if [ -z "$TAG_NAME" ]; then
    echo "TAG_NAME is not set"
    exit 1
fi

echo "Version change detection script works!"