# CLAUDE.md

This file provides guidance when working with code in this repository.

## Project Overview

This is a decentralized casino platform built on the Linera blockchain platform. The project implements a multi-chain architecture where game logic, token management, and player state are distributed across different chain types. It includes three games: Poker (Texas Hold'em), Rummy (Indian Rummy), and Roulette.

## Build and Development Commands

### Building the Project

```bash
# Build all workspace members for WebAssembly target
cargo build --release --target wasm32-unknown-unknown

# Build a specific package
cargo build -p poker --release --target wasm32-unknown-unknown
cargo build -p rummy --release --target wasm32-unknown-unknown
cargo build -p roulette --release --target wasm32-unknown-unknown
cargo build -p bankroll --release --target wasm32-unknown-unknown
cargo build -p abi --release --target wasm32-unknown-unknown

# Check code without building
cargo check

# Run clippy for linting
cargo clippy --target wasm32-unknown-unknown

# Format code
cargo fmt
```

### Testing

```bash
# Run tests
cargo test
```

### Frontend Development

```bash
cd frontend
npm install
npm run dev
```

### Docker Deployment

```bash
# Build frontend first
cd frontend && npm install && npm run build && cd ..

# Start with Docker Compose
docker compose up -d --build

# View logs
docker compose logs -f casino
```

## Architecture

### Workspace Structure

The project is organized as a Cargo workspace with multiple crates:

- **`abi/`**: Shared data structures, types, and game logic used by all applications
    - Poker game state and rules (`poker.rs`)
    - Rummy game logic (`rummy.rs`)
    - Roulette wheel logic (`roulette.rs`)
    - Deck and card handling (`deck.rs`)
    - Betting and chip profiles (`bet_chip_profile.rs`)
    - Random number generation (`random.rs`)

- **`bankroll/`**: Token and balance management application
    - Handles user balances and daily bonuses
    - Mints tokens on master chain
    - Provides balance queries via GraphQL
    - Supports multiple game types

- **`poker/`**: Poker game application
    - Implements Texas Hold'em
    - Single-player and multi-player game modes
    - Manages table seats and game state
    - Integrates with bankroll for balance management

- **`rummy/`**: Rummy game application
    - Implements Indian Rummy (13-card)
    - Meld validation and scoring
    - Single-player and multi-player modes

- **`roulette/`**: Roulette game application
    - Implements European Roulette
    - Multiple bet types and payout calculations
    - Real-time spinning

### Linera Multi-Chain Architecture

This application uses Linera's multi-chain messaging system with four distinct chain types:

1. **Master Chain**: Administrative operations
    - Mints tokens for all applications
    - Adds play chains to public chains
    - Requires authorization via chain ID validation

2. **Public Chains**: Message routing and discovery
    - Players send `FindPlayChain` messages to discover available game chains
    - Routes `AddPlayChain` messages to register new play chains
    - Acts as a directory service

3. **Play Chains**: Game execution environment
    - Hosts active games for each game type
    - Manages table seats and game rounds
    - Broadcasts game state via event streams

4. **User Chains**: Individual player state
    - Stores user status (Idle, FindPlayChain, InGame, etc.)
    - Maintains connection to assigned play chain
    - Handles subscribe/unsubscribe operations

### Contract and Service Pattern

Each application follows Linera's contract-service architecture:

- **Contract** (`contract.rs`): Executes operations that modify blockchain state
    - Processes `Operation` types (e.g., `PokerOperation`, `RummyOperation`, `RouletteOperation`)
    - Sends and receives cross-chain messages
    - Emits events for state changes

- **Service** (`service.rs`): Read-only GraphQL query interface
    - Provides queries for frontend applications
    - Does not modify state

## Important Constants and Configuration

- Rust toolchain: `1.86.0` (see `rust-toolchain.toml`)
- Linera SDK version: `0.15.6` (see `Cargo.toml`)
- Maximum players: Poker (8), Rummy (6)
- Daily bonus: 25,000 tokens (24-hour cooldown)

## WebAssembly Compilation

All contract and service binaries must compile to `wasm32-unknown-unknown`:

- Use `#![cfg_attr(target_arch = "wasm32", no_main)]` attribute in contract/service files
- Custom random number generation required (`getrandom` with custom feature)
- No standard library threading or file I/O available

## Cross-Application Calls

All game applications call into the bankroll application:

- Uses `ApplicationId<BankrollAbi>` in game parameters
- Calls `BankrollOperation::Balance` and `BankrollOperation::UpdateBalance`
- Response types: `BankrollResponse::Balance(Amount)` or `BankrollResponse::Ok`
- Game type tracking for multi-game support

Example pattern in game contracts:

```rust
let balance_response = self.runtime
    .call_application(/* bankroll operation */)
    .await;
```

## Frontend Integration

The frontend uses Apollo Client to connect to Linera GraphQL services:

- GraphQL queries for game state
- GraphQL mutations for game actions
- React hooks: `usePoker()`, `useRummy()`, `useRoulette()`
- Configuration loaded from `config.json` generated by deployment script

