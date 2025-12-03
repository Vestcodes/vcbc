#!/bin/bash

# VCBC Commit Analysis Script
# Analyzes recent commits to determine if a release should be created
# Groups commits by conventional commit types and determines release type

set -e

# Configuration
MAX_COMMITS=10
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Function to analyze commit messages
analyze_commits() {
    local branch="$1"
    local base_ref="${2:-HEAD~10}"

    log_info "Analyzing last $MAX_COMMITS commits on branch '$branch'..."

    # Get recent commits
    local commits
    commits=$(git log --oneline -n "$MAX_COMMITS" 2>/dev/null)

    if [ -z "$commits" ]; then
        log_warn "No commits to analyze"
        echo "should_release=false"
        echo "release_type=none"
        echo "breaking_changes=false"
        echo "features=false"
        echo "fixes=false"
        echo "performance=false"
        echo "refactoring=false"
        return 0
    fi

    # Filter for conventional commits
    local conventional_commits
    conventional_commits=$(echo "$commits" | grep -E "^[a-f0-9]+ (feat|fix|perf|refactor|breaking|chore|docs|test|style|ci)" || true)

    if [ -z "$conventional_commits" ]; then
        log_warn "No conventional commits found in the last $MAX_COMMITS commits"
        echo "should_release=false"
        echo "release_type=none"
        echo "breaking_changes=false"
        echo "features=false"
        echo "fixes=false"
        echo "performance=false"
        echo "refactoring=false"
        return 0
    fi

    commits="$conventional_commits"

    log_info "Found commits to analyze:"
    echo "$commits" | while read -r commit; do
        echo "  $commit"
    done

    # Analyze commit types
    local breaking_changes=false
    local features=false
    local fixes=false
    local performance=false
    local refactoring=false

    # Check each commit message
    while IFS= read -r line; do
        # Extract commit message (remove hash)
        local message=$(echo "$line" | sed 's/^[a-f0-9]* //')

        # Check for breaking changes
        if echo "$message" | grep -q "BREAKING\|breaking\|!:"; then
            breaking_changes=true
            log_info "Found breaking change: $message"
        fi

        # Check commit types
        if echo "$message" | grep -q "^feat"; then
            features=true
            log_info "Found feature: $message"
        elif echo "$message" | grep -q "^fix"; then
            fixes=true
            log_info "Found fix: $message"
        elif echo "$message" | grep -q "^perf"; then
            performance=true
            log_info "Found performance improvement: $message"
        elif echo "$message" | grep -q "^refactor"; then
            refactoring=true
            log_info "Found refactoring: $message"
        fi
    done <<< "$commits"

    # Determine release type
    local should_release=false
    local release_type="patch"

    if [ "$breaking_changes" = true ]; then
        should_release=true
        release_type="major"
        log_success "Breaking changes detected - MAJOR release needed"
    elif [ "$features" = true ]; then
        should_release=true
        release_type="minor"
        log_success "New features detected - MINOR release needed"
    elif [ "$fixes" = true ] || [ "$performance" = true ] || [ "$refactoring" = true ]; then
        should_release=true
        release_type="patch"
        log_success "Fixes/performance/refactoring detected - PATCH release needed"
    fi

    # Output results for GitHub Actions
    echo "should_release=$should_release"
    echo "release_type=$release_type"
    echo "breaking_changes=$breaking_changes"
    echo "features=$features"
    echo "fixes=$fixes"
    echo "performance=$performance"
    echo "refactoring=$refactoring"
}

# Function to check if commits were already released
check_existing_releases() {
    log_info "Checking for existing releases..."

    # Get the latest tag
    local latest_tag
    if latest_tag=$(git describe --tags --abbrev=0 2>/dev/null); then
        log_info "Latest tag: $latest_tag"

        # Check if there are commits since the latest tag
        local commits_since_tag
        commits_since_tag=$(git rev-list "$latest_tag..HEAD" --count 2>/dev/null || echo "0")

        if [ "$commits_since_tag" -eq 0 ]; then
            log_warn "No commits since latest tag $latest_tag"
            echo "commits_since_last_tag=0"
            return 1  # Indicate no new commits
        else
            log_info "$commits_since_tag commits since last tag"
            echo "commits_since_last_tag=$commits_since_tag"
        fi
    else
        log_info "No existing tags found - first release"
        echo "commits_since_last_tag=all"
    fi
    return 0
}

# Main function
main() {
    local branch="${1:-development}"

    cd "$PROJECT_ROOT"

    log_info "VCBC Commit Analysis - Branch: $branch"

    # Check existing releases
    if ! check_existing_releases; then
        # No new commits since last tag
        echo "should_release=false"
        echo "release_type=none"
        echo "breaking_changes=false"
        echo "features=false"
        echo "fixes=false"
        echo "performance=false"
        echo "refactoring=false"
        return 0
    fi

    # Analyze commits
    analyze_commits "$branch"
}

# Run main function with branch argument
main "$1"
