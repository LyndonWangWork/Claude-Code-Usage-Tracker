#!/bin/bash

# Release script for Claude Code Usage Tracker
# This script updates version numbers, commits changes, and creates a git tag

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

TAURI_CONF="$PROJECT_ROOT/src-tauri/tauri.conf.json"
CARGO_TOML="$PROJECT_ROOT/src-tauri/Cargo.toml"

# Check if files exist
if [ ! -f "$TAURI_CONF" ]; then
    echo -e "${RED}Error: tauri.conf.json not found at $TAURI_CONF${NC}"
    exit 1
fi

if [ ! -f "$CARGO_TOML" ]; then
    echo -e "${RED}Error: Cargo.toml not found at $CARGO_TOML${NC}"
    exit 1
fi

# Get current version from tauri.conf.json
CURRENT_VERSION=$(grep -o '"version": "[^"]*"' "$TAURI_CONF" | head -1 | cut -d'"' -f4)
echo -e "${YELLOW}Current version: ${GREEN}$CURRENT_VERSION${NC}"

# Prompt for new version
echo -n "Enter new version (without 'v' prefix): "
read NEW_VERSION

# Validate version input
if [ -z "$NEW_VERSION" ]; then
    echo -e "${RED}Error: Version cannot be empty${NC}"
    exit 1
fi

# Confirm
echo -e "\n${YELLOW}Will update version from ${GREEN}$CURRENT_VERSION${YELLOW} to ${GREEN}$NEW_VERSION${NC}"
echo -n "Continue? (y/n): "
read CONFIRM

if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
    echo -e "${YELLOW}Cancelled${NC}"
    exit 0
fi

# Update tauri.conf.json
echo -e "\n${YELLOW}Updating tauri.conf.json...${NC}"
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" "$TAURI_CONF"
else
    # Linux/Windows (Git Bash)
    sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" "$TAURI_CONF"
fi
echo -e "${GREEN}✓ Updated tauri.conf.json${NC}"

# Update Cargo.toml (only the package version, not dependencies)
echo -e "${YELLOW}Updating Cargo.toml...${NC}"
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "0,/^version = \"$CURRENT_VERSION\"/s//version = \"$NEW_VERSION\"/" "$CARGO_TOML"
else
    # Linux/Windows (Git Bash)
    sed -i "0,/^version = \"$CURRENT_VERSION\"/s//version = \"$NEW_VERSION\"/" "$CARGO_TOML"
fi
echo -e "${GREEN}✓ Updated Cargo.toml${NC}"

# Git operations
echo -e "\n${YELLOW}Committing changes...${NC}"
cd "$PROJECT_ROOT"
git add "$TAURI_CONF" "$CARGO_TOML"
git commit -m "chore: bump version to $NEW_VERSION"
echo -e "${GREEN}✓ Committed changes${NC}"

# Create tag
TAG_NAME="v$NEW_VERSION"
echo -e "${YELLOW}Creating tag $TAG_NAME...${NC}"
git tag "$TAG_NAME"
echo -e "${GREEN}✓ Created tag $TAG_NAME${NC}"

# Done
echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}Release preparation complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "\nTo push the release, run:"
echo -e "  ${YELLOW}git push && git push origin $TAG_NAME${NC}"
