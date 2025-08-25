#!/bin/bash
#
# Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
#

# Update assets to their latest available versions
# Usage: ./update-assets.sh [assets_dir]

set -euo pipefail

# Default parameters
ASSETS_DIR="${1:-./assets}"

# Get latest versions using jq
BOOTSTRAP_VERSION=$(curl -s https://registry.npmjs.org/bootstrap/latest | jq -r '.version')
BOOTSTRAP_ICONS_VERSION=$(curl -s https://registry.npmjs.org/bootstrap-icons/latest | jq -r '.version')
VUE_VERSION=$(curl -s https://registry.npmjs.org/vue/latest | jq -r '.version')

# Create directories
mkdir -p "$ASSETS_DIR/css" "$ASSETS_DIR/js" "$ASSETS_DIR/fonts"

# Download files
curl -sL -o "$ASSETS_DIR/css/bootstrap.min.css" \
    "https://cdn.jsdelivr.net/npm/bootstrap@$BOOTSTRAP_VERSION/dist/css/bootstrap.min.css"

curl -sL -o "$ASSETS_DIR/css/bootstrap-icons.css" \
    "https://cdn.jsdelivr.net/npm/bootstrap-icons@$BOOTSTRAP_ICONS_VERSION/font/bootstrap-icons.css"

curl -sL -o "$ASSETS_DIR/fonts/bootstrap-icons.woff2" \
    "https://cdn.jsdelivr.net/npm/bootstrap-icons@$BOOTSTRAP_ICONS_VERSION/font/fonts/bootstrap-icons.woff2"

curl -sL -o "$ASSETS_DIR/fonts/bootstrap-icons.woff" \
    "https://cdn.jsdelivr.net/npm/bootstrap-icons@$BOOTSTRAP_ICONS_VERSION/font/fonts/bootstrap-icons.woff"

curl -sL -o "$ASSETS_DIR/js/bootstrap.bundle.min.js" \
    "https://cdn.jsdelivr.net/npm/bootstrap@$BOOTSTRAP_VERSION/dist/js/bootstrap.bundle.min.js"

curl -sL -o "$ASSETS_DIR/js/vue.global.js" \
    "https://unpkg.com/vue@$VUE_VERSION/dist/vue.global.js"

# Print versions
echo "Bootstrap: $BOOTSTRAP_VERSION"
echo "Bootstrap Icons: $BOOTSTRAP_ICONS_VERSION"
echo "Vue.js: $VUE_VERSION"
