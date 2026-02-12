#!/bin/bash
# Build script for WebAssembly compilation
# This script compiles the Rust code to WebAssembly using wasm-pack

set -e

echo "Building WebAssembly binary..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack is not installed. Installing..."
    cargo install wasm-pack
fi

# Build for web target
wasm-pack build --target web --out-dir pkg

echo "WASM build complete! Output is in the 'pkg' directory."
echo ""
echo "To use in JavaScript:"
echo "  import init, { calculate_rolling_absences } from './pkg/ilr_calculator.js';"
echo "  await init();"
echo "  const result = calculate_rolling_absences(jsonString);"
