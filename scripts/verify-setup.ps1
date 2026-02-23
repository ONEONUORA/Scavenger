# Scavngr Project Setup Verification Script (PowerShell)

Write-Host "🔍 Verifying Scavngr Project Setup..." -ForegroundColor Cyan
Write-Host ""

# Check Rust installation
Write-Host "Checking Rust installation..." -ForegroundColor Yellow
if (Get-Command rustc -ErrorAction SilentlyContinue) {
    $rustVersion = rustc --version
    Write-Host "✅ Rust installed: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "❌ Rust not found. Install from https://rustup.rs" -ForegroundColor Red
    exit 1
}

# Check WASM target
Write-Host "Checking WASM target..." -ForegroundColor Yellow
$targets = rustup target list
if ($targets -match "wasm32-unknown-unknown \(installed\)") {
    Write-Host "✅ WASM target installed" -ForegroundColor Green
} else {
    Write-Host "⚠️  WASM target not installed. Installing..." -ForegroundColor Yellow
    rustup target add wasm32-unknown-unknown
}

# Check Soroban CLI
Write-Host "Checking Soroban CLI..." -ForegroundColor Yellow
if (Get-Command soroban -ErrorAction SilentlyContinue) {
    $sorobanVersion = soroban --version
    Write-Host "✅ Soroban CLI installed: $sorobanVersion" -ForegroundColor Green
} else {
    Write-Host "⚠️  Soroban CLI not found. Install with:" -ForegroundColor Yellow
    Write-Host "   cargo install --locked soroban-cli --features opt"
}

# Check project structure
Write-Host ""
Write-Host "Checking project structure..." -ForegroundColor Yellow
$requiredFiles = @(
    "Cargo.toml",
    "stellar-contract/Cargo.toml",
    "stellar-contract/src/lib.rs",
    "stellar-contract/src/types.rs",
    ".gitignore",
    "soroban.toml",
    "README.md"
)

foreach ($file in $requiredFiles) {
    if (Test-Path $file) {
        Write-Host "✅ $file" -ForegroundColor Green
    } else {
        Write-Host "❌ $file missing" -ForegroundColor Red
    }
}

# Try to build
Write-Host ""
Write-Host "Attempting to build project..." -ForegroundColor Yellow
$buildOutput = cargo build 2>&1 | Out-String
if ($buildOutput -match "Finished") {
    Write-Host "✅ Project builds successfully" -ForegroundColor Green
} else {
    Write-Host "⚠️  Build encountered issues" -ForegroundColor Yellow
}

# Try to run tests
Write-Host ""
Write-Host "Running tests..." -ForegroundColor Yellow
$testOutput = cargo test 2>&1 | Out-String
if ($testOutput -match "test result: ok") {
    Write-Host "✅ All tests passed" -ForegroundColor Green
} else {
    Write-Host "⚠️  Some tests failed" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "✨ Setup verification complete!" -ForegroundColor Cyan
