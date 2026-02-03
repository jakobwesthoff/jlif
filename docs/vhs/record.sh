#!/bin/bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# record.sh - Prepares demo environment, runs VHS recording, and cleans up
#
# Usage: ./record.sh
#
# This script:
# 1. Creates a temporary directory with the mock log script
# 2. Runs VHS to record the demo
# 3. Cleans up the temporary directory
#
# Output files (demo.webm, demo.mp4) are moved to docs/pages/assets/.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEMO_DIR="/tmp/jlif-demo"

# Setup
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cp "$SCRIPT_DIR/mock-logs.sh" "$DEMO_DIR/"
chmod +x "$DEMO_DIR/mock-logs.sh"

# Record
cd "$SCRIPT_DIR"
vhs demo.tape

# Move recordings to assets folder
ASSETS_DIR="$SCRIPT_DIR/../pages/assets"
mv demo.webm "$ASSETS_DIR/"
mv demo.mp4 "$ASSETS_DIR/"

# Cleanup
rm -rf "$DEMO_DIR"

echo "Recording complete. Output files in docs/pages/assets/: demo.webm, demo.mp4"
