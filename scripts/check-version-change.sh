#!/usr/bin/env bash

# check-version-change.sh
#
# Purpose: Check if current version needs to be released by comparing against existing git tags
# Usage: ./check-version-change.sh
#
# Outputs GitHub Actions environment variables:
# - VERSION_CHANGED: true/false (true if version needs releasing)
# - NEW_VERSION: the version to release
# - TAG_NAME: git tag name ({version})

set -euo pipefail

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $CURRENT_VERSION" >&2

# Check if tag already exists for this version
TAG_NAME="$CURRENT_VERSION"
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
  echo "Tag $TAG_NAME already exists - no release needed" >&2
  NEEDS_RELEASE=false
else
  echo "Tag $TAG_NAME does not exist - release needed" >&2
  NEEDS_RELEASE=true
fi

# Also check if this is a reasonable version to release
# (basic sanity check - should be a semver-like format)
if [[ ! "$CURRENT_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-].*)?$ ]]; then
  echo "Version $CURRENT_VERSION doesn't look like a valid semver - skipping release" >&2
  NEEDS_RELEASE=false
fi

# Output results
if [ "$NEEDS_RELEASE" = "true" ]; then
  echo "Release needed for version $CURRENT_VERSION" >&2
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "VERSION_CHANGED=true"
      echo "NEW_VERSION=$CURRENT_VERSION"
      echo "TAG_NAME=$TAG_NAME"
    } >> "$GITHUB_OUTPUT"
  else
    echo "VERSION_CHANGED=true"
    echo "NEW_VERSION=$CURRENT_VERSION"
    echo "TAG_NAME=$TAG_NAME"
  fi
else
  echo "No release needed" >&2
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "VERSION_CHANGED=false"
      echo "NEW_VERSION=false"
      echo "TAG_NAME=false"
    } >> "$GITHUB_OUTPUT"
  else
    echo "VERSION_CHANGED=false"
    echo "NEW_VERSION=false"
    echo "TAG_NAME=false"
  fi
fi
