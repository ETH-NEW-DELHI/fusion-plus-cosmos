#!/bin/bash

# Deploy script for fusion-plus-cosmos contracts
set -e

echo "Building contracts..."

# Build all contracts
cargo build --release --target wasm32-unknown-unknown

echo "Optimizing contracts..."

# Optimize contracts
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.11

echo "Contracts optimized successfully!"

# Deploy to testnet (example with Osmosis testnet)
echo "Deploying to testnet..."

# Deploy EscrowFactory
osmosisd tx wasm store artifacts/escrow_factory.wasm \
  --from validator \
  --chain-id osmo-test-4 \
  --gas auto \
  --gas-adjustment 1.3 \
  --yes

# Deploy EscrowDst
osmosisd tx wasm store artifacts/escrow_dst.wasm \
  --from validator \
  --chain-id osmo-test-4 \
  --gas auto \
  --gas-adjustment 1.3 \
  --yes

echo "Deployment completed!"
