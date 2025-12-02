#!/bin/bash
# VC BC Commit Message Helper
# This script helps format commit messages according to conventional commits

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to display usage
usage() {
    echo -e "${BLUE}VC BC Commit Message Helper${NC}"
    echo ""
    echo "This tool helps you create properly formatted commit messages"
    echo "that follow the Conventional Commits specification."
    echo ""
    echo -e "${YELLOW}Usage:${NC}"
    echo "  $0 [options]"
    echo ""
    echo -e "${YELLOW}Options:${NC}"
    echo "  --interactive    Interactive mode to build commit message"
    echo "  --validate MSG   Validate a commit message"
    echo "  --examples       Show commit message examples"
    echo "  --help          Show this help message"
    echo ""
    echo -e "${YELLOW}Quick Examples:${NC}"
    echo "  feat: add new MPT optimization"
    echo "  fix(blockchain): resolve chain validation bug"
    echo "  docs: update API reference"
    echo "  test: add integration tests"
}

# Function to validate commit message
validate_commit() {
    local msg="$1"

    # Check basic format
    if [[ ! "$msg" =~ ^(feat|fix|docs|style|refactor|perf|test|chore|ci|build|revert)(\(.+\))?:[[:space:]][A-Z].*[^.]$ ]]; then
        echo -e "${RED}❌ Invalid commit message format${NC}"
        echo ""
        echo -e "${YELLOW}Expected format:${NC} type(scope): Description"
        echo -e "${YELLOW}Examples:${NC}"
        echo "  feat: add new feature"
        echo "  fix(blockchain): resolve bug"
        echo "  docs: update documentation"
        return 1
    fi

    # Check length
    if [[ ${#msg} -gt 72 ]]; then
        echo -e "${RED}❌ Commit message too long (${#msg} chars, max 72)${NC}"
        return 1
    fi

    echo -e "${GREEN}✅ Valid commit message format${NC}"
    return 0
}

# Function to show examples
show_examples() {
    echo -e "${BLUE}Conventional Commit Examples${NC}"
    echo ""
    echo -e "${GREEN}Features:${NC}"
    echo "  feat: add MPT path compression"
    echo "  feat(blockchain): implement proof-of-work"
    echo "  feat(network): add peer discovery"
    echo ""
    echo -e "${GREEN}Bug Fixes:${NC}"
    echo "  fix: resolve memory leak in MPT"
    echo "  fix(cli): handle invalid arguments gracefully"
    echo "  fix(api): correct response format"
    echo ""
    echo -e "${GREEN}Documentation:${NC}"
    echo "  docs: update installation guide"
    echo "  docs(api): add endpoint examples"
    echo ""
    echo -e "${GREEN}Testing:${NC}"
    echo "  test: add unit tests for MPT operations"
    echo "  test(integration): add network sync tests"
    echo ""
    echo -e "${GREEN}Maintenance:${NC}"
    echo "  refactor: extract common validation logic"
    echo "  style: format code with rustfmt"
    echo "  chore: update dependencies"
    echo "  ci: add security audit to workflow"
    echo ""
    echo -e "${GREEN}Performance:${NC}"
    echo "  perf: optimize proof verification"
    echo "  perf(mpt): reduce memory usage"
}

# Function for interactive mode
interactive_mode() {
    echo -e "${BLUE}VC BC Interactive Commit Message Builder${NC}"
    echo ""

    # Select commit type
    echo -e "${YELLOW}Select commit type:${NC}"
    echo "1) feat      - New feature"
    echo "2) fix       - Bug fix"
    echo "3) docs      - Documentation"
    echo "4) style     - Code style changes"
    echo "5) refactor  - Code refactoring"
    echo "6) perf      - Performance improvements"
    echo "7) test      - Testing"
    echo "8) chore     - Maintenance"
    echo "9) ci        - CI/CD changes"
    echo "0) build     - Build system changes"
    echo ""

    local type_choice
    read -p "Enter choice (1-9): " type_choice

    local commit_type
    case $type_choice in
        1) commit_type="feat" ;;
        2) commit_type="fix" ;;
        3) commit_type="docs" ;;
        4) commit_type="style" ;;
        5) commit_type="refactor" ;;
        6) commit_type="perf" ;;
        7) commit_type="test" ;;
        8) commit_type="chore" ;;
        9) commit_type="ci" ;;
        0) commit_type="build" ;;
        *) echo -e "${RED}Invalid choice${NC}"; return 1 ;;
    esac

    # Optional scope
    echo ""
    read -p "Enter scope (optional, e.g., 'blockchain', 'mpt', 'api'): " scope

    # Description
    echo ""
    read -p "Enter description: " description

    # Build commit message
    local commit_msg="$commit_type"
    if [[ -n "$scope" ]]; then
        commit_msg="$commit_msg($scope)"
    fi
    commit_msg="$commit_msg: $description"

    echo ""
    echo -e "${YELLOW}Generated commit message:${NC}"
    echo "$commit_msg"
    echo ""

    # Validate
    if validate_commit "$commit_msg"; then
        echo ""
        read -p "Use this commit message? (y/n): " confirm
        if [[ "$confirm" =~ ^[Yy]$ ]]; then
            echo ""
            echo -e "${GREEN}To commit, run:${NC}"
            echo "git commit -m \"$commit_msg\""
            echo ""
            echo -e "${BLUE}Or with a body:${NC}"
            echo "git commit -m \"$commit_msg\" -m \"Additional details...\""
        fi
    fi
}

# Main script logic
case "${1:-}" in
    --interactive|-i)
        interactive_mode
        ;;
    --validate|-v)
        if [[ -z "$2" ]]; then
            echo -e "${RED}Error: Please provide a commit message to validate${NC}"
            echo "Usage: $0 --validate \"commit message\""
            exit 1
        fi
        validate_commit "$2"
        ;;
    --examples|-e)
        show_examples
        ;;
    --help|-h|"")
        usage
        ;;
    *)
        echo -e "${RED}Unknown option: $1${NC}"
        echo ""
        usage
        exit 1
        ;;
esac
