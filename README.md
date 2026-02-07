# Linera Casino

Linera Casino is a multi-chain casino built on [Linera](https://linera.io/) that demonstrates real-time multiplayer games (Poker, Rummy, Roulette) with cross-chain state, shared bankroll, and a React/Vite frontend.

## Features

### Core Games
- **Poker** (Texas Hold'em) – Real-time multiplayer with side pots, rake collection, and timeout handling
- **Roulette** – European wheel with inside bets (Split, Street, Corner, Line, Basket) and call bets (Voisins, Tiers, Orphelins, Neighbors)
- **Rummy** – Card matching game sharing the common bankroll

### Provably Fair System
- Commit-reveal RNG scheme for verifiable randomness
- Server seeds hashed before play; client seeds accepted for additional entropy
- Full verification proofs available post-game

### Staking & Profit Sharing
- Stake tokens to earn a share of house profits (rake)
- Real-time reward calculation based on stake proportion
- Claim rewards at any time

### Responsible Gaming
- Daily loss limits (self-set)
- Maximum single bet limits
- Self-exclusion periods (1-365 days)
- Automatic enforcement at the contract level

### VIP System
- Five tiers: Bronze → Silver → Gold → Platinum → Diamond
- Tier based on lifetime wagered amount
- Higher tiers receive bonus multipliers on rewards

### Governance & Admin
- Admin can pause/unpause games
- Configurable rake percentages and caps
- Operator roles for delegated management

## Repository Layout
```
abi/           – Shared game logic (poker, rummy, roulette, deck, provably_fair, audit, tournament)
bankroll/      – Token management, staking, limits, and governance
poker/         – Poker application with side pots and rake
rummy/         – Rummy card game
roulette/      – Roulette with extended bet types
frontend/      – React + TypeScript UI (Vite)
croissant/     – Extension and WASM client utilities (optional)
run.bash       – End-to-end setup against Linera Testnet Conway
```

## Prerequisites
- Rust nightly-2025-01-15 (specified in `rust-toolchain.toml`) with `wasm32-unknown-unknown` target
- `linera-service` and `linera-storage-service` 0.15.7 on PATH
- Node.js 18+ (npm) for the frontend
- Docker Desktop (if using Compose)
- `jq` and `curl` for `run.bash`

## Quick Start (Docker)
```bash
# Build frontend
cd frontend && npm install && npm run build && cd ..

# Launch
docker-compose up -d --build

# View logs
docker-compose logs -f
```

**Access:**
- Player 1: http://localhost:5173
- Player 2: http://localhost:5174
- Player 3: http://localhost:5175
- GraphQL: http://localhost:8081–8083

## Manual Run
```bash
# Build contracts
cargo build --release --target wasm32-unknown-unknown

# Deploy and run
bash run.bash
```

## GraphQL API

### Bankroll Queries
```graphql
query {
  getStakingPool { totalStaked, stakerCount, rewardPerToken }
  getStakerInfo(owner: "...") { stakedAmount, unclaimedRewards }
  getPlayerLimits(owner: "...") { dailyLossLimit, selfExclusionUntil }
  getCasinoConfig { pokerRakePercent, pausedGames }
}
```

### Game Queries
```graphql
query {
  singlePlayerData { game { pot, sidePots, rakeCollected, clientSeed } }
}
```

## Multiplayer Poker Flow
1. Authoritative state lives on a play chain
2. Users `Sit` to join; actions validated against `hand_id` and `current_player`
3. Side pots calculated automatically for all-in scenarios
4. Rake collected before pot distribution
5. Bankroll sync on sit/leave/settlement

## Troubleshooting
- **Services down**: `docker-compose logs -f linera-casino`
- **Port conflicts**: Adjust in `docker-compose.yaml`
- **GraphQL errors**: Verify config JSON matches deployed app/chain IDs

## Contributing
PRs welcome. Keep Rust/TS formatted, ensure WASM builds, and verify frontend builds before submitting.
