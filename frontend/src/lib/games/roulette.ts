import { useQuery, useMutation } from '@apollo/client';
import { getRouletteClient } from '../linera/client';
import { 
  GET_ROULETTE_SINGLE_PLAYER_DATA, 
  GET_ROULETTE_PROFILE
} from '../linera/queries';
import {
  ROULETTE_GET_BALANCE,
  ROULETTE_START_SINGLE_PLAYER_GAME,
  ROULETTE_PLACE_BET,
  ROULETTE_SPIN
} from '../linera/mutations';

export function useRoulette() {
  const client = getRouletteClient();
  
  const { data: gameData, loading: gameLoading, error: gameError, refetch: refetchGame } = useQuery(
    GET_ROULETTE_SINGLE_PLAYER_DATA,
    { 
      client,
      errorPolicy: 'all', // Continue even if there are errors
      fetchPolicy: 'network-only'
    }
  );
  
  const { data: profileData, loading: profileLoading, error: profileError, refetch: refetchProfile } = useQuery(
    GET_ROULETTE_PROFILE,
    { 
      client,
      errorPolicy: 'all', // Continue even if there are errors
      fetchPolicy: 'network-only'
    }
  );
  
  const [getBalance] = useMutation(ROULETTE_GET_BALANCE, { client });
  const [startGame] = useMutation(ROULETTE_START_SINGLE_PLAYER_GAME, { client });
  const [placeBet] = useMutation(ROULETTE_PLACE_BET, { client });
  const [spin] = useMutation(ROULETTE_SPIN, { client });

  return {
    game: gameData?.singlePlayerData?.game || null,
    profile: profileData?.getProfile || null,
    loading: gameLoading || profileLoading,
    error: gameError || profileError,
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
      placeBet: async (betType: string, amount: string) => {
        try {
          await placeBet({ variables: { betType, amount } });
          await refetchGame();
        } catch (error) {
          console.error('Error placing bet:', error);
        }
      },
      spin: async () => {
        try {
          await spin();
          await refetchGame();
        } catch (error) {
          console.error('Error spinning:', error);
        }
      },
    },
    refetch: () => {
      refetchGame();
      refetchProfile();
    },
  };
}
