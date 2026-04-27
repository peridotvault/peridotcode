#!/bin/bash
# Test script for PeridotCode file generation

echo "=== PeridotCode File Generation Test ==="
echo ""

# Create test directory
TEST_DIR="$HOME/test-peridot-$(date +%s)"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "Test directory: $TEST_DIR"
echo ""

# Set logging
export RUST_LOG=info

echo "Running peridotcode..."
echo "When the TUI opens, type:"
echo "  Create a platformer game"
echo ""
echo "Then check if files were created in: $TEST_DIR"
echo ""

# Run peridotcode
"$(dirname "$0")/target/release/peridotcode.exe"

echo ""
echo "=== Checking results ==="
echo "Files in test directory:"
ls -la "$TEST_DIR" 2>/dev/null || dir "$TEST_DIR"

if [ -f "$TEST_DIR/index.html" ]; then
    echo ""
    echo "✅ SUCCESS! Files were created:"
    find "$TEST_DIR" -type f | head -20
else
    echo ""
    echo "❌ No files were created"
    echo "Check the logs above for errors"
fi

echo ""
echo "Test directory: $TEST_DIR"
