#!/usr/bin/env bash

# dev-docker.sh
#
# Purpose: Helper script for Docker-based development
# Usage: ./scripts/dev-docker.sh [command]
#
# Commands:
#   build    - Build the development Docker image
#   run      - Run an interactive development container
#   shell    - Start a bash shell in the container
#   fish     - Start a fish shell in the container
#   test     - Run tests in the container
#   cross    - Test cross-compilation in the container
#   clean    - Remove development containers and images

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE_NAME="envsense-dev"
CONTAINER_NAME="envsense-dev-container"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_header() {
    echo
    print_status "$BLUE" "=== $1 ==="
}

print_success() {
    print_status "$GREEN" "✓ $1"
}

print_error() {
    print_status "$RED" "✗ $1"
}

print_warning() {
    print_status "$YELLOW" "⚠ $1"
}

# Build the development image
build_image() {
    print_header "Building Development Docker Image"
    
    cd "$PROJECT_ROOT"
    
    if docker build -f Dockerfile.dev -t "$IMAGE_NAME" .; then
        print_success "Development image built successfully"
    else
        print_error "Failed to build development image"
        exit 1
    fi
}

# Run interactive development container
run_container() {
    print_header "Starting Development Container"
    
    # Remove existing container if it exists
    docker rm -f "$CONTAINER_NAME" 2>/dev/null || true
    
    # Run new container with volume mount
    docker run -it --rm \
        --name "$CONTAINER_NAME" \
        -v "$PROJECT_ROOT:/home/dev/workspace/envsense" \
        -v /var/run/docker.sock:/var/run/docker.sock \
        -w /home/dev/workspace/envsense \
        "$IMAGE_NAME" \
        bash
}

# Start bash shell
start_shell() {
    print_header "Starting Bash Shell in Container"
    
    if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        docker exec -it "$CONTAINER_NAME" bash
    else
        print_warning "Container not running, starting new one..."
        run_container
    fi
}

# Start fish shell
start_fish() {
    print_header "Starting Fish Shell in Container"
    
    if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        docker exec -it "$CONTAINER_NAME" fish
    else
        print_warning "Container not running, starting new one with fish..."
        docker run -it --rm \
            --name "$CONTAINER_NAME" \
            -v "$PROJECT_ROOT:/home/dev/workspace/envsense" \
            -v /var/run/docker.sock:/var/run/docker.sock \
            -w /home/dev/workspace/envsense \
            "$IMAGE_NAME" \
            fish
    fi
}

# Run tests in container
run_tests() {
    print_header "Running Tests in Container"
    
    docker run --rm \
        -v "$PROJECT_ROOT:/home/dev/workspace/envsense" \
        -v /var/run/docker.sock:/var/run/docker.sock \
        -w /home/dev/workspace/envsense \
        "$IMAGE_NAME" \
        bash -c "cargo test --all"
}

# Test cross-compilation
test_cross_compilation() {
    print_header "Testing Cross-Compilation in Container"
    
    docker run --rm \
        -v "$PROJECT_ROOT:/home/dev/workspace/envsense" \
        -v /var/run/docker.sock:/var/run/docker.sock \
        -w /home/dev/workspace/envsense \
        "$IMAGE_NAME" \
        bash -c "./scripts/build-target.sh linux linux-cross"
}

# Clean up containers and images
clean_up() {
    print_header "Cleaning Up Docker Resources"
    
    # Remove container
    if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        docker rm -f "$CONTAINER_NAME"
        print_success "Removed container: $CONTAINER_NAME"
    fi
    
    # Remove image
    if docker images --format '{{.Repository}}' | grep -q "^${IMAGE_NAME}$"; then
        docker rmi "$IMAGE_NAME"
        print_success "Removed image: $IMAGE_NAME"
    fi
    
    print_success "Cleanup completed"
}

# Show usage
show_usage() {
    cat << EOF
Usage: $0 [command]

Docker-based development helper for envsense

Commands:
  build    Build the development Docker image
  run      Run an interactive development container
  shell    Start a bash shell in running container
  fish     Start a fish shell in running container  
  test     Run tests in container
  cross    Test cross-compilation in container
  clean    Remove development containers and images
  help     Show this help message

Examples:
  $0 build                    # Build development image
  $0 run                      # Start interactive container
  $0 cross                    # Test cross-compilation
  $0 shell                    # Open bash shell in container

The container mounts the current project directory, so changes
are reflected immediately between host and container.
EOF
}

# Main command dispatch
case "${1:-help}" in
    "build")
        build_image
        ;;
    "run")
        run_container
        ;;
    "shell")
        start_shell
        ;;
    "fish")
        start_fish
        ;;
    "test")
        run_tests
        ;;
    "cross")
        test_cross_compilation
        ;;
    "clean")
        clean_up
        ;;
    "help"|"--help"|"-h")
        show_usage
        ;;
    *)
        print_error "Unknown command: $1"
        echo
        show_usage
        exit 1
        ;;
esac
