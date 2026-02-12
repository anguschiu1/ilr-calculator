#!/bin/bash
# Update wasm-pack to the latest version.
# Option A: Official installer (prebuilt binary, no Rust version requirement)
# Option B: cargo install (requires Rust 1.86+ for wasm-pack 0.14+)

set -e

echo "Updating wasm-pack..."

if command -v wasm-pack &> /dev/null; then
  echo "Current version: $(wasm-pack --version 2>/dev/null || true)"
fi

# Prefer official installer (works with any Rust version)
echo "Using official wasm-pack installer..."
curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh -s -- -f

echo ""
echo "Done. New version: $(wasm-pack --version)"
