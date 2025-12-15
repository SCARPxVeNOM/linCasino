import React, { useState, useEffect } from 'react';
import { useRummy } from '../../lib/games/rummy';
import { ArrowLeft, Users, User } from 'lucide-react';
import './Game.css';

type GameMode = 'single' | 'multi';

export default function RummyGame({ onExit }: { onExit: () => void }) {
  const { game, profile, loading, actions } = useRummy();
  const [gameMode, setGameMode] = useState<GameMode | null>(() => {
    const saved = localStorage.getItem('rummy_game_mode');
    return (saved as GameMode) || null;
  });
  const [playerName, setPlayerName] = useState('');
  const [selectedCard, setSelectedCard] = useState<number | null>(null);

  // Save game mode to localStorage
  useEffect(() => {
    if (gameMode) {
      localStorage.setItem('rummy_game_mode', gameMode);
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
      <div className="flex flex-col h-screen bg-[#0f3628] font-sans overflow-hidden text-white items-center justify-center">
        <div className="bg-[#051f15] border border-indigo-500/50 rounded-2xl max-w-md w-full p-8 shadow-2xl">
          <h2 className="text-2xl font-bold text-indigo-400 text-center mb-6">Select Game Mode</h2>
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
            </>
          ) : (
            <p>No profile data</p>
          )}
        </div>

        {!game && (
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
        {game ? (
          <>
            <div className="game-status">
              <h2>Status: {game.status}</h2>
            </div>

            <div className="players-section">
              <h3>Your Hand</h3>
              {game.players && game.players.length > 0 ? (
                <div className="hand-display">
                  {game.players[0]?.hand?.map((card, idx) => (
                    <div
                      key={idx}
                      className={`card ${selectedCard === idx ? 'selected' : ''}`}
                      onClick={() => setSelectedCard(selectedCard === idx ? null : idx)}
                    >
                      {card}
                    </div>
                  ))}
                </div>
              ) : (
                <p>No cards in hand</p>
              )}
            </div>

            {game.players && game.players[0]?.melds && game.players[0].melds.length > 0 && (
              <div className="melds-section">
                <h3>Your Melds</h3>
                <div className="melds-list">
                  {game.players[0].melds.map((meld, idx) => (
                    <div key={idx} className="meld-card">
                      <p>Type: {meld.meldType}</p>
                      <div className="cards-display">
                        {meld.cards.map((card, cIdx) => (
                          <span key={cIdx} className="card-small">{card}</span>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="game-actions">
              <h3>Actions</h3>
              <div className="actions-grid">
                <button onClick={() => actions.drawFromDeck()} className="action-button">
                  Draw from Deck
                </button>
                <button onClick={() => actions.drawFromDiscard()} className="action-button">
                  Draw from Discard
                </button>
                <button
                  onClick={() => {
                    if (selectedCard !== null) {
                      actions.discardCard(selectedCard);
                      setSelectedCard(null);
                    }
                  }}
                  className="action-button"
                  disabled={selectedCard === null}
                >
                  Discard Card
                </button>
                <button onClick={() => actions.declare()} className="action-button primary">
                  Declare
                </button>
              </div>
            </div>

            {game.discardPile && game.discardPile.length > 0 && (
              <div className="discard-pile">
                <h3>Discard Pile (Top Card)</h3>
                <div className="card">{game.discardPile[game.discardPile.length - 1]}</div>
              </div>
            )}
          </>
        ) : (
          <div className="no-game">
            <p>Start a new game to begin playing!</p>
          </div>
        )}
      </div>
    </div>
  );
}


