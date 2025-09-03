#!/usr/bin/env bash

# check-version-change.sh
#
# Purpose: Check if version changed between commits and output GitHub Actions variables
# Usage: ./check-version-change.sh
#
# Outputs GitHub Actions environment variables:
# - VERSION_CHANGED: true/false
# - NEW_VERSION: the new version if changed
# - TAG_NAME: git tag name (v{version})

set -euo pipefail

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $CURRENT_VERSION"

# Get previous version from Cargo.toml
PREVIOUS_VERSION=$(git show HEAD~1:Cargo.toml | grep '^version = ' | head -1 | sed 's/version = "\(.*\)"/\1/' || echo "")
echo "Previous version: $PREVIOUS_VERSION"

# Check if version changed
if [ "$CURRENT_VERSION" != "$PREVIOUS_VERSION" ] && [ -n "$CURRENT_VERSION" ]; then
  echo "Version changed from $PREVIOUS_VERSION to $CURRENT_VERSION"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "VERSION_CHANGED=true"
      echo "NEW_VERSION=$CURRENT_VERSION"
      echo "TAG_NAME=v$CURRENT_VERSION"
    } >> "$GITHUB_OUTPUT"
  else
    echo "VERSION_CHANGED=true"
    echo "NEW_VERSION=$CURRENT_VERSION"
    echo "TAG_NAME=v$CURRENT_VERSION"
  fi
else
  echo "Version unchanged"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "VERSION_CHANGED=false" >> "$GITHUB_OUTPUT"
  else
    echo "VERSION_CHANGED=false"
  fi
fi
