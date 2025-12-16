import React, { useState, useEffect } from 'react';
import { usePoker } from '../../lib/games/poker';
import { ArrowLeft, Users, User } from 'lucide-react';
import './Game.css';

type GameMode = 'single' | 'multi';

export default function PokerGame({ onExit }: { onExit: () => void }) {
  const [gameMode, setGameMode] = useState<GameMode | null>(() => {
    const saved = localStorage.getItem('poker_game_mode');
    return (saved as GameMode) || null;
  });
  const { game, multiGame, profile, loading, lobbies, actions } = usePoker(gameMode ?? 'single');
  const [playerName, setPlayerName] = useState('');
  const [betAmount, setBetAmount] = useState('');
  const [multiBetAmount, setMultiBetAmount] = useState('');
  const [multiSeatId, setMultiSeatId] = useState('');

  // Save game mode to localStorage
  useEffect(() => {
    if (gameMode) {
      localStorage.setItem('poker_game_mode', gameMode);
    }
  }, [gameMode]);

  if (loading) {
    return <div className="game-loading">Loading game data...</div>;
  }

  const handleStartGame = () => {
    if (playerName.trim()) {
      actions.startGame(playerName.trim());
      setPlayerName('');
    }
  };

  const handleGetBalance = async () => {
    await actions.getBalance();
  };

  // Simple heartbeat while in multiplayer mode and table is active
  useEffect(() => {
    if (gameMode !== 'multi') return;
    if (!multiGame) return;
    const id = setInterval(() => {
      actions.sendHeartbeat();
    }, 5000);
    return () => clearInterval(id);
  }, [gameMode, multiGame, actions]);

  const handleRequestFaucet = async () => {
    try {
      const response = await fetch('http://localhost:8080', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({})
      });
      if (response.ok) {
        setTimeout(async () => {
          await actions.getBalance();
        }, 2000);
      }
    } catch (error) {
      console.error('Faucet error:', error);
    }
  };

  // Game Mode Selection Screen
  if (!gameMode) {
    return (
      <div className="flex flex-col h-screen bg-[#1e293b] font-sans overflow-hidden text-white items-center justify-center">
        <div className="bg-[#0f172a] border border-emerald-500/50 rounded-2xl max-w-md w-full p-8 shadow-2xl">
          <h2 className="text-2xl font-bold text-emerald-400 text-center mb-6">Select Game Mode</h2>
          <div className="space-y-4">
            <button
              onClick={() => setGameMode('single')}
              className="w-full p-4 bg-emerald-900/50 hover:bg-emerald-900 border border-emerald-500/30 rounded-lg flex items-center gap-4 transition-all"
            >
              <User size={24} className="text-emerald-400" />
              <div className="text-left">
                <div className="font-bold text-white">Single Player</div>
                <div className="text-sm text-slate-400">Play against AI</div>
              </div>
            </button>
            <button
              onClick={() => setGameMode('multi')}
              className="w-full p-4 bg-indigo-900/50 hover:bg-indigo-900 border border-indigo-500/30 rounded-lg flex items-center gap-4 transition-all"
            >
              <Users size={24} className="text-indigo-400" />
              <div className="text-left">
                <div className="font-bold text-white">Multiplayer</div>
                <div className="text-sm text-slate-400">Play with other players</div>
              </div>
            </button>
          </div>
          <button
            onClick={onExit}
            className="mt-6 w-full py-2 text-slate-400 hover:text-white text-sm"
          >
            ← Back to Lobby
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="game-container">
      <div className="game-sidebar">
        <div className="profile-section">
          <div className="flex items-center justify-between mb-4">
            <h3>Your Profile</h3>
            <button onClick={onExit} className="text-slate-400 hover:text-white">
              <ArrowLeft size={20} />
            </button>
          </div>
          <div className="mb-2">
            <div className="flex gap-2 mb-2">
              <button
                onClick={() => setGameMode('single')}
                className={`flex-1 p-2 rounded text-xs ${gameMode === 'single' ? 'bg-emerald-600 text-white' : 'bg-slate-700 text-slate-300'}`}
              >
                <User size={14} className="mx-auto mb-1" />
                Single
              </button>
              <button
                onClick={() => setGameMode('multi')}
                className={`flex-1 p-2 rounded text-xs ${gameMode === 'multi' ? 'bg-indigo-600 text-white' : 'bg-slate-700 text-slate-300'}`}
              >
                <Users size={14} className="mx-auto mb-1" />
                Multi
              </button>
            </div>
          </div>
          {profile ? (
            <>
              <div className="balance">
                <span>Balance:</span>
                <span className="amount">{(Number(profile.balance) / 1e9).toFixed(2)} LIN</span>
              </div>
              <button onClick={handleGetBalance} className="action-button">
                Refresh Balance
              </button>
              {(Number(profile.balance) / 1e9) < 10 && (
                <button onClick={handleRequestFaucet} className="action-button bg-blue-600 hover:bg-blue-700 mt-2">
                  Get Tokens from Faucet
                </button>
              )}
              {profile.betData && (
                <div className="bet-info">
                  <p>Min Bet: {(Number(profile.betData.minBet) / 1e9).toFixed(2)} LIN</p>
                  <p>Max Bet: {(Number(profile.betData.maxBet) / 1e9).toFixed(2)} LIN</p>
                </div>
              )}
            </>
          ) : (
            <p>No profile data</p>
          )}
        </div>

        {gameMode === 'single' && !game && (
          <div className="start-game-section">
            <h3>Start New Game</h3>
            <input
              type="text"
              placeholder="Enter your name"
              value={playerName}
              onChange={(e) => setPlayerName(e.target.value)}
              className="input-field"
            />
            <button onClick={handleStartGame} className="action-button primary">
              Start Game
            </button>
          </div>
        )}
      </div>

      <div className="game-main">
        {gameMode === 'single' && (
          <>
            {game ? (
              <>
                <div className="game-status">
                  <h2>Status: {game.status}</h2>
                  <p>Round: {game.currentRound}</p>
                  <p>Pot: {(Number(game.pot) / 1e9).toFixed(2)} LIN</p>
                </div>

                <div className="community-cards">
                  <h3>Community Cards</h3>
                  <div className="cards-display">
                    {game.communityCards && game.communityCards.length > 0 ? (
                      game.communityCards.map((card, idx) => (
                        <div key={idx} className="card">
                          {card}
                        </div>
                      ))
                    ) : (
                      <p>No community cards yet</p>
                    )}
                  </div>
                </div>

                <div className="players-section">
                  <h3>Players</h3>
                  {game.players && game.players.length > 0 ? (
                    <div className="players-list">
                      {game.players.map((player, idx) => (
                        <div key={idx} className="player-card">
                          <h4>{player.name || `Player ${idx + 1}`}</h4>
                          <p>Chips: {(Number(player.chips) / 1e9).toFixed(2)} LIN</p>
                          <p>Current Bet: {(Number(player.currentBet) / 1e9).toFixed(2)} LIN</p>
                          <p>Status: {player.isFolded ? 'Folded' : player.isActive ? 'Active' : 'Inactive'}</p>
                          {player.holeCards && player.holeCards.length > 0 && (
                            <div className="hole-cards">
                              {player.holeCards.map((card, cIdx) => (
                                <span key={cIdx} className="card-small">{card}</span>
                              ))}
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p>No players in game</p>
                  )}
                </div>

                <div className="game-actions">
                  <h3>Actions</h3>
                  <div className="actions-grid">
                    <input
                      type="number"
                      placeholder="Bet amount"
                      value={betAmount}
                      onChange={(e) => setBetAmount(e.target.value)}
                      className="input-field"
                    />
                    <button
                      onClick={() => {
                        if (betAmount) {
                          actions.bet(betAmount);
                          setBetAmount('');
                        }
                      }}
                      className="action-button"
                    >
                      Bet
                    </button>
                    <button onClick={() => actions.call()} className="action-button">
                      Call
                    </button>
                    <button onClick={() => actions.raise(betAmount || '0')} className="action-button">
                      Raise
                    </button>
                    <button onClick={() => actions.fold()} className="action-button danger">
                      Fold
                    </button>
                  </div>
                </div>
              </>
            ) : (
              <div className="no-game">
                <p>Start a new game to begin playing!</p>
              </div>
            )}
          </>
        )}

        {gameMode === 'multi' && (
          <>
            <div className="mb-4">
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-lg font-semibold">Open Lobbies</h3>
                <button
                  onClick={() => {
                    if (playerName.trim()) {
                      actions.sitAtTable(playerName.trim());
                    }
                  }}
                  className="px-3 py-1 text-xs rounded bg-indigo-600 hover:bg-indigo-700"
                >
                  Sit at Table
                </button>
              </div>
              <h3 className="text-lg font-semibold mb-2">Open Lobbies</h3>
              {lobbies.length === 0 ? (
                <p className="text-slate-400 text-sm">No open lobbies yet. Create one from another chain or play-chain.</p>
              ) : (
                <div className="space-y-2 max-h-48 overflow-y-auto">
                  {lobbies.map((lobby: any) => (
                    <div key={lobby.id} className="bg-slate-800/60 rounded p-2 flex items-center justify-between text-xs">
                      <div>
                        <div className="font-semibold">{lobby.id}</div>
                        <div className="text-slate-400">
                          {lobby.players.length}/{lobby.maxPlayers} players
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {multiGame ? (
              <>
                <div className="game-status">
                  <h2>Multiplayer Status: {multiGame.status}</h2>
                  <p>Round: {multiGame.currentRound}</p>
                  <p>Pot: {Number(multiGame.pot).toString()} units</p>
                  <p>Current Player: {multiGame.currentPlayer ?? 'N/A'}</p>
                </div>

                <div className="community-cards">
                  <h3>Community Cards</h3>
                  <div className="cards-display">
                    {multiGame.communityCards && multiGame.communityCards.length > 0 ? (
                      multiGame.communityCards.map((card: number, idx: number) => (
                        <div key={idx} className="card">
                          {card}
                        </div>
                      ))
                    ) : (
                      <p>No community cards yet</p>
                    )}
                  </div>
                </div>

                <div className="players-section">
                  <h3>Players</h3>
                  {multiGame.players && multiGame.players.length > 0 ? (
                    <div className="players-list">
                      {multiGame.players.map((player: any, idx: number) => (
                        <div key={idx} className="player-card">
                          <h4>{player.name || `Player ${idx + 1}`}</h4>
                          <p>Chips: {Number(player.chips).toString()}</p>
                          <p>Current Bet: {Number(player.currentBet).toString()}</p>
                          <p>Status: {player.isFolded ? 'Folded' : player.isAllIn ? 'All-in' : player.isActive ? 'Active' : 'Inactive'}</p>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p>No players seated</p>
                  )}
                </div>

                <div className="game-actions">
                  <h3>Your Actions</h3>
                  <div className="actions-grid">
                    <input
                      type="number"
                      placeholder="Your seat id"
                      value={multiSeatId}
                      onChange={(e) => setMultiSeatId(e.target.value)}
                      className="input-field"
                    />
                    <input
                      type="number"
                      placeholder="Bet / Raise amount"
                      value={multiBetAmount}
                      onChange={(e) => setMultiBetAmount(e.target.value)}
                      className="input-field"
                    />
                    <button
                      className="action-button"
                      onClick={() => {
                        if (!multiSeatId || !multiBetAmount) return;
                        actions.playerAction({
                          seatId: Number(multiSeatId),
                          action: 'Bet',
                          amount: multiBetAmount,
                        });
                        setMultiBetAmount('');
                      }}
                    >
                      Bet / Raise
                    </button>
                    <button
                      className="action-button"
                      onClick={() => {
                        if (!multiSeatId) return;
                        actions.playerAction({
                          seatId: Number(multiSeatId),
                          action: 'Call',
                        });
                      }}
                    >
                      Call
                    </button>
                    <button
                      className="action-button danger"
                      onClick={() => {
                        if (!multiSeatId) return;
                        actions.playerAction({
                          seatId: Number(multiSeatId),
                          action: 'Fold',
                        });
                      }}
                    >
                      Fold
                    </button>
                  </div>
                </div>
              </>
            ) : (
              <div className="no-game">
                <p>Join a lobby from another chain and the table state will appear here.</p>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}


