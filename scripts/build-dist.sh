#!/bin/bash
set -e

# Change directory to the workspace root
cd "$(dirname "$0")/.."

echo "Building spy-code binary in release mode..."
cargo build --release

# Determine target binary path (supports macOS/Linux and Windows if running in git bash)
if [ -f "target/release/spy-code" ]; then
    BINARY_PATH="target/release/spy-code"
    EXE=""
elif [ -f "target/release/spy-code.exe" ]; then
    BINARY_PATH="target/release/spy-code.exe"
    EXE=".exe"
else
    echo "Error: Compiled binary not found in target/release!"
    exit 1
fi

echo "Copying binary to NPM package structure..."
mkdir -p npm/bin
cp "$BINARY_PATH" "npm/bin/spy-code$EXE"
chmod +x "npm/bin/spy-code$EXE"
echo "NPM package binary setup complete."

echo "Copying binary to Python package structure..."
mkdir -p python-package/spy_code
cp "$BINARY_PATH" "python-package/spy_code/spy-code$EXE"
chmod +x "python-package/spy_code/spy-code$EXE"
echo "Python package binary setup complete."

echo "Dist preparation completed successfully! You can now test packages locally."
echo "For NPM: cd npm && npm link"
echo "For Python: cd python-package && pip install -e ."
