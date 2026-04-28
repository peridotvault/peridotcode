#!/bin/bash

# PeridotCode Alpha Verification Script
# This script tests the end-to-end happy path for the Installable Alpha.

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "--- PeridotCode Alpha Verification ---"

# 1. Check binary on PATH
if command -v peridotcode >/dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC}: peridotcode binary found on PATH"
else
    echo -e "${RED}FAIL${NC}: peridotcode binary not found on PATH. Run 'cargo install --path .' first."
    exit 1
fi

# 2. Check API Key
if [ -n "$OPENROUTER_API_KEY" ]; then
    echo -e "${GREEN}PASS${NC}: OPENROUTER_API_KEY is set"
else
    echo -e "${RED}FAIL${NC}: OPENROUTER_API_KEY environment variable is not set."
    exit 1
fi

# 3. Check Doctor
if peridotcode doctor >/dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC}: peridotcode doctor checks passed"
else
    echo -e "${RED}FAIL${NC}: peridotcode doctor failed. Check your Node.js/npm installation."
    exit 1
fi

# 4. Run Inference (Happy Path)
echo "Running test inference (this may take 20-40 seconds)..."
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

if peridotcode infer "make a simple 2D platformer with a blue player and red enemies" > inference.log 2>&1; then
    echo -e "${GREEN}PASS${NC}: Inference command completed successfully"
else
    echo -e "${RED}FAIL${NC}: Inference command failed. See $TEMP_DIR/inference.log for details."
    cat inference.log
    exit 1
fi

# 5. Check Output Files
MISSING=0
FILES=("index.html" "package.json" "src/main.js")
for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}PASS${NC}: Found $file"
    else
        echo -e "${RED}FAIL${NC}: Missing $file"
        MISSING=1
    fi
done

if [ $MISSING -eq 0 ]; then
    echo -e "\n${GREEN}ALL CHECKS PASSED!${NC} PeridotCode Alpha is functional."
else
    echo -e "\n${RED}VERIFICATION FAILED.${NC} Some files were not generated."
    exit 1
fi

# Cleanup
cd - >/dev/null
rm -rf "$TEMP_DIR"
