#!/usr/bin/env bash

VERSION="$1"

echo "Testing release notes creation..."
./scripts/create-release.sh "$VERSION"

if [ -f "changelog_excerpt.md" ]; then
    echo "Release notes created successfully"
    cat changelog_excerpt.md
else
    echo "Release notes not created!"
    exit 1
fi