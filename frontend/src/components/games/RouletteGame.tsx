import React, { useState, useEffect, useCallback } from 'react';
import { ArrowLeft, DollarSign, Crown, Users, User, AlertCircle } from 'lucide-react';
import { useRoulette } from '../../lib/games/roulette';

// Chip component
const Chip = ({ value, className = "" }: { value: number, className?: string }) => {
  let color = "bg-slate-200 border-slate-400 text-slate-800";
  if (value >= 100) color = "bg-red-600 border-red-800 text-white";
  else if (value >= 50) color = "bg-blue-600 border-blue-800 text-white";
  else if (value >= 25) color = "bg-green-600 border-green-800 text-white";
  else if (value >= 10) color = "bg-orange-600 border-orange-800 text-white";

  return (
    <div className={`w-8 h-8 sm:w-10 sm:h-10 rounded-full border-4 border-dashed shadow-[0_4px_6px_rgba(0,0,0,0.5)] flex items-center justify-center text-[10px] sm:text-xs font-bold ${color} ${className}`}>
      {value}
    </div>
  );
};

// --- Roulette Constants ---
const ROULETTE_NUMBERS = [
  0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8, 23, 10, 5, 24, 16, 33, 1, 20, 14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26
];
const RED_NUMBERS = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];

type GameMode = 'single' | 'multi';

export default function RouletteGame({ onExit }: { onExit: () => void }) {
  // Fully integrate with Linera - no try-catch, make it required
  const { game: lineraGame, profile, loading: lineraLoading, error: lineraError, actions, refetch } = useRoulette();
  
  // Ensure lineraGame is never undefined - default to null
  const safeLineraGame = lineraGame || null;
  
  const [gameMode, setGameMode] = useState<GameMode | null>(() => {
    try {
      const saved = localStorage.getItem('roulette_game_mode');
      return (saved as GameMode) || null;
    } catch {
      return null;
    }
  });
  const [playerName, setPlayerName] = useState('');
  const [spinning, setSpinning] = useState(false);
  const [rotation, setRotation] = useState(0);
  const [ballRotation, setBallRotation] = useState(0);
  const [lastNumber, setLastNumber] = useState<number | null>(null);
  const [bets, setBets] = useState<Record<string, number>>({});
  const [message, setMessage] = useState("Place your bets!");
  const [history, setHistory] = useState<number[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isPlacingBet, setIsPlacingBet] = useState(false);
  
  // Use chips from Linera profile - sync with backend
  const chips = profile?.balance ? Number(profile.balance) / 1e9 : 0;

  // Save game mode to localStorage
  useEffect(() => {
    if (gameMode) {
      localStorage.setItem('roulette_game_mode', gameMode);
    }
  }, [gameMode]);

  // Auto-refetch game state periodically when game is active
  useEffect(() => {
    if (safeLineraGame && !spinning) {
      const interval = setInterval(() => {
        refetch();
      }, 2000); // Refetch every 2 seconds
      return () => clearInterval(interval);
    }
  }, [safeLineraGame, spinning, refetch]);

  // Sync with Linera game state - this is the source of truth
  useEffect(() => {
    if (!safeLineraGame || typeof safeLineraGame !== 'object' || safeLineraGame === null) {
      return;
    }

    try {
      // Sync currentNumber from Linera (this is the actual result from backend)
      // Note: async-graphql converts snake_case to camelCase, so we use currentNumber
      if ('currentNumber' in safeLineraGame) {
        const currentNum = safeLineraGame.currentNumber;
        if (currentNum !== null && currentNum !== undefined) {
          const newNumber = Number(currentNum);
          if (!isNaN(newNumber) && newNumber !== lastNumber) {
            setLastNumber(newNumber);
            // If we just got a result and we were spinning, resolve bets
            if (spinning) {
              resolveBets(newNumber);
              setSpinning(false);
            }
          }
        }
      }
      
      // Sync history from Linera
      if ('history' in safeLineraGame && Array.isArray(safeLineraGame.history) && safeLineraGame.history.length > 0) {
        const lineraHistory = safeLineraGame.history.map((h: any) => Number(h)).filter((h: number) => !isNaN(h));
        if (lineraHistory.length > 0) {
          setHistory(lineraHistory.slice(-10));
        }
      }
      
      // Sync bets from Linera - this is the source of truth
      if ('bets' in safeLineraGame && Array.isArray(safeLineraGame.bets)) {
        const localBets: Record<string, number> = {};
        safeLineraGame.bets.forEach((bet: any) => {
          if (!bet || typeof bet !== 'object' || !('betType' in bet)) return;
          // Parse betType string (e.g., "{\"Number\":0}" or "Red")
          // Note: async-graphql converts snake_case to camelCase, so we use betType
          let betKey = String(bet.betType || '');
          try {
            const parsed = JSON.parse(bet.betType);
            if (parsed && typeof parsed === 'object') {
              if (parsed.Number !== undefined) {
                betKey = `num-${parsed.Number}`;
              } else if (parsed.Red) {
                betKey = 'red';
              } else if (parsed.Black) {
                betKey = 'black';
              } else if (parsed.Even) {
                betKey = 'even';
              } else if (parsed.Odd) {
                betKey = 'odd';
              }
            }
          } catch {
            // If not JSON, use as-is
          }
          const amount = Number(bet.amount || 0);
          if (!isNaN(amount)) {
            localBets[betKey] = (localBets[betKey] || 0) + amount / 1e9;
          }
        });
        setBets(localBets);
      }
    } catch (error) {
      console.error('Error syncing Linera game state:', error);
    }
  }, [safeLineraGame, spinning, lastNumber]);

  // Get balance on mount
  useEffect(() => {
    if (!lineraLoading && actions.getBalance) {
      actions.getBalance();
    }
  }, [lineraLoading, actions]);

  const resolveBets = useCallback((num: number) => {
    const isRed = RED_NUMBERS.includes(num);
    const isBlack = !isRed && num !== 0;
    const isEven = num !== 0 && num % 2 === 0;
    const isOdd = num !== 0 && num % 2 !== 0;

    let totalWinnings = 0;
    Object.entries(bets).forEach(([key, amount]) => {
      let win = false;
      let multiplier = 0;

      if (key === 'red' && isRed) { win = true; multiplier = 2; }
      if (key === 'black' && isBlack) { win = true; multiplier = 2; }
      if (key === 'even' && isEven) { win = true; multiplier = 2; }
      if (key === 'odd' && isOdd) { win = true; multiplier = 2; }
      if (key === `num-${num}`) { win = true; multiplier = 36; }

      if (win) totalWinnings += amount * multiplier;
    });

    if (totalWinnings > 0) {
      setMessage(`Result: ${num} ${isRed ? 'RED' : (num === 0 ? 'GREEN' : 'BLACK')}. You won $${totalWinnings.toFixed(2)}!`);
    } else {
      setMessage(`Result: ${num}. Better luck next time.`);
    }
    
    // Refetch to get updated balance from Linera
    setTimeout(() => {
      refetch();
    }, 500);
  }, [bets, refetch]);

  const placeBet = async (type: string, amount: number) => {
    if (spinning || isPlacingBet || chips < amount) {
      if (chips < amount) {
        setMessage("Insufficient balance! Get more tokens from faucet.");
        setError("Insufficient balance");
        setTimeout(() => setError(null), 3000);
      }
      return;
    }
    
    setIsPlacingBet(true);
    setError(null);
    
    // Convert bet type to Linera format
    let betTypeStr = '';
    if (type.startsWith('num-')) {
      const num = parseInt(type.replace('num-', ''));
      betTypeStr = JSON.stringify({ Number: num });
    } else if (type === 'red') {
      betTypeStr = JSON.stringify({ Red: true });
    } else if (type === 'black') {
      betTypeStr = JSON.stringify({ Black: true });
    } else if (type === 'even') {
      betTypeStr = JSON.stringify({ Even: true });
    } else if (type === 'odd') {
      betTypeStr = JSON.stringify({ Odd: true });
    } else {
      betTypeStr = type;
    }

    // Convert amount to Linera format (multiply by 1e9)
    const amountStr = (amount * 1e9).toString();
    
    try {
      // Place bet on Linera backend
      await actions.placeBet(betTypeStr, amountStr);
      
      // Update local state optimistically
      setBets(prev => ({ ...prev, [type]: (prev[type] || 0) + amount }));
      setMessage(`Bet placed: ${type} - $${amount}`);
      
      // Refetch to sync with backend
      await refetch();
    } catch (error: any) {
      console.error('Error placing bet in Linera:', error);
      setError(`Failed to place bet: ${error.message || 'Unknown error'}`);
      setTimeout(() => setError(null), 3000);
    } finally {
      setIsPlacingBet(false);
    }
  };

  const spinWheel = async () => {
    if (spinning || Object.keys(bets).length === 0) {
      if (Object.keys(bets).length === 0) {
        setMessage("Place a bet first!");
        setError("No bets placed");
        setTimeout(() => setError(null), 2000);
      }
      return;
    }

    setSpinning(true);
    setMessage("No more bets!");
    setError(null);

    // Animate wheel while waiting for Linera result
    const randomIndex = Math.floor(Math.random() * ROULETTE_NUMBERS.length);
    const resultNumber = ROULETTE_NUMBERS[randomIndex];
    const sliceDeg = 360 / 37;
    const targetIndex = randomIndex;
    const extraSpins = 5 + Math.floor(Math.random() * 3);
    const targetRotation = (extraSpins * 360) - (targetIndex * sliceDeg);
    const ballTarget = (extraSpins * 360) + (Math.random() * 360);

    setRotation(targetRotation);
    setBallRotation(ballTarget);

    try {
      // Call Linera spin - this will generate the actual result
      await actions.spin();
      
      // Wait for animation to complete
      setTimeout(async () => {
        // Refetch to get the actual result from Linera
        await refetch();
        // The result will be synced via the useEffect that watches safeLineraGame
      }, 4000);
    } catch (error: any) {
      console.error('Error spinning in Linera:', error);
      setError(`Failed to spin: ${error.message || 'Unknown error'}`);
      setSpinning(false);
      setTimeout(() => {
        setError(null);
        setMessage("Place your bets!");
      }, 3000);
    }
  };

  const handleGetBalance = async () => {
    try {
      setMessage("Refreshing balance...");
      await actions.getBalance();
      await refetch();
      setMessage("Balance refreshed!");
      setTimeout(() => setMessage("Place your bets!"), 2000);
    } catch (error: any) {
      console.error('Error getting balance:', error);
      setError(`Failed to refresh balance: ${error.message || 'Unknown error'}`);
      setTimeout(() => setError(null), 3000);
    }
  };

  const handleRequestFaucet = async () => {
    try {
      setMessage("Requesting tokens from faucet...");
      // Try /faucet endpoint first
      let response = await fetch('http://localhost:8080/faucet', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({})
      });

      if (!response.ok) {
        // If /faucet fails, try root endpoint
        response = await fetch('http://localhost:8080', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({})
        });
      }

      if (response.ok) {
        setMessage("Faucet tokens requested! Refreshing balance...");
        setTimeout(async () => {
          await actions.getBalance();
          await refetch();
          setMessage("Balance updated!");
          setTimeout(() => setMessage("Place your bets!"), 2000);
        }, 2000);
      } else {
        setError("Faucet request failed. Check if faucet is running on port 8080.");
        setTimeout(() => setError(null), 5000);
      }
    } catch (error: any) {
      console.error('Faucet error:', error);
      setError(`Faucet connection error: ${error.message}. Is faucet running?`);
      setTimeout(() => setError(null), 5000);
    }
  };

  const handleStartGame = async () => {
    if (!playerName.trim()) {
      setError("Please enter your name");
      setTimeout(() => setError(null), 2000);
      return;
    }

    try {
      setMessage("Starting game...");
      await actions.startGame(playerName.trim());
      await refetch();
      setMessage("Game started! Place your bets!");
    } catch (error: any) {
      console.error('Error starting game:', error);
      setError(`Failed to start game: ${error.message || 'Unknown error'}`);
      setTimeout(() => setError(null), 3000);
    }
  };

  // Show error if any
  const displayError = error || (lineraError ? String(lineraError) : null);

  // Game Mode Selection Screen
  if (!gameMode) {
    return (
      <div className="flex flex-col h-screen bg-[#0a0c10] font-sans overflow-hidden text-slate-200 items-center justify-center">
        <div className="bg-[#1a1c29] border border-yellow-500/50 rounded-2xl max-w-md w-full p-8 shadow-2xl">
          <h2 className="text-2xl font-bold text-yellow-400 text-center mb-6">Select Game Mode</h2>
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

  // Start Game Screen
  if (!safeLineraGame && !lineraLoading) {
    return (
      <div className="flex flex-col h-screen bg-[#0a0c10] font-sans overflow-hidden text-slate-200">
        <div className="h-16 bg-[#050608] flex items-center justify-between px-6 shadow-2xl border-b border-[#2a2d3e] z-20 relative">
          <button onClick={onExit} className="flex items-center text-slate-400 hover:text-white transition-colors relative z-10 group">
            <ArrowLeft size={20} className="mr-2 group-hover:-translate-x-1 transition-transform" /> Lobby
          </button>
          <div className="text-2xl font-black text-transparent bg-clip-text bg-gradient-to-r from-amber-200 via-yellow-500 to-amber-700 tracking-widest uppercase">
            Roulette Royale
          </div>
          <div className="w-32"></div>
        </div>
        <div className="flex-1 flex items-center justify-center">
          <div className="bg-[#1a1c29] border border-yellow-500/50 rounded-2xl max-w-md w-full p-8 shadow-2xl">
            <h2 className="text-2xl font-bold text-yellow-400 text-center mb-6">Start New Game</h2>
            {displayError && (
              <div className="mb-4 p-3 bg-red-900/50 border border-red-500/50 rounded-lg flex items-center gap-2 text-red-300 text-sm">
                <AlertCircle size={16} />
                {displayError}
              </div>
            )}
            <div className="mb-4">
              <label className="block text-sm font-medium text-slate-300 mb-2">Game Mode</label>
              <div className="flex gap-2">
                <button
                  onClick={() => setGameMode('single')}
                  className={`flex-1 p-3 rounded-lg border transition-all ${
                    gameMode === 'single' 
                      ? 'bg-emerald-900/50 border-emerald-500 text-white' 
                      : 'bg-slate-800/50 border-slate-600 text-slate-400'
                  }`}
                >
                  <User size={20} className="mx-auto mb-1" />
                  <div className="text-xs">Single</div>
                </button>
                <button
                  onClick={() => setGameMode('multi')}
                  className={`flex-1 p-3 rounded-lg border transition-all ${
                    gameMode === 'multi' 
                      ? 'bg-indigo-900/50 border-indigo-500 text-white' 
                      : 'bg-slate-800/50 border-slate-600 text-slate-400'
                  }`}
                >
                  <Users size={20} className="mx-auto mb-1" />
                  <div className="text-xs">Multiplayer</div>
                </button>
              </div>
            </div>
            <div className="mb-4">
              <label className="block text-sm font-medium text-slate-300 mb-2">Player Name</label>
              <input
                type="text"
                value={playerName}
                onChange={(e) => setPlayerName(e.target.value)}
                placeholder="Enter your name"
                className="w-full px-4 py-2 bg-slate-800 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-yellow-500"
                onKeyPress={(e) => {
                  if (e.key === 'Enter') {
                    handleStartGame();
                  }
                }}
              />
            </div>
            <div className="mb-6">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-slate-300">Balance</span>
                <button
                  onClick={handleGetBalance}
                  className="text-xs text-yellow-400 hover:text-yellow-300"
                >
                  Refresh
                </button>
              </div>
              <div className="flex items-center gap-2 px-4 py-2 bg-slate-800/50 rounded-lg">
                <DollarSign size={16} className="text-yellow-400" />
                <span className="font-mono font-bold text-yellow-400">{chips.toFixed(2)} LIN</span>
              </div>
              {chips < 10 && (
                <button
                  onClick={handleRequestFaucet}
                  className="mt-2 w-full py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-bold"
                >
                  Get Tokens from Faucet
                </button>
              )}
            </div>
            <button
              onClick={handleStartGame}
              disabled={!playerName.trim()}
              className="w-full py-3 bg-gradient-to-r from-yellow-600 via-yellow-400 to-yellow-600 text-black font-bold rounded-lg hover:scale-105 transition-transform disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Start Game
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Show loading only if Linera is loading
  if (lineraLoading && !safeLineraGame) {
    return (
      <div className="flex flex-col items-center justify-center h-screen text-white bg-[#0a0c10]">
        <div className="text-xl mb-4">Loading Roulette game...</div>
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-yellow-400"></div>
      </div>
    );
  }

  // Main game render - fully integrated with Linera
  return (
    <div className="flex flex-col h-screen bg-[#0a0c10] font-sans overflow-hidden text-slate-200">
      {/* Error Banner */}
      {displayError && (
        <div className="absolute top-20 left-1/2 -translate-x-1/2 z-50 bg-red-900/90 border border-red-500 text-white p-3 rounded-lg shadow-xl max-w-md flex items-center gap-2">
          <AlertCircle size={20} />
          <span>{displayError}</span>
        </div>
      )}

      {/* Header */}
      <div className="h-16 bg-[#050608] flex items-center justify-between px-6 shadow-2xl border-b border-[#2a2d3e] z-20 relative">
        <div className="absolute inset-0 bg-gradient-to-r from-red-900/20 via-black to-red-900/20 pointer-events-none"></div>
        <button onClick={onExit} className="flex items-center text-slate-400 hover:text-white transition-colors relative z-10 group">
          <ArrowLeft size={20} className="mr-2 group-hover:-translate-x-1 transition-transform" /> Lobby
        </button>
        <div className="flex flex-col items-center relative z-10">
          <div className="text-2xl font-black text-transparent bg-clip-text bg-gradient-to-r from-amber-200 via-yellow-500 to-amber-700 tracking-widest uppercase drop-shadow-[0_2px_4px_rgba(0,0,0,0.8)]">
            Roulette Royale
          </div>
          <div className="text-[10px] text-yellow-600 font-serif tracking-[0.3em] uppercase">
            {gameMode === 'single' ? 'Single Player' : 'Multiplayer'} • VIP Table • Linera Powered
          </div>
        </div>
        <div className="flex items-center gap-2 relative z-10">
          <button
            onClick={handleGetBalance}
            disabled={isPlacingBet}
            className="px-3 py-1.5 bg-slate-800/50 hover:bg-slate-700 border border-slate-600 rounded-lg text-xs text-slate-300 transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
          <div className="flex items-center bg-[#1a1c29] px-4 py-1.5 rounded-full border border-[#3f435e] shadow-inner">
            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse mr-2"></div>
            <DollarSign size={16} className="text-yellow-400 mr-1" />
            <span className="font-mono font-bold text-yellow-400 tracking-wider">{chips.toFixed(2)}</span>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-hidden flex flex-col lg:flex-row relative">
        {/* Background Ambience */}
        <div className="absolute inset-0 pointer-events-none z-0">
           <div className="absolute top-0 left-1/3 w-full h-full bg-[radial-gradient(circle_at_50%_0%,_#3b0a0a_0%,_transparent_70%)] opacity-40"></div>
        </div>

        {/* Wheel Section */}
        <div className="flex-1 relative flex flex-col items-center justify-center p-8 lg:border-r border-white/5 shadow-[20px_0_50px_rgba(0,0,0,0.5)] z-10">
           {/* Wheel Halo */}
           <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[400px] bg-yellow-500/5 rounded-full blur-[80px] pointer-events-none"></div>

           {/* The Wheel Assembly */}
           <div className="relative w-[300px] h-[300px] sm:w-[420px] sm:h-[420px] rounded-full bg-[#1a0f0a] p-4 shadow-[0_20px_50px_rgba(0,0,0,0.8),inset_0_5px_20px_rgba(255,255,255,0.05)] border-[8px] border-[#3d2314] ring-1 ring-white/10">
             
             {/* Wood Texture Ring */}
             <div className="absolute inset-0 rounded-full border-[24px] border-[#2a150d] opacity-80 pointer-events-none"></div>
             
             {/* Gold Inner Ring */}
             <div className="absolute inset-6 rounded-full border-[2px] border-yellow-600/50 shadow-[inset_0_0_20px_rgba(0,0,0,0.5)] pointer-events-none"></div>

             {/* Spinning Part */}
             <div className="w-full h-full rounded-full relative overflow-hidden">
                <div 
                    className="w-full h-full rounded-full relative transition-transform duration-[4000ms] cubic-bezier(0.15, 0, 0.15, 1)"
                    style={{ transform: `rotate(${rotation}deg)` }}
                >
                  {/* Wheel Background (Pockets) */}
                  <div className="absolute inset-2 rounded-full border-[40px] border-black opacity-40"></div>
                  
                  {/* Slices */}
                  {ROULETTE_NUMBERS.map((num, i) => {
                    const angle = (360 / 37) * i;
                    const isRed = RED_NUMBERS.includes(num);
                    const isGreen = num === 0;
                    
                    return (
                      <div 
                        key={i}
                        className="absolute top-0 left-1/2 w-0.5 h-[50%] origin-bottom"
                        style={{ transform: `translateX(-50%) rotate(${angle}deg)` }}
                      >
                        {/* The Wedge Visual */}
                        <div 
                          className={`absolute top-0 -left-[16px] w-[32px] h-[65px] flex flex-col items-center pt-2
                          ${isGreen ? 'bg-gradient-to-b from-green-600 to-green-800' : 
                            isRed ? 'bg-gradient-to-b from-red-600 to-red-900' : 
                            'bg-gradient-to-b from-slate-800 to-slate-950'}
                          `}
                          style={{ 
                            clipPath: 'polygon(0% 0%, 100% 0%, 50% 100%)',
                          }}
                        >
                           <span className="text-[10px] sm:text-xs font-bold text-white transform rotate-180 drop-shadow-md">{num}</span>
                        </div>
                        {/* Metal Separator */}
                        <div className="absolute top-0 left-0 w-[1px] h-[65px] bg-yellow-500/30"></div>
                      </div>
                    );
                  })}

                  {/* Center Hub */}
                  <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-32 h-32 rounded-full bg-[conic-gradient(from_0deg,#b8860b,#f59e0b,#b8860b)] shadow-[0_0_30px_rgba(0,0,0,0.8)] flex items-center justify-center z-10 border-4 border-[#5e3822]">
                      <div className="w-24 h-24 rounded-full bg-[#1a0f0a] flex items-center justify-center relative">
                         {/* Spinner Handle Effect */}
                         <div className="absolute w-full h-2 bg-yellow-600/20 rotate-45"></div>
                         <div className="absolute w-full h-2 bg-yellow-600/20 -rotate-45"></div>
                         <div className="w-16 h-16 rounded-full bg-gradient-to-br from-[#ffd700] to-[#b8860b] shadow-inner flex items-center justify-center">
                            <Crown size={24} className="text-[#3d2314]" />
                         </div>
                      </div>
                  </div>
                </div>

                {/* The Ball */}
                <div 
                  className={`absolute top-0 left-1/2 w-3.5 h-3.5 bg-white rounded-full shadow-[0_0_8px_rgba(255,255,255,0.8),inset_-2px_-2px_4px_rgba(0,0,0,0.3)] z-20 ${spinning ? 'opacity-100' : 'opacity-0'}`}
                  style={{ 
                    transformOrigin: '50% 190px',
                    transform: `translateX(-50%) rotate(${-ballRotation}deg)`,
                    transition: 'transform 4000ms cubic-bezier(0.15, 0, 0.15, 1)'
                  }}
                />
             </div>
             
             {/* Fixed Indicator */}
             <div className="absolute -top-3 left-1/2 -translate-x-1/2 w-0 h-0 border-l-[10px] border-l-transparent border-r-[10px] border-r-transparent border-t-[20px] border-t-yellow-400 drop-shadow-[0_4px_4px_rgba(0,0,0,0.5)] z-30" />
             <div className="absolute -top-3 left-1/2 -translate-x-1/2 w-1 h-2 bg-yellow-200 blur-[2px] z-30"></div>
           </div>

           {/* Game Info Panel */}
           <div className="mt-10 w-full max-w-md">
             <div className="bg-black/40 backdrop-blur-xl border border-white/5 rounded-2xl p-4 flex flex-col items-center">
               <div className="text-yellow-400/80 text-xs font-bold uppercase tracking-widest mb-2">{message}</div>
               
               {lastNumber !== null && !spinning && (
                 <div className="flex items-center gap-4 animate-in zoom-in duration-300">
                    <div className={`w-12 h-12 rounded-lg flex items-center justify-center text-xl font-bold text-white shadow-lg border border-white/10
                      ${lastNumber === 0 ? 'bg-green-600 shadow-green-500/20' : 
                        RED_NUMBERS.includes(lastNumber) ? 'bg-red-600 shadow-red-500/20' : 
                        'bg-slate-800 shadow-slate-500/20'}
                    `}>
                      {lastNumber}
                    </div>
                    <div className="flex flex-col">
                      <span className="text-white font-bold text-lg">WINNER</span>
                      <span className="text-xs text-slate-400">
                        {lastNumber === 0 ? 'Zero' : (RED_NUMBERS.includes(lastNumber) ? 'Red' : 'Black')} • {lastNumber % 2 === 0 && lastNumber !== 0 ? 'Even' : (lastNumber !== 0 ? 'Odd' : '')}
                      </span>
                    </div>
                 </div>
               )}
             </div>

             {/* Recent History */}
             <div className="flex justify-center gap-2 mt-4 opacity-70">
                {history.map((h, i) => (
                  <div key={i} className={`w-8 h-8 rounded flex items-center justify-center text-xs font-bold border border-white/5 shadow-lg
                    ${h === 0 ? 'bg-green-900 text-green-100' : RED_NUMBERS.includes(h) ? 'bg-red-900 text-red-100' : 'bg-slate-900 text-slate-200'}
                  `}>
                    {h}
                  </div>
                ))}
             </div>
           </div>
        </div>

        {/* Betting Board Section */}
        <div className="flex-1 bg-[#1e3a2a] relative overflow-hidden flex flex-col">
          {/* Felt Texture */}
          <div className="absolute inset-0 bg-[url('https://www.transparenttextures.com/patterns/green-felt.png')] opacity-60"></div>
          <div className="absolute inset-0 bg-gradient-to-br from-black/40 via-transparent to-black/20 pointer-events-none"></div>

          <div className="relative z-10 flex-1 p-4 lg:p-8 overflow-y-auto flex items-center justify-center">
             <div className="w-full max-w-2xl bg-emerald-900/80 p-6 rounded-xl border-[6px] border-[#daa520] shadow-[0_20px_60px_rgba(0,0,0,0.6),inset_0_0_40px_rgba(0,0,0,0.4)] backdrop-blur-sm">
                
                {/* Board Label */}
                <div className="text-center mb-6">
                   <span className="text-[#daa520] font-serif text-2xl tracking-[0.2em] font-bold opacity-40">PLACE YOUR BETS</span>
                </div>

                <div className="grid grid-cols-[auto_repeat(3,1fr)] gap-1.5 select-none">
                  
                  {/* 0 */}
                  <div 
                    onClick={() => placeBet('num-0', 10)}
                    className="row-span-[13] w-14 bg-green-700/90 hover:bg-green-600 rounded-l-md border border-white/10 flex items-center justify-center cursor-pointer relative group transition-colors shadow-inner disabled:opacity-50 disabled:cursor-not-allowed"
                    style={{ pointerEvents: spinning || isPlacingBet ? 'none' : 'auto' }}
                  >
                    <span className="text-white font-bold text-xl -rotate-90">0</span>
                    {bets['num-0'] && <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-20"><Chip value={bets['num-0']} className="scale-75 shadow-xl" /></div>}
                    <div className="absolute inset-0 border-2 border-transparent group-hover:border-white/20 rounded-l-md transition-colors"></div>
                  </div>

                  {/* 1-36 Grid */}
                  <div className="col-span-3 grid grid-cols-3 gap-1.5">
                    {Array.from({length: 36}, (_, i) => i + 1).map(num => {
                      const isRed = RED_NUMBERS.includes(num);
                      return (
                        <div 
                          key={num}
                          onClick={() => placeBet(`num-${num}`, 10)}
                          className={`
                            h-12 flex items-center justify-center cursor-pointer relative transition-all duration-200 group
                            ${isRed ? 'bg-red-700/90 hover:bg-red-600' : 'bg-slate-800/90 hover:bg-slate-700'}
                            rounded-sm border border-white/5 shadow-sm
                            ${spinning || isPlacingBet ? 'opacity-50 cursor-not-allowed' : ''}
                          `}
                          style={{ pointerEvents: spinning || isPlacingBet ? 'none' : 'auto' }}
                        >
                          <span className="font-bold text-white text-lg font-serif">{num}</span>
                          {bets[`num-${num}`] && <div className="absolute z-20"><Chip value={bets[`num-${num}`]} className="scale-75 shadow-xl" /></div>}
                          {/* Hover Glow */}
                          <div className="absolute inset-0 bg-white/0 group-hover:bg-white/10 transition-colors"></div>
                        </div>
                      )
                    })}
                  </div>

                </div>

                {/* Outside Bets */}
                <div className="grid grid-cols-4 gap-2 mt-4">
                   {[
                     { id: 'red', label: 'RED', color: 'bg-red-800' },
                     { id: 'black', label: 'BLACK', color: 'bg-slate-900' },
                     { id: 'even', label: 'EVEN', color: 'bg-emerald-900' },
                     { id: 'odd', label: 'ODD', color: 'bg-emerald-900' }
                   ].map(opt => (
                     <div 
                        key={opt.id}
                        onClick={() => placeBet(opt.id, 10)} 
                        className={`${opt.color} h-14 rounded border border-[#daa520]/30 hover:border-[#daa520] flex items-center justify-center cursor-pointer relative group transition-all ${spinning || isPlacingBet ? 'opacity-50 cursor-not-allowed' : ''}`}
                        style={{ pointerEvents: spinning || isPlacingBet ? 'none' : 'auto' }}
                     >
                       <span className="text-[#daa520] font-bold tracking-wider text-xs sm:text-sm">{opt.label}</span>
                       {bets[opt.id] && <div className="absolute right-1 top-1"><Chip value={bets[opt.id]} className="scale-50 shadow-lg" /></div>}
                     </div>
                   ))}
                </div>

                {/* Controls */}
                <div className="flex justify-between items-center mt-8 pt-6 border-t border-[#daa520]/20">
                   <button 
                      onClick={() => setBets({})} 
                      disabled={spinning || isPlacingBet} 
                      className="px-4 py-2 text-red-400 hover:text-red-300 text-xs font-bold uppercase tracking-widest disabled:opacity-50"
                    >
                      Clear Table
                   </button>
                   
                   <button 
                    onClick={spinWheel}
                    disabled={spinning || isPlacingBet || Object.keys(bets).length === 0}
                    className={`
                      relative px-10 py-4 rounded-full font-black text-xl tracking-widest shadow-[0_0_20px_rgba(234,179,8,0.4)]
                      transform transition-all duration-200 overflow-hidden group
                      ${spinning || isPlacingBet || Object.keys(bets).length === 0 ? 'bg-slate-700 text-slate-400 scale-95 cursor-not-allowed' : 'bg-gradient-to-r from-yellow-600 via-yellow-400 to-yellow-600 text-black hover:scale-105'}
                    `}
                   >
                     <span className="relative z-10">{spinning ? 'NO MORE BETS' : isPlacingBet ? 'PLACING BET...' : 'SPIN'}</span>
                     {!spinning && !isPlacingBet && Object.keys(bets).length > 0 && <div className="absolute inset-0 bg-white/30 skew-x-12 -translate-x-full group-hover:animate-shine"></div>}
                   </button>
                </div>

             </div>
          </div>
        </div>
      </div>
    </div>
  );
}
