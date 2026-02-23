#!/bin/bash

# Scavngr Project Setup Verification Script

echo "🔍 Verifying Scavngr Project Setup..."
echo ""

# Check Rust installation
echo "Checking Rust installation..."
if command -v rustc &> /dev/null; then
    echo "✅ Rust installed: $(rustc --version)"
else
    echo "❌ Rust not found. Install from https://rustup.rs"
    exit 1
fi

# Check WASM target
echo "Checking WASM target..."
if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "✅ WASM target installed"
else
    echo "⚠️  WASM target not installed. Installing..."
    rustup target add wasm32-unknown-unknown
fi

# Check Soroban CLI
echo "Checking Soroban CLI..."
if command -v soroban &> /dev/null; then
    echo "✅ Soroban CLI installed: $(soroban --version)"
else
    echo "⚠️  Soroban CLI not found. Install with:"
    echo "   cargo install --locked soroban-cli --features opt"
fi

# Check project structure
echo ""
echo "Checking project structure..."
required_files=(
    "Cargo.toml"
    "stellar-contract/Cargo.toml"
    "stellar-contract/src/lib.rs"
    "stellar-contract/src/types.rs"
    ".gitignore"
    "soroban.toml"
    "README.md"
)

for file in "${required_files[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file"
    else
        echo "❌ $file missing"
    fi
done

# Try to build
echo ""
echo "Attempting to build project..."
if cargo build 2>&1 | grep -q "Finished"; then
    echo "✅ Project builds successfully"
else
    echo "⚠️  Build encountered issues (check output above)"
fi

# Try to run tests
echo ""
echo "Running tests..."
if cargo test 2>&1 | grep -q "test result: ok"; then
    echo "✅ All tests passed"
else
    echo "⚠️  Some tests failed (check output above)"
fi

echo ""
echo "✨ Setup verification complete!"
