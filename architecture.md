# Fusion Plus Cosmos Architecture

## Overview

Implementation of 1inch Fusion Plus protocol in CosmWasm to enable cross-chain atomic swaps from Ethereum to Cosmos ecosystems.

## Protocol Flow

### Ethereum -> Cosmos Swap
1. Resolver fills order on Ethereum (creates EscrowSrc via Limit Order Protocol)
2. Resolver creates EscrowDst on Cosmos (deposits ATOM tokens)
3. Secret is revealed to unlock tokens on both chains
4. User gets ATOM on Cosmos, resolver gets ETH on Ethereum

## Contract Structure

```
fusion-plus-cosmos/
├── contracts/
│   ├── escrow-factory/          # Factory contract
│   ├── escrow-dst/              # Destination escrow
│   ├── shared/                  # Shared libraries
│   └── interfaces/              # Contract interfaces
├── schemas/                     # JSON schemas
├── tests/                       # Tests
└── scripts/                     # Deployment scripts
```

## Core Contracts

### EscrowFactory (Cosmos)
- **Purpose**: Deploy EscrowDst instances
- **Key Functions**:
  - `create_escrow_dst()` - Create destination escrow
  - `compute_escrow_address()` - Deterministic address computation

### EscrowDst (Ethereum -> Cosmos)
- **Purpose**: Lock resolver tokens, unlock with secret
- **Key Functions**:
  - `withdraw()` - Private withdrawal with secret
  - `public_withdraw()` - Public withdrawal after timelock
  - `cancel()` - Cancel escrow and return tokens
  - `rescue_funds()` - Emergency fund recovery

## Implementation Phases

### Phase 1: Core Infrastructure
1. Set up CosmWasm project structure
2. Implement data structures (Immutables, Timelocks)
3. Create shared libraries

### Phase 2: Escrow Contracts
1. Implement EscrowFactory contract
2. Implement EscrowDst contract
3. Add timelock and access control logic
4. Implement deterministic address computation

### Phase 3: More Features
1. Implement partial fill support with Merkle tree validation
2. Add fee handling (protocol and integrator fees)
3. Add emergency and rescue functions
4. Implement token handling (CW20 and native tokens)

### Phase 5: Testing and Optimization
1. Comprehensive unit testing
2. Integration testing with Ethereum