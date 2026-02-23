# Build and optimize WASM for Soroban deployment (PowerShell)

Write-Host "🔨 Building Scavngr Contract..." -ForegroundColor Cyan

# Build WASM
Write-Host "Building WASM..." -ForegroundColor Yellow
cargo build --target wasm32-unknown-unknown --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed" -ForegroundColor Red
    exit 1
}

# Optimize WASM
Write-Host "Optimizing WASM..." -ForegroundColor Yellow
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/stellar_scavngr_contract.wasm

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Optimization failed" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Build complete!" -ForegroundColor Green
Write-Host "📦 Optimized WASM: target/wasm32-unknown-unknown/release/stellar_scavngr_contract.optimized.wasm" -ForegroundColor Cyan
