import React, { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { ArrowLeft, RefreshCw, DollarSign, AlertCircle, Layers, Trophy, Coins, RotateCcw, Crown, Sparkles, Zap, MessageCircle } from 'lucide-react';
import { usePoker } from '../lib/games/poker';
import { useRummy } from '../lib/games/rummy';
import { useRoulette } from '../lib/games/roulette';
import PokerGame from './games/PokerGame';
import RummyGame from './games/RummyGame';
import RouletteGame from './games/RouletteGame';

// --- Global Types ---
type Suit = 'hearts' | 'diamonds' | 'clubs' | 'spades';
type Rank = '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '10' | 'J' | 'Q' | 'K' | 'A';

interface Card {
  id: string;
  suit: Suit;
  rank: Rank;
  value: number;
  isJoker?: boolean;
}

interface Player {
  id: number;
  name: string;
  cards: Card[];
  chips: number;
  isFolded: boolean;
  isActive: boolean;
  currentBet: number;
  avatar: string;
  personality?: string;
}

// --- Roulette Constants ---
const ROULETTE_NUMBERS = [
  0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8, 23, 10, 5, 24, 16, 33, 1, 20, 14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26
];
const RED_NUMBERS = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];

// --- Helpers ---
const SUITS: Suit[] = ['hearts', 'diamonds', 'clubs', 'spades'];
const RANKS: Rank[] = ['2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K', 'A'];

const createDeck = (): Card[] => {
  const deck: Card[] = [];
  SUITS.forEach(suit => {
    RANKS.forEach((rank, index) => {
      deck.push({ id: `${rank}-${suit}-${Math.random()}`, suit, rank, value: index + 2, isJoker: false });
    });
  });
  return deck;
};

const shuffleDeck = (deck: Card[]): Card[] => {
  const newDeck = [...deck];
  for (let i = newDeck.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [newDeck[i], newDeck[j]] = [newDeck[j], newDeck[i]];
  }
  return newDeck;
};

const getSuitSymbol = (suit: Suit) => {
  switch (suit) {
    case 'hearts': return '♥';
    case 'diamonds': return '♦';
    case 'clubs': return '♣';
    case 'spades': return '♠';
  }
};

const getCardColor = (suit: Suit) => (suit === 'hearts' || suit === 'diamonds') ? 'text-red-600' : 'text-slate-900';

// --- GEMINI API INTEGRATION (Optional) ---
const apiKey = import.meta.env.VITE_GEMINI_API_KEY || "";

const callGemini = async (prompt: string): Promise<string> => {
  if (!apiKey) return "AI features require API key configuration.";
  try {
    const response = await fetch(
      `https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-preview-09-2025:generateContent?key=${apiKey}`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          contents: [{ parts: [{ text: prompt }] }],
        }),
      }
    );

    if (!response.ok) throw new Error('API call failed');
    const data = await response.json();
    return data.candidates?.[0]?.content?.parts?.[0]?.text || "I'm speechless right now.";
  } catch (error) {
    console.error("Gemini API Error:", error);
    return "Thinking...";
  }
};

// --- Poker Evaluator ---
const evaluateHandStrength = (holeCards: Card[], communityCards: Card[]): { score: number, name: string } => {
  const allCards = [...holeCards, ...communityCards].sort((a, b) => b.value - a.value);
  const flushSuit = SUITS.find(s => allCards.filter(c => c.suit === s).length >= 5);
  const ranks = allCards.map(c => c.value);
  
  let straightHigh = 0;
  const uniqueRanks = Array.from(new Set(ranks));
  for (let i = 0; i < uniqueRanks.length - 4; i++) {
    if (uniqueRanks[i] - uniqueRanks[i+4] === 4) { straightHigh = uniqueRanks[i]; break; }
  }

  const counts: Record<number, number> = {};
  ranks.forEach(r => counts[r] = (counts[r] || 0) + 1);
  const fours = Object.keys(counts).filter(r => counts[parseInt(r)] === 4).map(Number);
  const threes = Object.keys(counts).filter(r => counts[parseInt(r)] === 3).map(Number);
  const pairs = Object.keys(counts).filter(r => counts[parseInt(r)] === 2).map(Number);

  if (flushSuit && straightHigh) return { score: 800 + straightHigh, name: 'Straight Flush' };
  if (fours.length > 0) return { score: 700 + fours[0], name: 'Four of a Kind' };
  if (threes.length > 0 && pairs.length > 0) return { score: 600 + threes[0], name: 'Full House' };
  if (flushSuit) return { score: 500, name: 'Flush' };
  if (straightHigh) return { score: 400 + straightHigh, name: 'Straight' };
  if (threes.length > 0) return { score: 300 + threes[0], name: 'Three of a Kind' };
  if (pairs.length >= 2) return { score: 200 + pairs[0], name: 'Two Pair' };
  if (pairs.length === 1) return { score: 100 + pairs[0], name: 'Pair' };
  return { score: ranks[0], name: 'High Card' };
};

// --- Shared Components ---
const CardComponent = ({ card, hidden = false, selected = false, onClick, className = "" }: { card: Card, hidden?: boolean, selected?: boolean, onClick?: () => void, className?: string }) => {
  if (hidden) {
    return (
      <div onClick={onClick} className={`w-14 h-20 sm:w-20 sm:h-28 rounded-lg bg-blue-800 border-2 border-slate-200 shadow-xl flex items-center justify-center relative overflow-hidden ${className} cursor-pointer hover:brightness-110 transition-transform`}>
        <div className="absolute inset-0 bg-[url('https://www.transparenttextures.com/patterns/diagmonds-light.png')] opacity-30"></div>
        <div className="w-8 h-8 rounded-full bg-blue-600/50 flex items-center justify-center border border-blue-400/30">
           <div className="text-white/50 text-xs">CR</div>
        </div>
      </div>
    );
  }

  if (!card) return null;

  return (
    <div 
      onClick={onClick}
      className={`
        w-14 h-20 sm:w-20 sm:h-28 bg-white rounded-lg shadow-xl flex flex-col justify-between p-1.5 
        ${selected ? 'ring-4 ring-yellow-400 -translate-y-4 z-10' : 'hover:-translate-y-1'} 
        transition-all duration-200 cursor-pointer relative ${className} border border-slate-300
      `}
    >
      <div className={`text-xs sm:text-base font-bold leading-none ${getCardColor(card.suit)}`}>
        {card.rank}
        <div className="text-[10px] sm:text-xs">{getSuitSymbol(card.suit)}</div>
      </div>
      <div className={`absolute inset-0 flex items-center justify-center text-4xl sm:text-5xl ${getCardColor(card.suit)} opacity-10 pointer-events-none`}>
        {getSuitSymbol(card.suit)}
      </div>
      <div className={`text-xs sm:text-base font-bold leading-none self-end rotate-180 ${getCardColor(card.suit)}`}>
        {card.rank}
        <div className="text-[10px] sm:text-xs">{getSuitSymbol(card.suit)}</div>
      </div>
    </div>
  );
};

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

const PitBossModal = ({ onClose, advice }: { onClose: () => void, advice: string }) => (
  <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-in fade-in">
    <div className="bg-[#1a1c29] border border-yellow-500/50 rounded-2xl max-w-md w-full p-6 shadow-2xl relative">
      <div className="absolute -top-10 left-1/2 -translate-x-1/2 w-20 h-20 bg-[#1a1c29] rounded-full border-2 border-yellow-500 flex items-center justify-center shadow-lg">
        <Sparkles className="text-yellow-400 w-10 h-10 animate-pulse" />
      </div>
      <h3 className="text-xl font-bold text-yellow-400 text-center mt-8 mb-4">The Pit Boss Says...</h3>
      <div className="text-white text-lg leading-relaxed text-center min-h-[100px] flex items-center justify-center">
        {advice ? `"${advice}"` : <span className="animate-pulse">Analyzing the table...</span>}
      </div>
      <button onClick={onClose} className="mt-6 w-full py-3 bg-yellow-600 hover:bg-yellow-500 text-black font-bold rounded-xl transition-colors">
        Got it, Boss
      </button>
    </div>
  </div>
);

// --- POKER GAME (Integrated with Linera) ---
const PokerGame = ({ onExit }: { onExit: () => void }) => {
  const { game: lineraGame, profile, loading, actions } = usePoker();
  const [deck, setDeck] = useState<Card[]>([]);
  const [communityCards, setCommunityCards] = useState<Card[]>([]);
  const [players, setPlayers] = useState<Player[]>([]);
  const [pot, setPot] = useState(0);
  const [turnIndex, setTurnIndex] = useState(0);
  const [gameStage, setGameStage] = useState<'preflop' | 'flop' | 'turn' | 'river' | 'showdown'>('preflop');
  const [currentBet, setCurrentBet] = useState(0);
  const [winnerMessage, setWinnerMessage] = useState<string | null>(null);
  const [showPitBoss, setShowPitBoss] = useState(false);
  const [pitBossAdvice, setPitBossAdvice] = useState("");
  const [botMessages, setBotMessages] = useState<Record<number, string>>({});
  const [playerName, setPlayerName] = useState('');

  // Sync with Linera
  useEffect(() => {
    if (profile?.balance) {
      // Update player chips from Linera balance
      setPlayers(prev => prev.map((p, i) => i === 0 ? { ...p, chips: Number(profile.balance) / 1e9 } : p));
    }
  }, [profile]);

  useEffect(() => {
    if (!lineraGame && !loading && !playerName) {
      const name = prompt("Enter your name to start:");
      if (name) {
        setPlayerName(name);
        actions.startGame(name);
      }
    }
  }, [lineraGame, loading]);

  useEffect(() => { 
    if (!lineraGame) startNewHand(); 
  }, []);

  const startNewHand = () => {
    const newDeck = shuffleDeck(createDeck());
    const newPlayers: Player[] = [
      { id: 0, name: 'You', cards: [newDeck.pop()!, newDeck.pop()!], chips: profile ? Number(profile.balance) / 1e9 : 1000, isFolded: false, isActive: true, currentBet: 0, avatar: '😎' },
      { id: 1, name: 'Bot 1', cards: [newDeck.pop()!, newDeck.pop()!], chips: 1000, isFolded: false, isActive: true, currentBet: 0, avatar: '🤖', personality: 'Aggressive' },
      { id: 2, name: 'Bot 2', cards: [newDeck.pop()!, newDeck.pop()!], chips: 1000, isFolded: false, isActive: true, currentBet: 0, avatar: '🤠', personality: 'Cautious' },
      { id: 3, name: 'Bot 3', cards: [newDeck.pop()!, newDeck.pop()!], chips: 1000, isFolded: false, isActive: true, currentBet: 0, avatar: '🦊', personality: 'Wild' },
    ];
    setDeck(newDeck);
    setPlayers(newPlayers);
    setCommunityCards([]);
    setPot(0);
    setGameStage('preflop');
    setTurnIndex(0);
    setWinnerMessage(null);
    setBotMessages({});
  };

  const triggerBotChat = async (playerId: number, action: string) => {
    if (Math.random() > 0.3) return;
    const player = players.find(p => p.id === playerId);
    if (!player) return;
    const prompt = `You are a poker player named ${player.name} with a ${player.personality} playing style. You just decided to ${action}. Write a very short, punchy, 1-sentence trash-talk line or reaction to the table. Don't use hashtags.`;
    const msg = await callGemini(prompt);
    setBotMessages(prev => ({ ...prev, [playerId]: msg }));
    setTimeout(() => {
      setBotMessages(prev => {
        const next = { ...prev };
        delete next[playerId];
        return next;
      });
    }, 4000);
  };
  
  const handleAction = async (type: string) => {
    if (type === 'fold') {
      await actions.fold();
      nextStage();
    } else if (type === 'call') {
      await actions.call();
      setPot(p => p + 50);
      nextStage();
    } else if (type === 'raise') {
      const amount = prompt("Enter raise amount:");
      if (amount) {
        await actions.raise((parseFloat(amount) * 1e9).toString());
        setPot(p => p + parseFloat(amount));
        nextStage();
      }
    }
  };

  useEffect(() => {
     if (turnIndex !== 0 && !winnerMessage) {
        const timer = setTimeout(() => {
           const actions = ['call', 'raise', 'fold'];
           const randomAction = actions[Math.floor(Math.random() * actions.length)];
           triggerBotChat(players[turnIndex]?.id || 1, randomAction);
           setTurnIndex((turnIndex + 1) % 4);
        }, 2000);
        return () => clearTimeout(timer);
     }
  }, [turnIndex, winnerMessage]);

  const askPitBoss = async () => {
    setShowPitBoss(true);
    setPitBossAdvice("");
    const myHand = players[0]?.cards.map(c => `${c.rank}${c.suit}`).join(", ") || "";
    const comm = communityCards.map(c => `${c.rank}${c.suit}`).join(", ");
    const prompt = `You are a casino pit boss advising a player in Texas Hold'em. User Hand: ${myHand}. Community Cards: ${comm || "None yet"}. Pot Size: ${pot}. Stage: ${gameStage}. Give brief, strategic advice in 1-2 sentences. Be cool and professional.`;
    const advice = await callGemini(prompt);
    setPitBossAdvice(advice);
  };

  const nextStage = () => {
     if (gameStage === 'preflop') { setCommunityCards([deck[0], deck[1], deck[2]]); setGameStage('flop'); }
     else if (gameStage === 'flop') { setCommunityCards(prev => [...prev, deck[3]]); setGameStage('turn'); }
     else if (gameStage === 'turn') { setCommunityCards(prev => [...prev, deck[4]]); setGameStage('river'); }
     else if (gameStage === 'river') { 
       const win = evaluateHandStrength(players[0]?.cards || [], communityCards);
       setWinnerMessage(`You Win with ${win.name}!`);
       setGameStage('showdown');
     }
     setTurnIndex(0);
  };

  return (
    <div className="flex flex-col h-screen bg-[#1e293b] overflow-hidden font-sans">
      {showPitBoss && <PitBossModal onClose={() => setShowPitBoss(false)} advice={pitBossAdvice} />}

      <div className="h-14 bg-[#0f172a] flex items-center justify-between px-4 shadow-xl z-20 border-b border-slate-700">
        <button onClick={onExit} className="flex items-center text-slate-400 hover:text-white"><ArrowLeft size={20} className="mr-2" /> Lobby</button>
        <div className="text-xl font-bold text-yellow-500 tracking-wider">TEXAS HOLD'EM</div>
        <div className="flex gap-4">
           <button 
             onClick={askPitBoss}
             className="flex items-center gap-2 px-3 py-1 bg-gradient-to-r from-purple-600 to-indigo-600 rounded-full text-xs font-bold text-white hover:scale-105 transition-transform shadow-[0_0_15px_rgba(124,58,237,0.5)] border border-white/20"
           >
             <Sparkles size={14} className="text-yellow-300" /> Ask Pit Boss
           </button>
           <div className="flex items-center text-yellow-400 font-mono"><DollarSign size={16} /> Pot: {pot}</div>
        </div>
      </div>

      <div className="flex-1 relative flex items-center justify-center p-2 bg-[url('https://www.transparenttextures.com/patterns/black-felt.png')]">
        <div className="w-full max-w-5xl aspect-[1.8/1] bg-[#2e5c46] rounded-[150px] border-[16px] border-[#4a3627] shadow-[inset_0_0_100px_rgba(0,0,0,0.6),0_20px_50px_rgba(0,0,0,0.5)] relative flex items-center justify-center">
          <div className="absolute inset-0 rounded-[135px] bg-[url('https://www.transparenttextures.com/patterns/green-felt.png')] opacity-60 pointer-events-none mix-blend-overlay"></div>
          <div className="absolute text-emerald-900/20 font-black text-6xl tracking-widest pointer-events-none select-none">POKER</div>

          <div className="flex gap-2 sm:gap-4 z-10">
            {communityCards.map((c, i) => <CardComponent key={c.id} card={c} />)}
            {Array(5 - communityCards.length).fill(0).map((_, i) => (
              <div key={i} className="w-14 h-20 sm:w-20 sm:h-28 border-2 border-emerald-700/30 rounded-lg bg-emerald-900/20 shadow-inner" />
            ))}
          </div>

          {players.map((p, i) => {
             const pos = ['bottom-[-30px] left-1/2 -translate-x-1/2', 'left-[-40px] top-1/2 -translate-y-1/2', 'top-[-40px] left-1/2 -translate-x-1/2', 'right-[-40px] top-1/2 -translate-y-1/2'][i];
             return (
               <div key={p.id} className={`absolute ${pos} flex flex-col items-center z-20`}>
                 {botMessages[p.id] && (
                    <div className="absolute -top-16 bg-white text-black text-xs p-2 rounded-lg shadow-lg max-w-[150px] text-center animate-in zoom-in fade-in duration-300 z-50 pointer-events-none">
                       <div className="absolute bottom-[-6px] left-1/2 -translate-x-1/2 w-3 h-3 bg-white rotate-45"></div>
                       {botMessages[p.id]}
                    </div>
                 )}
                 <div className={`w-14 h-14 rounded-full border-4 ${i===0 ? 'border-yellow-400 shadow-[0_0_20px_rgba(250,204,21,0.5)]' : 'border-slate-600'} bg-slate-800 flex items-center justify-center text-2xl shadow-lg relative`}>
                    {p.avatar}
                    <div className="absolute -bottom-2 bg-black/80 text-white text-[10px] px-2 rounded-full border border-white/20 whitespace-nowrap">{p.name}</div>
                 </div>
                 <div className="flex -space-x-6 mt-2">
                   {p.cards.map((c, idx) => (
                     <div key={idx} className={`transform ${idx===0?'-rotate-6':'rotate-6'} origin-bottom transition-transform hover:-translate-y-2`}>
                       <CardComponent card={c} hidden={i !== 0 && gameStage !== 'showdown'} className="scale-75 sm:scale-90" />
                     </div>
                   ))}
                 </div>
               </div>
             )
          })}
          
          {winnerMessage && (
            <div className="absolute inset-0 z-50 flex flex-col items-center justify-center bg-black/70 rounded-[140px] backdrop-blur-sm animate-in fade-in zoom-in duration-300">
               <Trophy size={64} className="text-yellow-400 mb-4 drop-shadow-[0_0_15px_rgba(250,204,21,0.8)]" />
               <div className="text-3xl font-bold text-white mb-6 drop-shadow-md">{winnerMessage}</div>
               <button onClick={startNewHand} className="px-8 py-3 bg-gradient-to-r from-yellow-500 to-yellow-600 hover:from-yellow-400 hover:to-yellow-500 text-slate-900 font-bold rounded-full shadow-lg transform hover:scale-105 transition-all flex items-center">
                 <RefreshCw className="mr-2" /> Play Again
               </button>
            </div>
          )}
        </div>
      </div>

      <div className="h-20 bg-[#0f172a] border-t border-slate-700 flex items-center justify-center gap-4 px-4">
         <button onClick={() => handleAction('fold')} className="px-6 py-2 rounded-full bg-red-900/50 text-red-400 border border-red-800 hover:bg-red-900 transition-colors font-bold uppercase tracking-wider">Fold</button>
         <button onClick={() => handleAction('call')} className="px-6 py-2 rounded-full bg-blue-900/50 text-blue-400 border border-blue-800 hover:bg-blue-900 transition-colors font-bold uppercase tracking-wider">Check/Call</button>
         <button onClick={() => handleAction('raise')} className="px-6 py-2 rounded-full bg-yellow-900/50 text-yellow-400 border border-yellow-800 hover:bg-yellow-900 transition-colors font-bold uppercase tracking-wider">Raise</button>
      </div>
    </div>
  );
};

// --- RUMMY GAME (Integrated with Linera) ---
const RummyGame = ({ onExit }: { onExit: () => void }) => {
  const { game: lineraGame, profile, loading, actions } = useRummy();
  const [deck, setDeck] = useState<Card[]>([]);
  const [playerHand, setPlayerHand] = useState<Card[]>([]);
  const [discardPile, setDiscardPile] = useState<Card[]>([]);
  const [selectedCard, setSelectedCard] = useState<number | null>(null);
  const [showPitBoss, setShowPitBoss] = useState(false);
  const [pitBossAdvice, setPitBossAdvice] = useState("");
  const [playerName, setPlayerName] = useState('');

  useEffect(() => {
    if (!lineraGame && !loading && !playerName) {
      const name = prompt("Enter your name to start:");
      if (name) {
        setPlayerName(name);
        actions.startGame(name);
      }
    }
  }, [lineraGame, loading]);

  useEffect(() => {
    if (!lineraGame) {
      const d = shuffleDeck(createDeck());
      setPlayerHand(d.slice(0, 13));
      setDeck(d.slice(13));
      setDiscardPile([]);
    } else if (lineraGame.players && lineraGame.players[0]) {
      // Sync with Linera game state
      const hand = lineraGame.players[0].hand || [];
      if (hand.length > 0) {
        // Convert Linera card format to our Card format
        const convertedHand = hand.map((card: any, idx: number) => ({
          id: `card-${idx}`,
          suit: card.suit || 'hearts',
          rank: card.rank || 'A',
          value: card.value || 14,
        }));
        setPlayerHand(convertedHand);
      }
      if (lineraGame.discardPile) {
        setDiscardPile(lineraGame.discardPile.map((card: any, idx: number) => ({
          id: `discard-${idx}`,
          suit: card.suit || 'hearts',
          rank: card.rank || 'A',
          value: card.value || 14,
        })));
      }
    }
  }, [lineraGame]);

  const askPitBoss = async () => {
    setShowPitBoss(true);
    setPitBossAdvice(""); 
    const sorted = [...playerHand].sort((a,b) => a.value - b.value);
    const handStr = sorted.map(c => `${c.rank}${c.suit}`).join(", ");
    const prompt = `You are a rummy expert. I have these 13 cards: ${handStr}. Suggest how to group them or which card might be weak. Keep it short.`;
    const advice = await callGemini(prompt);
    setPitBossAdvice(advice);
  };

  const handleDrawFromDeck = async () => {
    await actions.drawFromDeck();
    if (deck.length > 0) {
      const newCard = deck[0];
      setPlayerHand([...playerHand, newCard]);
      setDeck(deck.slice(1));
    }
  };

  const handleDrawFromDiscard = async () => {
    await actions.drawFromDiscard();
    if (discardPile.length > 0) {
      const newCard = discardPile[discardPile.length - 1];
      setPlayerHand([...playerHand, newCard]);
      setDiscardPile(discardPile.slice(0, -1));
    }
  };

  const handleDiscard = async () => {
    if (selectedCard !== null) {
      await actions.discardCard(selectedCard);
      const card = playerHand[selectedCard];
      setDiscardPile([...discardPile, card]);
      setPlayerHand(playerHand.filter((_, i) => i !== selectedCard));
      setSelectedCard(null);
    }
  };

  const handleDeclare = async () => {
    await actions.declare();
  };

  return (
    <div className="flex flex-col h-screen bg-[#0f3628] overflow-hidden font-sans">
      {showPitBoss && <PitBossModal onClose={() => setShowPitBoss(false)} advice={pitBossAdvice} />}

      <div className="h-14 bg-[#051f15] flex items-center justify-between px-4 shadow-xl z-20">
         <button onClick={onExit} className="flex items-center text-emerald-100 hover:text-white"><ArrowLeft size={20} className="mr-2" /> Lobby</button>
         <div className="text-xl font-bold text-yellow-500 tracking-wider">INDIAN RUMMY</div>
         <button 
             onClick={askPitBoss}
             className="flex items-center gap-2 px-3 py-1 bg-gradient-to-r from-purple-600 to-indigo-600 rounded-full text-xs font-bold text-white hover:scale-105 transition-transform shadow-[0_0_15px_rgba(124,58,237,0.5)] border border-white/20"
           >
             <Sparkles size={14} className="text-yellow-300" /> Ask Pit Boss
         </button>
      </div>

      <div className="flex-1 relative bg-[url('https://www.transparenttextures.com/patterns/green-felt.png')] flex flex-col justify-between p-4">
        <div className="absolute inset-0 bg-black/20 pointer-events-none" />
        
        <div className="flex justify-center z-10">
           <div className="w-16 h-16 rounded-full bg-emerald-900 border-2 border-emerald-700 flex items-center justify-center text-3xl shadow-lg">🤖</div>
        </div>

        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex gap-8 z-10 bg-emerald-900/40 p-8 rounded-3xl border border-white/5 backdrop-blur-sm shadow-2xl">
           <div className="relative group cursor-pointer hover:-translate-y-2 transition-transform" onClick={handleDrawFromDeck}>
             <div className="absolute -inset-2 bg-white/5 rounded-xl blur-md opacity-0 group-hover:opacity-100 transition-opacity" />
             <div className="w-20 h-28 bg-emerald-800 rounded-lg border-2 border-white/20 shadow-xl flex items-center justify-center">
               <div className="text-emerald-400 font-bold opacity-50">DECK</div>
             </div>
             <div className="absolute top-1 left-1 w-20 h-28 bg-emerald-800 rounded-lg border border-white/10 -z-10" />
             <div className="absolute top-2 left-2 w-20 h-28 bg-emerald-800 rounded-lg border border-white/10 -z-20" />
           </div>
           
           <div className="relative cursor-pointer hover:-translate-y-2 transition-transform" onClick={handleDrawFromDiscard}>
              <CardComponent card={discardPile[discardPile.length - 1] || {id:'x', suit:'hearts', rank:'A', value:14}} />
              <div className="absolute -bottom-6 w-full text-center text-xs font-bold text-emerald-300">OPEN PILE</div>
           </div>
        </div>

        <div className="z-10 mb-4 overflow-x-auto pb-4 px-4">
           <div className="flex justify-center min-w-max">
             <div className="flex -space-x-8 bg-black/30 p-4 rounded-2xl border border-white/5 backdrop-blur-md shadow-2xl">
               {playerHand.map((c, i) => (
                 <div key={i} className="hover:-translate-y-4 hover:z-50 transition-all duration-200" onClick={() => setSelectedCard(selectedCard === i ? null : i)}>
                   <CardComponent card={c} selected={selectedCard === i} />
                 </div>
               ))}
             </div>
           </div>
        </div>

        <div className="z-10 flex justify-center gap-4 pb-4">
          <button onClick={handleDiscard} disabled={selectedCard === null} className="px-6 py-2 bg-red-600 hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-bold rounded-lg">Discard</button>
          <button onClick={handleDeclare} className="px-6 py-2 bg-yellow-600 hover:bg-yellow-700 text-white font-bold rounded-lg">Declare</button>
        </div>
      </div>
    </div>
  );
};

// --- MAIN MENU ---
const FloatingChips = () => {
  const [chips, setChips] = useState<any[]>([]);

  useEffect(() => {
    const newChips = Array.from({ length: 20 }).map((_, i) => ({
      id: i,
      left: Math.random() * 100,
      duration: 10 + Math.random() * 20,
      delay: Math.random() * 10,
      size: 20 + Math.random() * 40,
      color: ['#ef4444', '#10b981', '#3b82f6', '#f59e0b'][Math.floor(Math.random() * 4)]
    }));
    setChips(newChips);
  }, []);

  return (
    <>
       <style>{`
          @keyframes float-title {
            0%, 100% { transform: translateY(0px); }
            50% { transform: translateY(-15px); }
          }
          .animate-float-title {
            animation: float-title 4s ease-in-out infinite;
          }
          
          @keyframes chip-fall {
             0% { transform: translateY(-10vh) rotate(0deg); opacity: 0; }
             10% { opacity: 0.6; }
             90% { opacity: 0.6; }
             100% { transform: translateY(110vh) rotate(720deg); opacity: 0; }
          }
           @keyframes shine {
            100% { transform: translateX(100%) skewX(12deg); }
          }
          .animate-shine {
            animation: shine 1s;
          }
       `}</style>
       <div className="absolute inset-0 overflow-hidden pointer-events-none z-0">
          {chips.map((chip) => (
            <div
              key={chip.id}
              className="absolute rounded-full border-4 border-dashed opacity-40 shadow-lg"
              style={{
                left: `${chip.left}%`,
                width: `${chip.size}px`,
                height: `${chip.size}px`,
                backgroundColor: chip.color,
                borderColor: 'rgba(255,255,255,0.3)',
                animation: `chip-fall ${chip.duration}s linear infinite`,
                animationDelay: `-${chip.delay}s`,
                top: '-50px'
              }}
            >
               <div className="w-full h-full flex items-center justify-center text-white/20 text-[10px] font-bold">CR</div>
            </div>
          ))}
       </div>
    </>
  );
};

export default function CasinoRoyale() {
  const [game, setGame] = useState<'menu' | 'poker' | 'rummy' | 'roulette'>('menu');
  const [walletBalance, setWalletBalance] = useState<number>(0);
  const [loadingBalance, setLoadingBalance] = useState(false);

  // Load wallet balance on mount
  useEffect(() => {
    loadWalletBalance();
  }, []);

  const loadWalletBalance = async () => {
    try {
      setLoadingBalance(true);
      // Try to get balance from any game's profile
      const response = await fetch('/config.json');
      if (response.ok) {
        const config = await response.json();
        // Query balance from Linera service
        const graphqlResponse = await fetch(config.nodeServiceURL || 'http://localhost:8081', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            query: `
              query {
                getBalances {
                  chain
                  amount
                }
              }
            `
          })
        });
        if (graphqlResponse.ok) {
          const data = await graphqlResponse.json();
          if (data.data?.getBalances && data.data.getBalances.length > 0) {
            const total = data.data.getBalances.reduce((sum: number, b: any) => sum + Number(b.amount || 0), 0);
            setWalletBalance(total / 1e9);
          }
        }
      }
    } catch (error) {
      console.error('Error loading balance:', error);
    } finally {
      setLoadingBalance(false);
    }
  };

  const handleRequestFaucet = async () => {
    try {
      setLoadingBalance(true);
      // Try faucet endpoint first
      let response = await fetch('http://localhost:8080/faucet', {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' }
      });
      
      if (!response.ok) {
        // Try root endpoint
        response = await fetch('http://localhost:8080', {
          method: 'GET'
        });
      }
      
      if (response.ok) {
        alert('Faucet tokens requested! Please wait a moment and refresh your balance.');
        setTimeout(() => {
          loadWalletBalance();
        }, 3000);
      } else {
        alert('Faucet request failed. Please use Linera CLI:\n\nlinera wallet request-chain --faucet http://localhost:8080\n\nOr make sure the faucet is running on port 8080.');
      }
    } catch (error) {
      console.error('Faucet error:', error);
      alert('Faucet request failed. Please use Linera CLI:\n\nlinera wallet request-chain --faucet http://localhost:8080\n\nOr make sure the faucet is running on port 8080.');
    } finally {
      setLoadingBalance(false);
    }
  };

  if (game === 'poker') return <PokerGame onExit={() => setGame('menu')} />;
  if (game === 'rummy') return <RummyGame onExit={() => setGame('menu')} />;
  if (game === 'roulette') return <RouletteGame onExit={() => setGame('menu')} />;

  return (
    <div className="min-h-screen bg-[#050505] text-white font-sans overflow-hidden relative selection:bg-yellow-500/30">
      <FloatingChips />
      
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_#2a1c0e_0%,_#000000_100%)] z-[-2]"></div>
      <div className="absolute inset-0 bg-[url('https://www.transparenttextures.com/patterns/dark-leather.png')] opacity-30 z-[-1]"></div>
      <div className="absolute inset-0 bg-gradient-to-b from-black/20 via-transparent to-black z-[-1]"></div>
      
      <div className="absolute top-0 left-1/4 w-96 h-96 bg-yellow-600/10 rounded-full blur-[120px] pointer-events-none animate-pulse"></div>
      <div className="absolute bottom-0 right-1/4 w-96 h-96 bg-red-900/10 rounded-full blur-[120px] pointer-events-none animate-pulse delay-700"></div>

      <div className="relative z-10 container mx-auto px-6 py-8 flex flex-col h-full justify-between">
        
        {/* Wallet Balance Header */}
        <div className="absolute top-4 right-4 z-20 flex items-center gap-3">
          <div className="bg-[#1a1c29] border border-yellow-500/30 rounded-full px-4 py-2 flex items-center gap-2 shadow-lg">
            <DollarSign size={18} className="text-yellow-400" />
            <span className="font-mono font-bold text-yellow-400">
              {loadingBalance ? '...' : walletBalance.toFixed(2)} LIN
            </span>
          </div>
          <button
            onClick={handleRequestFaucet}
            disabled={loadingBalance}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-full text-sm font-bold transition-colors shadow-lg"
          >
            {loadingBalance ? 'Loading...' : 'Get Tokens'}
          </button>
          <button
            onClick={loadWalletBalance}
            disabled={loadingBalance}
            className="px-3 py-2 bg-slate-800 hover:bg-slate-700 disabled:opacity-50 text-slate-300 rounded-full text-sm transition-colors"
            title="Refresh Balance"
          >
            <RefreshCw size={16} className={loadingBalance ? 'animate-spin' : ''} />
          </button>
        </div>
        
        <div className="flex flex-col items-center justify-center mt-4 mb-8">
           <div className="flex items-center space-x-2 text-yellow-500/80 mb-4 tracking-[0.3em] text-xs font-bold uppercase">
              <Sparkles size={12} />
              <span>The Exclusive Club</span>
              <Sparkles size={12} />
           </div>
           
           <div className="relative animate-float-title">
             <h1 className="text-7xl md:text-9xl font-black text-transparent bg-clip-text bg-gradient-to-b from-[#ffd700] via-[#bf953f] to-[#b38728] drop-shadow-[0_0_25px_rgba(255,215,0,0.3)] tracking-tighter" style={{ fontFamily: 'serif' }}>
               CASINO
             </h1>
             <div className="absolute -bottom-4 md:-bottom-8 left-0 right-0 text-center">
                <span className="text-3xl md:text-5xl font-light text-white tracking-[0.5em] drop-shadow-lg uppercase" style={{ textShadow: '0 0 10px rgba(255,255,255,0.5)' }}>ROYALE</span>
             </div>
           </div>
        </div>

        <div className="flex-1 flex items-center justify-center">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 md:gap-12 w-full max-w-7xl">
            
            <div 
              onClick={() => setGame('poker')}
              className="group relative h-[450px] w-full bg-[#121212] rounded-[24px] border border-white/5 cursor-pointer transition-all duration-500 hover:-translate-y-4 hover:shadow-[0_0_50px_rgba(16,185,129,0.3)] overflow-hidden flex flex-col backdrop-blur-sm bg-opacity-80"
            >
              <div className="h-2/3 bg-gradient-to-br from-emerald-900 via-emerald-950 to-black relative overflow-hidden">
                 <div className="absolute inset-0 bg-[url('https://www.transparenttextures.com/patterns/poker-chip.png')] opacity-20 group-hover:scale-110 transition-transform duration-700"></div>
                 <div className="absolute top-4 right-4 bg-emerald-500/20 px-3 py-1 rounded-full border border-emerald-500/30 text-emerald-400 text-xs font-bold tracking-widest uppercase backdrop-blur-sm">
                   Live Tables
                 </div>
                 <div className="absolute bottom-4 left-4">
                    <DollarSign size={48} className="text-emerald-500 drop-shadow-[0_0_10px_rgba(16,185,129,0.5)]" />
                 </div>
              </div>
              <div className="h-1/3 bg-[#0a0a0a] p-6 flex flex-col justify-between border-t border-white/5 relative z-10">
                 <div>
                   <h2 className="text-2xl font-bold text-white mb-1 font-serif">Texas Hold'em</h2>
                   <p className="text-gray-500 text-sm">High stakes, no limit.</p>
                 </div>
                 <div className="flex items-center justify-between mt-4">
                    <div className="flex -space-x-2">
                       <div className="w-8 h-8 rounded-full bg-slate-700 border border-black"></div>
                       <div className="w-8 h-8 rounded-full bg-slate-600 border border-black"></div>
                       <div className="w-8 h-8 rounded-full bg-slate-500 border border-black flex items-center justify-center text-[10px] text-white">+2k</div>
                    </div>
                    <div className="w-10 h-10 rounded-full bg-emerald-600 flex items-center justify-center group-hover:bg-emerald-500 transition-colors shadow-lg">
                       <ArrowLeft className="rotate-180 text-white" size={20} />
                    </div>
                 </div>
              </div>
            </div>

            <div 
              onClick={() => setGame('rummy')}
              className="group relative h-[450px] w-full bg-[#121212] rounded-[24px] border border-white/5 cursor-pointer transition-all duration-500 hover:-translate-y-4 hover:shadow-[0_0_50px_rgba(99,102,241,0.3)] overflow-hidden flex flex-col backdrop-blur-sm bg-opacity-80"
            >
              <div className="h-2/3 bg-gradient-to-br from-indigo-900 via-indigo-950 to-black relative overflow-hidden">
                 <div className="absolute inset-0 bg-[url('https://www.transparenttextures.com/patterns/argyle.png')] opacity-10 group-hover:scale-110 transition-transform duration-700"></div>
                 <div className="absolute top-4 right-4 bg-indigo-500/20 px-3 py-1 rounded-full border border-indigo-500/30 text-indigo-400 text-xs font-bold tracking-widest uppercase backdrop-blur-sm">
                   Skill Based
                 </div>
                 <div className="absolute bottom-4 left-4">
                    <Layers size={48} className="text-indigo-500 drop-shadow-[0_0_10px_rgba(99,102,241,0.5)]" />
                 </div>
              </div>
              <div className="h-1/3 bg-[#0a0a0a] p-6 flex flex-col justify-between border-t border-white/5 relative z-10">
                 <div>
                   <h2 className="text-2xl font-bold text-white mb-1 font-serif">Indian Rummy</h2>
                   <p className="text-gray-500 text-sm">13 Card classic strategy.</p>
                 </div>
                 <div className="flex items-center justify-between mt-4">
                    <div className="text-xs text-indigo-400 font-bold uppercase tracking-widest border-b border-indigo-500/30 pb-1">
                       Multiplayer Ready
                    </div>
                    <div className="w-10 h-10 rounded-full bg-indigo-600 flex items-center justify-center group-hover:bg-indigo-500 transition-colors shadow-lg">
                       <ArrowLeft className="rotate-180 text-white" size={20} />
                    </div>
                 </div>
              </div>
            </div>

            <div 
              onClick={() => setGame('roulette')}
              className="group relative h-[450px] w-full bg-[#121212] rounded-[24px] border border-white/5 cursor-pointer transition-all duration-500 hover:-translate-y-4 hover:shadow-[0_0_50px_rgba(239,68,68,0.3)] overflow-hidden flex flex-col backdrop-blur-sm bg-opacity-80"
            >
              <div className="h-2/3 bg-gradient-to-br from-red-900 via-red-950 to-black relative overflow-hidden">
                 <div className="absolute inset-0 bg-[url('https://www.transparenttextures.com/patterns/carbon-fibre.png')] opacity-20 group-hover:scale-110 transition-transform duration-700"></div>
                 <div className="absolute top-4 right-4 bg-red-500/20 px-3 py-1 rounded-full border border-red-500/30 text-red-400 text-xs font-bold tracking-widest uppercase backdrop-blur-sm">
                   Instant Win
                 </div>
                 <div className="absolute bottom-4 left-4">
                    <RotateCcw size={48} className="text-red-500 drop-shadow-[0_0_10px_rgba(239,68,68,0.5)]" />
                 </div>
                 <div className="absolute -right-12 -bottom-12 w-48 h-48 rounded-full border-[16px] border-dashed border-white/10 opacity-50 animate-spin-slow"></div>
              </div>
              <div className="h-1/3 bg-[#0a0a0a] p-6 flex flex-col justify-between border-t border-white/5 relative z-10">
                 <div>
                   <h2 className="text-2xl font-bold text-white mb-1 font-serif">Royal Roulette</h2>
                   <p className="text-gray-500 text-sm">Spin the wheel of fortune.</p>
                 </div>
                 <div className="flex items-center justify-between mt-4">
                     <div className="flex items-center space-x-2">
                        <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse"></span>
                        <span className="text-xs text-red-400 font-bold uppercase tracking-widest">Live Now</span>
                     </div>
                    <div className="w-10 h-10 rounded-full bg-red-600 flex items-center justify-center group-hover:bg-red-500 transition-colors shadow-lg">
                       <ArrowLeft className="rotate-180 text-white" size={20} />
                    </div>
                 </div>
              </div>
            </div>

          </div>
        </div>
        
        <div className="mt-12 border-t border-white/5 pt-6 flex justify-between items-center text-xs text-gray-500 uppercase tracking-widest">
           <div className="flex space-x-6">
              <span className="hover:text-yellow-500 cursor-pointer transition-colors">Provably Fair</span>
              <span className="hover:text-yellow-500 cursor-pointer transition-colors">VIP Club</span>
              <span className="hover:text-yellow-500 cursor-pointer transition-colors">Support</span>
           </div>
           <div className="flex items-center">
              <span className="w-2 h-2 rounded-full bg-green-500 mr-2"></span>
              2,405 Players Online
           </div>
        </div>

      </div>
    </div>
  );
}
