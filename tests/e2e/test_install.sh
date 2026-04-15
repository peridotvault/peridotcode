#!/bin/bash
# 
# Usage: ./tests/e2e/test_install.sh
# 
# Validates that peridotcode can be installed from the repository using `cargo install`
# and that the binary executes correctly and reports a version.

set -e

echo "Running Alpha End-to-End Installation Test..."

# Capture current directory
ROOT_DIR=$(pwd)

# Install into a local temp bin path so we don't clobber the user's global bin if they care
TEST_BIN_DIR=$(mktemp -d)
export CARGO_INSTALL_ROOT=$TEST_BIN_DIR

echo "1/2: Running cargo install --path crates/cli"
if cargo install --path crates/cli; then
    echo "✓ cargo install succeeded"
else
    echo "✗ cargo install failed"
    exit 1
fi

echo "2/2: Verifying peridotcode executable version"
PERIDOT_BIN="${TEST_BIN_DIR}/bin/peridotcode"

if [ ! -f "$PERIDOT_BIN" ]; then
    echo "✗ ERROR: peridotcode binary not found at $PERIDOT_BIN"
    exit 1
fi

if "$PERIDOT_BIN" --version; then
    echo "✓ Executable verified."
else
    echo "✗ ERROR: Unable to run peridotcode --version"
    exit 1
fi

echo "Cleaning up..."
rm -rf "$TEST_BIN_DIR"

echo "All installation tests passed successfully."
exit 0
