# 🎰 How to Run the Linera Casino Project

## Prerequisites

Before running the project, make sure you have:

1. **Docker Desktop** installed and running
   - Download from: https://www.docker.com/products/docker-desktop
   - Make sure Docker Desktop is **running** before proceeding

2. **Node.js** (for building frontend)
   - Download from: https://nodejs.org/
   - Version: LTS (18.x or higher)

## Quick Start (Recommended)

### Step 1: Build the Frontend

Open a terminal in the project directory and run:

```powershell
cd frontend
npm install
npm run build
cd ..
```

**Note**: This builds the React frontend into the `frontend/dist/` directory.

### Step 2: Start with Docker

```powershell
docker compose up -d --build
```

This will:
- Build the Docker container with all Linera services
- Deploy all game applications (Poker, Rummy, Roulette)
- Start GraphQL services
- Set up the frontend web servers

### Step 3: Wait for Services to Start

Monitor the logs to see when everything is ready:

```powershell
docker compose logs -f casino
```

**Wait until you see**: `Linera Casino READY!`

This may take 2-5 minutes on first run as it:
- Compiles Rust code to WebAssembly
- Deploys applications to Linera network
- Sets up wallets and chains
- Starts all services

### Step 4: Access the Casino

Once ready, open your browser and visit:

- **Player 1**: http://localhost:5173
- **Player 2**: http://localhost:5174
- **Player 3**: http://localhost:5175

## What You'll See

1. **Casino Royale Main Menu** - Choose from three games:
   - 🃏 **Texas Hold'em Poker**
   - 🃎 **Indian Rummy**
   - 🎲 **Royal Roulette**

2. **Game Selection** - Click any game card to start playing

3. **Game Interface** - Each game has:
   - Balance display
   - Game controls
   - Real-time game state
   - All actions connected to Linera blockchain

## Managing the Project

### Check Status

```powershell
docker compose ps
```

### View Logs

```powershell
# All logs
docker compose logs -f casino

# Last 50 lines
docker compose logs casino --tail 50
```

### Stop Services

```powershell
docker compose down
```

### Restart Services

```powershell
docker compose restart casino
```

### Rebuild After Code Changes

```powershell
# Rebuild frontend
cd frontend
npm run build
cd ..

# Rebuild and restart Docker
docker compose down
docker compose up -d --build
```

## Troubleshooting

### Docker Not Running

**Error**: `unable to get image` or `cannot connect to Docker`

**Solution**: 
1. Open Docker Desktop
2. Wait for it to fully start (whale icon should be steady)
3. Try again

### Frontend Not Loading

**Error**: Blank page or connection errors

**Solutions**:
1. **Hard refresh browser**: `Ctrl + Shift + R` (Windows) or `Cmd + Shift + R` (Mac)
2. **Check container is running**:
   ```powershell
   docker compose ps
   ```
3. **Check logs for errors**:
   ```powershell
   docker compose logs casino --tail 100
   ```
4. **Rebuild frontend**:
   ```powershell
   cd frontend
   npm run build
   cd ..
   docker compose restart casino
   ```

### Build Errors

**Error**: `cannot allocate memory` during Docker build

**Solutions**:
1. **Increase Docker Desktop memory**:
   - Docker Desktop → Settings → Resources → Memory
   - Set to at least 4GB (8GB recommended)
   - Click "Apply & Restart"

2. **Clean Docker cache**:
   ```powershell
   docker system prune -a
   ```

### Port Already in Use

**Error**: `port is already allocated`

**Solutions**:
1. **Find what's using the port**:
   ```powershell
   netstat -ano | findstr :5173
   ```
2. **Stop the conflicting service** or change ports in `compose.yaml`

### Services Not Ready

**Error**: GraphQL errors or "service unavailable"

**Solutions**:
1. **Wait longer** - First build can take 5-10 minutes
2. **Check all services are running**:
   ```powershell
   docker compose ps
   ```
3. **Restart services**:
   ```powershell
   docker compose restart casino
   ```

## Alternative: Manual Setup (Advanced)

If you prefer not to use Docker, you can set up manually:

1. **Install Linera SDK**:
   ```bash
   cargo install --locked linera-service@0.15.6 linera-storage-service@0.15.6
   ```

2. **Build Rust applications**:
   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```

3. **Run deployment script**:
   ```bash
   bash run.bash
   ```

4. **Start frontend**:
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

## Game Features

### 🃏 Poker (Texas Hold'em)
- **Actions**: Bet, Call, Raise, Fold
- **Game Flow**: Pre-flop → Flop → Turn → River → Showdown
- **Balance**: Synced with Linera bankroll

### 🃎 Rummy (Indian Rummy)
- **Actions**: Draw from Deck, Draw from Discard, Discard Card, Declare
- **Game Flow**: Deal 13 cards → Draw → Discard → Form Melds → Declare
- **Balance**: Synced with Linera bankroll

### 🎲 Roulette
- **Actions**: Place Bet (Red/Black/Numbers), Spin
- **Game Flow**: Place Bets → Spin → Calculate Winnings
- **Balance**: Synced with Linera bankroll

## Backend Services

The project runs these services:

- **GraphQL Service (Player 1)**: http://localhost:8081
- **GraphQL Service (Player 2)**: http://localhost:8082
- **GraphQL Service (Player 3)**: http://localhost:8083
- **Faucet Service**: http://localhost:8080
- **Frontend (Player 1)**: http://localhost:5173
- **Frontend (Player 2)**: http://localhost:5174
- **Frontend (Player 3)**: http://localhost:5175

## Next Steps

1. ✅ Open http://localhost:5173 in your browser
2. ✅ Select a game (Poker, Rummy, or Roulette)
3. ✅ Start playing!

**Enjoy your casino! 🎰**

