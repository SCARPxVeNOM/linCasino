import { useQuery, useMutation } from '@apollo/client';
import { getRummyClient } from '../linera/client';
import { 
  GET_RUMMY_SINGLE_PLAYER_DATA, 
  GET_RUMMY_PROFILE
} from '../linera/queries';
import {
  RUMMY_GET_BALANCE,
  RUMMY_START_SINGLE_PLAYER_GAME,
  RUMMY_DRAW_FROM_DECK,
  RUMMY_DRAW_FROM_DISCARD,
  RUMMY_DISCARD_CARD,
  RUMMY_DECLARE
} from '../linera/mutations';

export function useRummy() {
  const client = getRummyClient();
  
  const { data: gameData, loading: gameLoading, refetch: refetchGame } = useQuery(
    GET_RUMMY_SINGLE_PLAYER_DATA,
    { client }
  );
  
  const { data: profileData, loading: profileLoading, refetch: refetchProfile } = useQuery(
    GET_RUMMY_PROFILE,
    { client }
  );
  
  const [getBalance] = useMutation(RUMMY_GET_BALANCE, { client });
  const [startGame] = useMutation(RUMMY_START_SINGLE_PLAYER_GAME, { client });
  const [drawFromDeck] = useMutation(RUMMY_DRAW_FROM_DECK, { client });
  const [drawFromDiscard] = useMutation(RUMMY_DRAW_FROM_DISCARD, { client });
  const [discardCard] = useMutation(RUMMY_DISCARD_CARD, { client });
  const [declare] = useMutation(RUMMY_DECLARE, { client });

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
      drawFromDeck: async () => {
        try {
          await drawFromDeck();
          await refetchGame();
        } catch (error) {
          console.error('Error drawing from deck:', error);
        }
      },
      drawFromDiscard: async () => {
        try {
          await drawFromDiscard();
          await refetchGame();
        } catch (error) {
          console.error('Error drawing from discard:', error);
        }
      },
      discardCard: async (card: number) => {
        try {
          await discardCard({ variables: { card } });
          await refetchGame();
        } catch (error) {
          console.error('Error discarding card:', error);
        }
      },
      declare: async () => {
        try {
          await declare();
          await refetchGame();
        } catch (error) {
          console.error('Error declaring:', error);
        }
      },
    },
    refetch: () => {
      refetchGame();
      refetchProfile();
    },
  };
}
