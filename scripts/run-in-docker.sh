#!/bin/bash
set -euo pipefail

# Script to run envsense tests in a Docker container that simulates GitHub Actions
# This is an alternative to using VS Code devcontainers

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Building Docker image for CI simulation..."

# Create a temporary Dockerfile
cat > "$PROJECT_ROOT/Dockerfile.ci-test" << 'EOF'
FROM rust:1.89.0-bullseye

# Install required tools
RUN apt-get update && apt-get install -y \
    jq \
    coreutils \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set up GitHub Actions environment variables
ENV GITHUB_ACTIONS=true \
    CI=true \
    GITHUB_WORKSPACE=/workspace \
    GITHUB_REPOSITORY=technicalpickles/envsense \
    GITHUB_REPOSITORY_OWNER=technicalpickles \
    GITHUB_ACTOR=technicalpickles \
    RUNNER_OS=Linux \
    RUNNER_ARCH=X64

WORKDIR /workspace
COPY . .

# Build the project
RUN cargo build

# Default command
CMD ["bash"]
EOF

# Build the Docker image
docker build -f "$PROJECT_ROOT/Dockerfile.ci-test" -t envsense-ci-test "$PROJECT_ROOT"

echo "Docker image built successfully!"
echo ""
echo "Usage examples:"
echo ""
echo "# Run interactive shell in CI environment:"
echo "docker run -it --rm envsense-ci-test"
echo ""
echo "# Run baseline comparison:"
echo "docker run --rm envsense-ci-test ./scripts/compare-baseline.sh"
echo ""
echo "# Run environment comparison:"
echo "docker run --rm envsense-ci-test ./scripts/compare-environments.sh"
echo ""
echo "# Test specific scenario:"
echo "docker run --rm envsense-ci-test ./scripts/compare-baseline.sh cursor"
echo ""

# Clean up temporary Dockerfile
rm -f "$PROJECT_ROOT/Dockerfile.ci-test"

echo "Running baseline comparison in Docker container..."
docker run --rm envsense-ci-test ./scripts/compare-baseline.sh
