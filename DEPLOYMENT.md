# Deployment Guide

This guide explains how to deploy the Linera Casino application.

## Prerequisites

1. Install Linera CLI tools (version 0.15.7 to match Testnet Conway)
2. Install Rust with `wasm32-unknown-unknown` target
3. Install Node.js and npm

**Important**: Make sure your Linera CLI version matches the testnet version (0.15.7). You can check your version with:
```bash
linera --version
```

If you need to update, install the correct version:
```bash
cargo install --locked linera-service@0.15.7
cargo install --locked linera-storage-service@0.15.7
```

## Build Steps

### 1. Build Backend Applications

```bash
# Build all applications
cargo build --release --target wasm32-unknown-unknown

# Or build individually
cargo build -p abi --release --target wasm32-unknown-unknown
cargo build -p bankroll --release --target wasm32-unknown-unknown
cargo build -p poker --release --target wasm32-unknown-unknown
cargo build -p rummy --release --target wasm32-unknown-unknown
cargo build -p roulette --release --target wasm32-unknown-unknown
```

### 2. Connect to Testnet Conway

The project is configured to connect to Testnet Conway. The deployment script (`run.bash`) uses the testnet faucet at:
```
https://faucet.testnet-conway.linera.net/
```

No local network setup is required - the script will automatically connect to the testnet when deploying.

### 3. Deploy Applications

Deploy applications in this order:
1. Bankroll
2. Poker
3. Rummy
4. Roulette

Example deployment command:
```bash
linera service publish bankroll_contract.wasm bankroll_service.wasm
linera service publish poker_contract.wasm poker_service.wasm
linera service publish rummy_contract.wasm rummy_service.wasm
linera service publish roulette_contract.wasm roulette_service.wasm
```

### 4. Configure Frontend

Create `frontend/public/config.json`:

```json
{
  "nodeServiceURL": "http://localhost:8080",
  "pokerAppId": "<poker_app_id>",
  "rummyAppId": "<rummy_app_id>",
  "rouletteAppId": "<roulette_app_id>",
  "bankrollAppId": "<bankroll_app_id>",
  "defaultChain": "<default_chain_id>"
}
```

### 5. Start Frontend

```bash
cd frontend
npm install
npm run dev
```

## Multi-Chain Setup

For multi-chain deployment, you'll need to:
1. Create master chain
2. Create public chains
3. Create play chains
4. Configure each application with chain IDs

Refer to the reference project (`microcard-cross-app`) for detailed multi-chain setup.

