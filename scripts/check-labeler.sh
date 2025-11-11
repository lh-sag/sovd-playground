#!/bin/bash
# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

# Check that all workspace crates have corresponding entries in .github/labeler.yml
# Usage: ./check-labeler.sh

set -euo pipefail

metadata=$(cargo metadata --format-version 1 --no-deps)
# Get project root
PROJECT_ROOT=$(echo "$metadata" | jq -r '.workspace_root')
# Get workspace crate labels (without sovd- prefix)
workspace_crates=$(echo "$metadata" | jq -r '.workspace_members[] | split("/") | last | split("#")[0] | sub("^sovd-"; "")')

missing=()
for crate in $workspace_crates; do
    if ! grep -q "^crate:$crate:" "$PROJECT_ROOT/.github/labeler.yml"; then
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
        echo "  - 'sovd-$crate/**'"
        echo
    done
    exit 1
fi
