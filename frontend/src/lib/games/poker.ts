import { useQuery, useMutation } from '@apollo/client';
import { getPokerClient } from '../linera/client';
import { 
  GET_POKER_SINGLE_PLAYER_DATA, 
  GET_POKER_PROFILE
} from '../linera/queries';
import {
  POKER_GET_BALANCE,
  POKER_START_SINGLE_PLAYER_GAME,
  POKER_BET,
  POKER_FOLD,
  POKER_CALL,
  POKER_RAISE
} from '../linera/mutations';

export function usePoker() {
  const client = getPokerClient();
  
  const { data: gameData, loading: gameLoading, refetch: refetchGame } = useQuery(
    GET_POKER_SINGLE_PLAYER_DATA,
    { client }
  );
  
  const { data: profileData, loading: profileLoading, refetch: refetchProfile } = useQuery(
    GET_POKER_PROFILE,
    { client }
  );
  
  const [getBalance] = useMutation(POKER_GET_BALANCE, { client });
  const [startGame] = useMutation(POKER_START_SINGLE_PLAYER_GAME, { client });
  const [bet] = useMutation(POKER_BET, { client });
  const [fold] = useMutation(POKER_FOLD, { client });
  const [call] = useMutation(POKER_CALL, { client });
  const [raise] = useMutation(POKER_RAISE, { client });

  return {
    game: gameData?.singlePlayerData?.game,
    profile: profileData?.getProfile,
    loading: gameLoading || profileLoading,
    actions: {
      getBalance: async () => {
        try {
          await getBalance();
          await refetchProfile();
        } catch (error) {
          console.error('Error getting balance:', error);
        }
      },
      startGame: async (name: string) => {
        try {
          await startGame({ variables: { name } });
          await refetchGame();
        } catch (error) {
          console.error('Error starting game:', error);
        }
      },
      bet: async (amount: string) => {
        try {
          await bet({ variables: { amount } });
          await refetchGame();
        } catch (error) {
          console.error('Error placing bet:', error);
        }
      },
      fold: async () => {
        try {
          await fold();
          await refetchGame();
        } catch (error) {
          console.error('Error folding:', error);
        }
      },
      call: async () => {
        try {
          await call();
          await refetchGame();
        } catch (error) {
          console.error('Error calling:', error);
        }
      },
      raise: async (amount: string) => {
        try {
          await raise({ variables: { amount } });
          await refetchGame();
        } catch (error) {
          console.error('Error raising:', error);
        }
      },
    },
    refetch: () => {
      refetchGame();
      refetchProfile();
    },
  };
}
