# Linera Casino

A decentralized casino platform built on Linera with three games: Poker (Texas Hold'em), Rummy (Indian Rummy), and Roulette.

## Overview

This project implements a multi-chain casino application where game logic, token management, and player state are distributed across different chain types. Built with Rust (for Linera applications) and React (for the frontend), it showcases Linera's unique multi-chain architecture.

## Architecture

The system follows a multi-chain architecture:

- **Master Chain**: Administrative operations (mint tokens, register play chains)
- **Public Chains**: Message routing and play chain discovery
- **Play Chains**: Game execution environment (hosts active games)
- **User Chains**: Individual player state and game subscriptions

Each game (Poker, Rummy, Roulette) has its own Linera application that integrates with a shared bankroll system.

## Project Structure

```
linCasino/
├── Cargo.toml                    # Workspace configuration
├── rust-toolchain.toml          # Rust version
├── abi/                          # Shared game logic library
│   ├── src/
│   │   ├── poker.rs             # Poker game logic
│   │   ├── rummy.rs             # Rummy game logic
│   │   ├── roulette.rs          # Roulette game logic
│   │   ├── deck.rs              # Card deck utilities
│   │   ├── bet_chip_profile.rs  # Betting profiles
│   │   └── random.rs            # Random number generation
├── bankroll/                     # Extended bankroll application
├── poker/                        # Poker Linera application
├── rummy/                        # Rummy Linera application
├── roulette/                     # Roulette Linera application
└── frontend/                     # React frontend integration
    └── src/
        ├── lib/
        │   ├── linera/          # GraphQL client and queries
        │   └── games/            # Game-specific hooks
        └── components/           # React components
```

## Building

### Backend (Rust)

Build all applications for WebAssembly:

```bash
cargo build --release --target wasm32-unknown-unknown
```

Build a specific application:

```bash
cargo build -p poker --release --target wasm32-unknown-unknown
cargo build -p rummy --release --target wasm32-unknown-unknown
cargo build -p roulette --release --target wasm32-unknown-unknown
cargo build -p bankroll --release --target wasm32-unknown-unknown
```

### Frontend (React)

```bash
cd frontend
npm install
npm run dev
```

## Dependencies

- Linera SDK: `0.15.6`
- Rust: `1.86.0`
- Node.js: Latest LTS
- React: `18.2.0`
- Apollo Client: `3.8.0`

## Running with Docker

The easiest way to get started is using Docker:

1. Clone this repository and navigate to the folder:
   ```bash
   git clone <repository-url>
   cd linCasino
   ```

2. Build the frontend first:
   ```bash
   cd frontend
   npm install
   npm run build
   cd ..
   ```

3. Start the application with Docker Compose:
   ```bash
   docker compose up -d --build
   ```

4. Monitor the logs to ensure the application is ready:
   ```bash
   docker compose logs -f casino
   ```

5. Wait until you see the following message in the logs:
   ```
   Linera Casino READY!
   ```

6. Open your browser to access the multiplayer demo. Three players are available, each running on their own node with their own microchains:
   - **Player A**: [http://localhost:5173](http://localhost:5173)
   - **Player B**: [http://localhost:5174](http://localhost:5174)
   - **Player C**: [http://localhost:5175](http://localhost:5175)

Complete command:

```bash
cd frontend && npm install && npm run build && cd ..
docker compose up -d --build
docker compose logs -f casino
```

## Development

1. Start Linera network
2. Deploy applications to Linera
3. Configure frontend with app IDs and chain IDs
4. Start frontend development server

## Games

### Poker (Texas Hold'em)
- Single-player and multi-player modes
- Betting rounds: Pre-flop, Flop, Turn, River
- Hand evaluation and showdown
- Up to 8 players per table

### Rummy (Indian Rummy)
- 13-card rummy
- Meld validation (sets and sequences)
- Deadwood calculation
- Up to 6 players per game

### Roulette
- European Roulette (0-36)
- Multiple bet types (number, color, even/odd, low/high, dozen, column)
- Payout calculations
- Real-time spinning and results

## License

[Your License Here]

