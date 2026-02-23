# Scavngr - Stellar Recycling Platform

A decentralized recycling platform built on Stellar blockchain using Soroban smart contracts. Scavngr connects recyclers, collectors, and manufacturers in a transparent and efficient ecosystem.

## Project Structure

```
Scavenger/
├── stellar-contract/      # Soroban smart contract (Rust)
│   ├── src/
│   │   ├── lib.rs        # Main contract implementation
│   │   └── types.rs      # ParticipantRole enum and types
│   └── Cargo.toml
├── frontend/             # React frontend (to be implemented)
├── .github/workflows/    # CI/CD pipelines
├── Cargo.toml           # Workspace configuration
├── soroban.toml         # Soroban CLI configuration
└── README.md
```

## Features

- **Role-Based Participant System**: Recycler, Collector, and Manufacturer roles
- **Participant Registration**: On-chain participant management
- **Role Validation**: Permission checks for different actions
- **Soroban Storage**: Efficient on-chain data storage

## Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Soroban CLI
- Stellar account with XLM (for deployment)

## Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install --locked soroban-cli --features opt
```

## Build

```bash
# Build the contract
cargo build --release

# Build WASM
cd stellar-contract
cargo build --target wasm32-unknown-unknown --release

# Optimize WASM
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/stellar_scavngr_contract.wasm
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Deployment

### Local (Standalone Network)

```bash
# Start Stellar standalone
docker run --rm -it -p 8000:8000 \
  stellar/quickstart:latest --standalone --enable-soroban-rpc

# Deploy contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_scavngr_contract.optimized.wasm \
  --source <YOUR_SECRET_KEY> \
  --network standalone
```

### Testnet

```bash
# Generate keypair
soroban keys generate testnet-deployer

# Fund account
curl "https://friendbot.stellar.org?addr=$(soroban keys address testnet-deployer)"

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_scavngr_contract.optimized.wasm \
  --source testnet-deployer \
  --network testnet
```

## Contract API

### ParticipantRole Enum

```rust
pub enum ParticipantRole {
    Recycler = 0,      // Can collect and process recyclables
    Collector = 1,     // Can collect materials
    Manufacturer = 2,  // Can manufacture products
}
```

### Functions

- `register_participant(address, role)` - Register new participant
- `get_participant(address)` - Get participant info
- `update_role(address, new_role)` - Update participant role
- `can_collect(address)` - Check collection permission
- `can_manufacture(address)` - Check manufacturing permission

## Development

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Watch for changes
cargo watch -x test
```

## CI/CD

GitHub Actions automatically:
- Runs tests on push/PR
- Checks code formatting
- Runs clippy linting
- Builds optimized WASM
- Uploads build artifacts

## License

MIT License - see LICENSE file for details
