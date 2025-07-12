#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    log_error "Not in a git repository"
    exit 1
fi

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    log_error "Working directory is not clean. Please commit or stash your changes."
    exit 1
fi

# Get current version from git tags
CURRENT_VERSION=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "0.1.0")
log_info "Current version: v${CURRENT_VERSION}"

# Parse version components
IFS='.' read -r -a VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]:-0}
MINOR=${VERSION_PARTS[1]:-0}
PATCH=${VERSION_PARTS[2]:-0}

# Function to update version in files
update_version_files() {
    local new_version=$1

    # Update Cargo.toml
    if [[ -f "server/Cargo.toml" ]]; then
        sed -i.bak "s/^version = \".*\"/version = \"${new_version}\"/" server/Cargo.toml
        rm server/Cargo.toml.bak
        log_success "Updated server/Cargo.toml"
    fi

    # Update Cargo.lock by running cargo check
    if [[ -f "server/Cargo.lock" ]]; then
        cd server && cargo check --quiet
        cd ..
        log_success "Updated server/Cargo.lock"
    fi

    # Update package.json
    if [[ -f "web/package.json" ]]; then
        # Use a more robust sed command for JSON
        if command -v jq >/dev/null 2>&1; then
            jq ".version = \"${new_version}\"" web/package.json > web/package.json.tmp && mv web/package.json.tmp web/package.json
        else
            sed -i.bak "s/\"version\": \".*\"/\"version\": \"${new_version}\"/" web/package.json
            rm web/package.json.bak
        fi
        log_success "Updated web/package.json"
    fi
}

# Main release function
create_release() {
    local version_type=$1
    local new_version=""

    case $version_type in
        "patch")
            new_version="${MAJOR}.${MINOR}.$((PATCH + 1))"
            ;;
        "minor")
            new_version="${MAJOR}.$((MINOR + 1)).0"
            ;;
        "major")
            new_version="$((MAJOR + 1)).0.0"
            ;;
        *)
            # Custom version provided
            if [[ $version_type =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                new_version=$version_type
            else
                log_error "Invalid version format. Use: major, minor, patch, or X.Y.Z"
                exit 1
            fi
            ;;
    esac

    log_info "Creating release v${new_version}"

    # Update version in files
    update_version_files "$new_version"

    # Commit version changes
    git add server/Cargo.toml server/Cargo.lock web/package.json 2>/dev/null || true
    git commit -m "chore: bump version to v${new_version}" || log_warning "No version files to commit"

    # Create and push tag
    git tag -a "v${new_version}" -m "Release v${new_version}"

    log_success "Created tag v${new_version}"
    log_info "Push the tag to trigger the release:"
    echo "  git push origin v${new_version}"
    echo ""
    log_info "Or push everything including the version commit:"
    echo "  git push && git push --tags"
}

# Show usage
show_usage() {
    echo "Usage: $0 [patch|minor|major|X.Y.Z]"
    echo ""
    echo "Examples:"
    echo "  $0 patch      # v${MAJOR}.${MINOR}.$((PATCH + 1))"
    echo "  $0 minor      # v${MAJOR}.$((MINOR + 1)).0"
    echo "  $0 major      # v$((MAJOR + 1)).0.0"
    echo "  $0 1.2.3      # v1.2.3"
    echo ""
    echo "Current version: v${CURRENT_VERSION}"
}

# Main script logic
if [[ $# -eq 0 ]]; then
    show_usage
    exit 0
fi

case $1 in
    "-h"|"--help")
        show_usage
        ;;
    *)
        create_release "$1"
        ;;
esac
