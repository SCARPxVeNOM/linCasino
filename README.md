# Linera Casino

Linera Casino is a multi-chain casino built on [Linera](https://linera.io/) that demonstrates real-time multiplayer games (Poker, Rummy, Roulette) with cross-chain state, shared bankroll, and a React/Vite frontend.

## Features
- Real-time multiplayer Poker (Texas Hold'em) with authoritative play-chain state and timeout handling
- Rummy and Roulette implementations sharing a common bankroll application
- Multi-chain topology: master/public/play/user chains for routing, execution, and balances
- GraphQL services for game state and actions, plus optional Croissant wallet support

## Repository Layout
- `abi/` – shared game logic (poker, rummy, roulette, deck, RNG)
- `bankroll/` – token management contract and GraphQL service
- `poker/`, `rummy/`, `roulette/` – game applications
- `frontend/` – React + TypeScript UI (Vite) and static web bundles
- `croissant/` – extension and WASM client utilities (optional)
- `run.bash` – end-to-end setup against Linera Testnet Conway
- `compose.yaml` / `Dockerfile` – containerized runner for the whole stack

## Prerequisites
- Rust 1.86+ with `wasm32-unknown-unknown` target
- `linera-service` and `linera-storage-service` 0.15.7 on PATH
- Node.js 18+ (npm) for the frontend; `pnpm` optional for Croissant
- Docker Desktop (if using Compose)
- `jq` and `curl` (run.bash uses them), `http-server` is installed via npm when building the Docker image

## Quick Start (Docker)
1) Install dependencies and build the frontend once:
```bash
cd frontend
npm install
npm run build
cd ..
```
2) Launch the stack:
```bash
docker compose up -d --build
```
The container runs `run.bash`, deploys apps to Testnet Conway, generates per-player configs, and hosts static sites.

3) Visit the players:
- Player 1: http://localhost:5173
- Player 2: http://localhost:5174
- Player 3: http://localhost:5175

GraphQL services: http://localhost:8081–8083 (one per player wallet).

## Manual Run (local host)
1) Build contracts:
```bash
cargo build --release --target wasm32-unknown-unknown
```
2) Ensure `linera-service` and `linera-storage-service` 0.15.7 are installed, then run:
```bash
bash run.bash
```
The script:
- creates wallets via the Conway faucet,
- publishes bankroll, poker, rummy, roulette apps,
- spins three Linera services on ports 8081/8082/8083,
- writes `frontend/web_{a,b,c}/config.json`,
- serves the built frontend at 5173/5174/5175 using `http-server`.

If you want live frontend dev instead of the static servers:
```bash
cd frontend
npm install
npm run dev
```
Point `frontend/public/config.json` to your desired node URLs/app IDs when developing manually.

## Development Notes
- Build specific apps: `cargo build -p bankroll -p poker -p rummy -p roulette --release --target wasm32-unknown-unknown`
- Frontend production build: `npm run build` (outputs to `frontend/dist`)
- The run script copies `frontend/dist` into `frontend/web_a|b|c`; rebuild before re-running if UI changes.
- Default ports: 5173/5174/5175 for web, 8081/8082/8083 for GraphQL, 8080 faucet proxy.

## Multiplayer Poker Flow
- Authoritative state lives on a play chain (`PokerState.game`)
- Users `Sit` to join; actions (`Bet`, `Call`, `Fold`, etc.) are validated against `hand_id` and `current_player`
- Heartbeats enforce deadlines; stale/conflicting actions are rejected deterministically
- Bankroll sync on sit/leave/settlement; frontend polls GraphQL every ~800ms

## Troubleshooting
- Services down in Docker: `docker compose logs -f casino`
- Port conflicts: adjust mappings in `compose.yaml`
- GraphQL errors: confirm config JSON matches deployed app IDs and chain IDs
- Re-run after UI changes: rebuild the frontend before `run.bash` or `docker compose up --build`

## Contributing
PRs welcome—keep Rust/TS code formatted, ensure WASM builds succeed, and verify the frontend builds before submitting.
