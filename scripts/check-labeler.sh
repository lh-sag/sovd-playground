#!/bin/bash

# Check that all workspace crates have corresponding entries in .github/labeler.yaml
# Usage: ./check-labeler.sh

set -euo pipefail

metadata=$(cargo metadata --format-version 1 --no-deps)
# Get project root
PROJECT_ROOT=$(echo "$metadata" | jq -r '.workspace_root')
# Get workspace crates
workspace_crates=$(echo "$metadata" | jq -r '.workspace_members[]' | sed 's/.*\/\([^#]*\)#.*/\1/')

missing=()
for crate in $workspace_crates; do
    if ! grep -q "^crate:$crate:" "$PROJECT_ROOT/.github/labeler.yaml"; then
        missing+=("$crate")
    fi
done

if [ ${#missing[@]} -eq 0 ]; then
    echo "All crates have labeler entries"
    exit 0
else
    echo "Missing labeler entries for: ${missing[*]}"
    echo "Add these to .github/labeler.yaml:"
    for crate in "${missing[@]}"; do
        echo "crate:$crate:"
        echo "  - '$crate/**'"
        echo
    done
    exit 1
fi
