# Use the official Rust image with version 1.89.0
FROM rust:1.89.0-bullseye

# Install additional tools that might be needed
RUN apt-get update && apt-get install -y \
    jq \
    coreutils \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set up environment variables to simulate GitHub Actions
ENV GITHUB_ACTIONS=true \
    CI=true \
    GITHUB_WORKSPACE=/workspaces/envsense \
    GITHUB_REPOSITORY=technicalpickles/envsense \
    GITHUB_REPOSITORY_OWNER=technicalpickles \
    GITHUB_ACTOR=technicalpickles \
    RUNNER_OS=Linux \
    RUNNER_ARCH=X64 \
    DEBIAN_FRONTEND=noninteractive

# Create a non-root user
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Set the default user
USER $USERNAME
