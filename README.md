# Fusion Plus Cosmos

Implementation of 1inch Fusion Plus protocol in CosmWasm to enable cross-chain atomic swaps from Ethereum to Cosmos ecosystems.

## Project Structure

```
fusion-plus-cosmos/
├── contracts/
│   ├── escrow-factory/          # Factory contract
│   ├── escrow-dst/              # Destination escrow
│   ├── shared/                  # Shared libraries
│   └── interfaces/              # Contract interfaces
├── schemas/                     # JSON schemas
├── tests/                       # Tests
├── scripts/                     # Deployment scripts
└── architecture.md              # Architecture documentation
```

## Contracts

### EscrowFactory
- Deploys EscrowDst instances
- Manages escrow creation and configuration
- Provides deterministic address computation

### EscrowDst
- Holds resolver tokens on Cosmos chain
- Implements withdrawal, cancellation, and rescue functions
- Supports time-based access control

### Shared
- Common types and utilities
- Immutables and Timelocks structures
- Cryptographic functions (Keccak256, SHA256)

## Development

### Prerequisites
- Rust 1.70+
- CosmWasm toolchain
- Docker (for optimization)

### Build
```bash
cargo build --release --target wasm32-unknown-unknown
```

### Test
```bash
cargo test
```

### Optimize
```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.11
```

### Deploy
```bash
./scripts/deploy.sh
```

## Protocol Flow

1. Resolver fills order on Ethereum (creates EscrowSrc via Limit Order Protocol)
2. Resolver creates EscrowDst on Cosmos (deposits ATOM tokens)
3. Secret is revealed to unlock tokens on both chains
4. User gets ATOM on Cosmos, resolver gets ETH on Ethereum

## Key Features

- **No IBC Required**: Coordination-based swaps using hashlock/secret pattern
- **Resolver Pre-funding**: Resolver maintains liquidity on both chains
- **Token Support**: CW20, native tokens, and IBC denoms
- **Time-based Access**: Private/public withdrawal and cancellation periods
- **Safety Deposits**: Native token incentives for proper execution

## Security

- Keccak256 compatibility with Ethereum
- Secret-based validation
- Timelock enforcement
- Role-based access control
- Emergency rescue functions
