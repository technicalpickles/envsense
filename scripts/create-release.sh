#!/usr/bin/env bash

# create-release.sh
#
# Purpose: Create GitHub release with changelog extraction
# Usage: ./create-release.sh <version>
#
# Arguments:
#   version: Version string (e.g., 0.1.0)

set -euo pipefail

VERSION="$1"

echo "Creating release for version $VERSION"

# Try to extract changelog for this version
if [ -f "CHANGELOG.md" ]; then
  # Look for version section in changelog
  CHANGELOG_CONTENT=$(awk "/^## \[?v?${VERSION}\]?/,/^## \[?v?[0-9]/ { if (/^## \[?v?[0-9]/ && !/^## \[?v?${VERSION}\]?/) exit; print }" CHANGELOG.md | head -n -1)
  
  if [ -n "$CHANGELOG_CONTENT" ]; then
    echo "Found changelog content for version $VERSION"
    echo "$CHANGELOG_CONTENT" > changelog_excerpt.md
  else
    echo "No specific changelog found for version $VERSION"
    {
      echo "## Changes in v${VERSION}"
      echo ""
      echo "See commit history for details."
    } > changelog_excerpt.md
  fi
else
  echo "No CHANGELOG.md found"
  {
    echo "## Changes in v${VERSION}"
    echo ""
    echo "See commit history for details."
  } > changelog_excerpt.md
fi

echo "Release preparation completed"
