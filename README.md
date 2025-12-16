# Linera Casino

A decentralized, multi-chain casino platform built on [Linera](https://linera.io/) featuring real-time multiplayer poker, rummy, and roulette games with integrated token management and cross-chain state synchronization.

## 🎯 Overview

Linera Casino demonstrates a production-ready multi-chain gaming platform where game logic, player state, and token management are distributed across different chain types. The platform features a **fully synchronized, multi-user, real-time poker room** with authoritative state management, proper turn order enforcement, conflict handling, timeout management, and seamless Bankroll integration.

### Key Features

- **🃏 Real-Time Multiplayer Poker**: Fully synchronized Texas Hold'em with authoritative play-chain state, turn-based betting, and automatic timeout handling
- **🎲 Multiple Game Types**: Poker (Texas Hold'em), Rummy (Indian Rummy), and Roulette (European)
- **🔗 Multi-Chain Architecture**: Leverages Linera's unique microchain model for scalable, distributed game execution
- **💰 Integrated Bankroll System**: Unified token management across all games with balance synchronization
- **⚡ Real-Time Updates**: GraphQL-based state synchronization with polling for live game state
- **🔄 Robust Reconnection**: Automatic state recovery and seamless reconnection after disconnects

## 🏗️ Architecture

The system follows Linera's multi-chain architecture pattern:

```
┌─────────────────────────────────────────────────────────────┐
│                    Multi-Chain Architecture                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Master Chain ──────► Administrative operations            │
│     │                      (mint tokens, register chains)  │
│     │                                                         │
│     ├──► Public Chains ───► Message routing & discovery     │
│     │                                                         │
│     ├──► Play Chains ──────► Authoritative game state        │
│     │                      (single table per play chain)     │
│     │                                                         │
│     └──► User Chains ──────► Individual player state         │
│                              (balance, game subscriptions)   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Chain Responsibilities

- **Master Chain**: Administrative operations (mint tokens, register play chains, manage lobbies)
- **Public Chains**: Message routing and play chain discovery
- **Play Chains**: Authoritative game execution environment (hosts active poker tables)
- **User Chains**: Individual player state, balance management, and game subscriptions

### Multiplayer Poker Protocol

The poker room implements a robust distributed protocol:

- **Authoritative State**: Single source of truth on the play chain (`PokerState.game`)
- **Turn Order Enforcement**: Strict validation of player actions based on `current_player` and `hand_id`
- **Conflict Resolution**: Deterministic handling of concurrent actions via state-based validation
- **Timeout Management**: Heartbeat-based system with automatic fold/check on timeout
- **Bankroll Integration**: Seamless balance synchronization at sit/leave and hand settlement

## 📁 Project Structure

```
linCasino/
├── Cargo.toml                    # Workspace configuration
├── rust-toolchain.toml          # Rust version (1.86.0)
├── compose.yaml                  # Docker Compose configuration
├── run.bash                      # Manual deployment script
│
├── abi/                          # Shared game logic library
│   ├── src/
│   │   ├── poker.rs             # Poker game logic & state machine
│   │   ├── rummy.rs             # Rummy game logic
│   │   ├── roulette.rs          # Roulette game logic
│   │   ├── deck.rs              # Card deck utilities
│   │   ├── bet_chip_profile.rs  # Betting profiles
│   │   └── random.rs            # Random number generation
│
├── bankroll/                     # Bankroll application
│   ├── src/
│   │   ├── contract.rs          # Token management contract
│   │   ├── service.rs           # GraphQL service
│   │   └── state.rs             # Bankroll state
│
├── poker/                        # Poker Linera application
│   ├── src/
│   │   ├── contract.rs          # Poker contract (operations & messages)
│   │   ├── service.rs           # GraphQL queries & mutations
│   │   ├── state.rs             # Poker state (per-chain)
│   │   └── lib.rs               # ABI definitions
│
├── rummy/                        # Rummy Linera application
├── roulette/                     # Roulette Linera application
│
└── frontend/                     # React + TypeScript frontend
    ├── src/
    │   ├── lib/
    │   │   ├── linera/          # GraphQL client & queries
    │   │   └── games/            # Game-specific hooks (usePoker, etc.)
    │   └── components/
    │       └── games/           # Game UI components
    ├── package.json
    └── vite.config.ts
```

## 🚀 Quick Start

### Prerequisites

- **Docker Desktop** (for containerized deployment)
- **Node.js** LTS (18.x or higher) - for frontend development
- **Rust** 1.86.0+ (for contract development)
- **Linera SDK** 0.15.6

### Option 1: Docker Deployment (Recommended)

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd linCasino
   ```

2. **Build the frontend**:
   ```bash
   cd frontend
   npm install
   npm run build
   cd ..
   ```

3. **Start all services**:
   ```bash
   docker compose up -d --build
   ```

4. **Monitor startup**:
   ```bash
   docker compose logs -f casino
   ```
   Wait for: `Linera Casino READY!`

5. **Access the casino**:
   - **Player 1**: http://localhost:5173
   - **Player 2**: http://localhost:5174
   - **Player 3**: http://localhost:5175

### Option 2: Manual Development Setup

1. **Build Rust contracts**:
   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```

2. **Deploy to Linera network**:
   ```bash
   bash run.bash
   ```

3. **Start frontend development server**:
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

## 🎮 Games

### 🃏 Poker (Texas Hold'em)

**Features**:
- **Single-Player Mode**: Play against AI with full betting rounds
- **Multi-Player Mode**: Real-time synchronized tables with up to 8 players
- **Betting Rounds**: Pre-flop → Flop → Turn → River → Showdown
- **Hand Evaluation**: Full poker hand ranking (straight flush, four of a kind, etc.)
- **Turn Management**: Strict turn order with timeout handling
- **Conflict Resolution**: Deterministic action validation

**Multiplayer Protocol**:
- Players join via `Sit` operation (fetches balance from Bankroll)
- Actions sent as cross-chain messages to authoritative play chain
- State synchronized via GraphQL polling (800ms interval)
- Automatic timeout handling via heartbeat system
- Hand settlement with Bankroll integration

**Actions**:
- `Fold` - Surrender the current hand
- `Check` - Pass when no bet to call
- `Call` - Match the current bet
- `Bet` - Place initial bet in a round
- `Raise` - Increase the current bet
- `All-In` - Bet all remaining chips

### 🃎 Rummy (Indian Rummy)

- 13-card rummy variant
- Meld validation (sets and sequences)
- Deadwood calculation
- Up to 6 players per game

### 🎲 Roulette (European)

- European Roulette (0-36)
- Multiple bet types:
  - Number bets (straight up)
  - Color bets (red/black)
  - Even/Odd
  - Low/High (1-18 / 19-36)
  - Dozen bets
  - Column bets
- Real-time spinning and payout calculations

## 🔧 Development

### Building Contracts

Build all applications:
```bash
cargo build --release --target wasm32-unknown-unknown
```

Build specific application:
```bash
cargo build -p poker --release --target wasm32-unknown-unknown
cargo build -p bankroll --release --target wasm32-unknown-unknown
```

### Frontend Development

```bash
cd frontend
npm install
npm run dev          # Development server with hot reload
npm run build        # Production build
```

### Testing Multiplayer Poker

1. **Start three browser windows** (or use Docker ports 5173, 5174, 5175)
2. **Navigate to Poker** in each window
3. **Select "Multiplayer" mode**
4. **Click "Sit at Table"** in each window (enter player names)
5. **Once 2+ players seated**, the game starts automatically
6. **Test actions**: Bet, Raise, Call, Fold
7. **Observe real-time synchronization** across all windows

### GraphQL Endpoints

- **Player 1**: http://localhost:8081/chains/{chainId}/applications/{appId}
- **Player 2**: http://localhost:8082/chains/{chainId}/applications/{appId}
- **Player 3**: http://localhost:8083/chains/{chainId}/applications/{appId}

### Key GraphQL Queries

```graphql
# Get multiplayer table state
query GetMultiPlayerTable {
  multiPlayerData {
    userStatus
    game {
      handId
      status
      currentRound
      pot
      currentPlayer
      actionDeadlineMicros
      players {
        id
        name
        chips
        currentBet
        isFolded
        isAllIn
      }
      communityCards
    }
  }
}

# Get player profile
query GetProfile {
  getProfile {
    balance
    betData {
      minBet
      maxBet
    }
  }
}
```

### Key Mutations

```graphql
# Create a poker table
mutation CreateTable($smallBlind: String!, $bigBlind: String!, $maxPlayers: Int!) {
  createTable(smallBlind: $smallBlind, bigBlind: $bigBlind, maxPlayers: $maxPlayers)
}

# Sit at table
mutation Sit($tableChain: String!, $name: String!) {
  sit(tableChain: $tableChain, name: $name)
}

# Player action
mutation PlayerAction(
  $tableChain: String!
  $handId: String!
  $seatId: Int!
  $action: String!
  $amount: String
) {
  playerAction(
    tableChain: $tableChain
    handId: $handId
    seatId: $seatId
    action: $action
    amount: $amount
  )
}

# Heartbeat (for timeout management)
mutation Heartbeat($tableChain: String!) {
  heartbeat(tableChain: $tableChain)
}
```

## 📚 Technical Details

### Multiplayer Poker Implementation

**State Management**:
- `PokerGame` struct on play chain contains authoritative table state
- Fields: `hand_id`, `min_raise`, `action_deadline_micros` for turn management
- Helper methods: `next_active_player()`, `is_betting_round_complete()`

**Operation Flow**:
1. User chain: `Sit` operation → fetches balance → sends `JoinTable` message
2. Play chain: `handle_join_table()` → assigns seat → initializes chips
3. User chain: `PlayerAction` operation → sends `ApplyAction` message
4. Play chain: `handle_apply_action()` → validates turn → applies action → advances state
5. Frontend: Polls `multiPlayerData` query every 800ms for updates

**Conflict Handling**:
- Actions validated against `hand_id`, `current_player`, and `action_deadline_micros`
- Stale actions rejected silently (state already advanced)
- Deterministic ordering via Linera's message serialization

**Timeout System**:
- `Heartbeat` operation triggers timeout checks
- If `now > action_deadline_micros`: auto-fold (if bet to call) or auto-check
- New deadline set when advancing to next player

**Bankroll Integration**:
- `Sit`: Calls `BankrollOperation::Balance` to get buy-in amount
- `Leave`: Computes net gain/loss, calls `BankrollOperation::UpdateBalance`
- Hand settlement: Pot distributed, balances updated

### Dependencies

**Backend**:
- `linera-sdk`: 0.15.6
- `async-graphql`: 7.0.17
- `serde`: 1.0.228

**Frontend**:
- `react`: 18.2.0
- `@apollo/client`: 3.8.0
- `vite`: 4.5.14
- `typescript`: Latest

## 🐛 Troubleshooting

### Docker Issues

**Services not starting**:
```bash
docker compose down
docker compose up -d --build
docker compose logs -f casino
```

**Port conflicts**:
- Check if ports 5173-5175, 8080-8083 are available
- Modify `compose.yaml` if needed

### Build Issues

**Rust compilation errors**:
```bash
cargo clean
cargo build --release --target wasm32-unknown-unknown
```

**Frontend build errors**:
```bash
cd frontend
rm -rf node_modules package-lock.json
npm install
npm run build
```

### Runtime Issues

**GraphQL connection errors**:
- Verify Linera services are running: `docker compose ps`
- Check service logs: `docker compose logs casino`
- Ensure frontend `config.json` has correct chain IDs and app IDs

**Multiplayer not syncing**:
- Verify all players are querying the same play chain
- Check browser console for GraphQL errors
- Ensure polling is enabled (800ms interval)

## 📖 Documentation

- [HOW_TO_RUN.md](HOW_TO_RUN.md) - Detailed setup instructions
- [QUICKSTART.md](QUICKSTART.md) - Quick start guide
- [DEPLOYMENT.md](DEPLOYMENT.md) - Deployment procedures
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) - Feature status

## 🤝 Contributing

Contributions are welcome! Please ensure:

1. Code follows Rust/TypeScript best practices
2. All contracts compile to WASM successfully
3. Frontend builds without errors
4. Multiplayer protocol maintains consistency
5. Tests pass (when test suite is added)

## 📄 License

[Your License Here]

## 🙏 Acknowledgments

- Built on [Linera](https://linera.io/) - The multi-chain blockchain platform
- Uses [Async-GraphQL](https://async-graphql.github.io/) for GraphQL support
- Frontend powered by [React](https://react.dev/) and [Vite](https://vitejs.dev/)

---

**Built with ❤️ using Linera's multi-chain architecture**
