# 🎰 Running the Linera Casino Project

## Quick Start

The project is already running! Here's how to access it:

### 🌐 Access the Frontend

Open your web browser and navigate to:

- **Player 1**: http://localhost:5173
- **Player 2**: http://localhost:5174  
- **Player 3**: http://localhost:5175

### 🔧 Backend Services

- **GraphQL Service (Player 1)**: http://localhost:8081
- **GraphQL Service (Player 2)**: http://localhost:8082
- **GraphQL Service (Player 3)**: http://localhost:8083
- **GraphiQL IDE**: http://localhost:8082 (for testing GraphQL queries)
- **Faucet Service**: http://localhost:8080

## Current Status

✅ **Container Status**: Running and Healthy
✅ **Frontend**: Served on ports 5173-5175
✅ **Backend**: GraphQL services on ports 8081-8083
✅ **Applications**: All games published and ready

## How to Use

1. **Open the Casino**: Navigate to http://localhost:5173 in your browser
2. **Select a Game**: Choose from Poker, Rummy, or Roulette
3. **Start Playing**: 
   - Enter your name
   - Click "Start Game"
   - Use the game controls to play

## Managing the Project

### Check Status
```powershell
docker compose ps
```

### View Logs
```powershell
docker compose logs -f casino
```

### Restart Services
```powershell
docker compose restart casino
```

### Stop Services
```powershell
docker compose down
```

### Start Services (if stopped)
```powershell
docker compose up -d
```

### Rebuild and Restart
```powershell
docker compose down
docker compose up -d --build
```

## Troubleshooting

### If services are not accessible:

1. **Check Docker is running**:
   ```powershell
   docker ps
   ```

2. **Check container logs**:
   ```powershell
   docker compose logs casino --tail 50
   ```

3. **Restart the container**:
   ```powershell
   docker compose restart casino
   ```

### If frontend shows errors:

1. **Check config.json exists** in the frontend directories
2. **Verify GraphQL services are running** on ports 8081-8083
3. **Check browser console** for specific error messages

## Game Features

### 🃏 Poker
- Texas Hold'em gameplay
- Bet, Call, Raise, Fold actions
- Real-time game state

### 🃎 Rummy
- Card matching and melds
- Draw from deck or discard pile
- Declare to win

### 🎲 Roulette
- Multiple bet types (Red, Black, Numbers, etc.)
- Spin the wheel
- View betting history

## Next Steps

1. Open http://localhost:5173 in your browser
2. Start playing your favorite game!
3. Check the GraphiQL IDE at http://localhost:8082 to explore the GraphQL API

Enjoy your casino! 🎰

