#!/bin/bash

# Build and optimize WASM for Soroban deployment

set -e

echo "🔨 Building Scavngr Contract..."

# Build WASM
echo "Building WASM..."
cargo build --target wasm32-unknown-unknown --release

# Optimize WASM
echo "Optimizing WASM..."
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/stellar_scavngr_contract.wasm

echo "✅ Build complete!"
echo "📦 Optimized WASM: target/wasm32-unknown-unknown/release/stellar_scavngr_contract.optimized.wasm"
