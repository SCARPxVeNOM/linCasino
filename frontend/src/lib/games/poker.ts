import { useQuery, useMutation } from '@apollo/client';
import { getPokerClient, getConfig } from '../linera/client';
import { 
  GET_POKER_SINGLE_PLAYER_DATA, 
  GET_POKER_MULTI_PLAYER_DATA,
  GET_POKER_PROFILE,
  GET_POKER_LOBBIES,
} from '../linera/queries';
import {
  POKER_GET_BALANCE,
  POKER_START_SINGLE_PLAYER_GAME,
  POKER_BET,
  POKER_FOLD,
  POKER_CALL,
  POKER_RAISE,
  POKER_CREATE_LOBBY,
  POKER_JOIN_LOBBY,
  POKER_START_LOBBY,
  POKER_CREATE_TABLE,
  POKER_SIT,
  POKER_LEAVE,
  POKER_PLAYER_ACTION,
  POKER_HEARTBEAT,
} from '../linera/mutations';

export function usePoker(mode: 'single' | 'multi' = 'single') {
  const client = getPokerClient();
  const config = getConfig();

  const {
    data: singlePlayerData,
    loading: singleLoading,
    refetch: refetchSingle,
  } = useQuery(GET_POKER_SINGLE_PLAYER_DATA, {
    client,
    skip: mode !== 'single',
  });

  const {
    data: multiPlayerData,
    loading: multiLoading,
    refetch: refetchMulti,
  } = useQuery(GET_POKER_MULTI_PLAYER_DATA, {
    client,
    skip: mode !== 'multi',
    pollInterval: mode === 'multi' ? 800 : 0,
  });

  const {
    data: profileData,
    loading: profileLoading,
    refetch: refetchProfile,
  } = useQuery(GET_POKER_PROFILE, {
    client,
  });

  const {
    data: lobbyData,
    loading: lobbyLoading,
    refetch: refetchLobbies,
  } = useQuery(GET_POKER_LOBBIES, {
    client,
  });

  const [getBalance] = useMutation(POKER_GET_BALANCE, { client });
  const [startGame] = useMutation(POKER_START_SINGLE_PLAYER_GAME, { client });
  const [bet] = useMutation(POKER_BET, { client });
  const [fold] = useMutation(POKER_FOLD, { client });
  const [call] = useMutation(POKER_CALL, { client });
  const [raise] = useMutation(POKER_RAISE, { client });
  const [createLobby] = useMutation(POKER_CREATE_LOBBY, { client });
  const [joinLobby] = useMutation(POKER_JOIN_LOBBY, { client });
  const [createTable] = useMutation(POKER_CREATE_TABLE, { client });
  const [sit] = useMutation(POKER_SIT, { client });
  const [leave] = useMutation(POKER_LEAVE, { client });
  const [playerAction] = useMutation(POKER_PLAYER_ACTION, { client });
  const [heartbeat] = useMutation(POKER_HEARTBEAT, { client });

  return {
    game: singlePlayerData?.singlePlayerData?.game,
    multiGame: multiPlayerData?.multiPlayerData?.game,
    profile: profileData?.getProfile,
    lobbies: lobbyData?.openLobbies ?? [],
    loading: singleLoading || multiLoading || profileLoading || lobbyLoading,
    config,
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
          await refetchSingle();
        } catch (error) {
          console.error('Error starting game:', error);
        }
      },
      bet: async (amount: string) => {
        try {
          await bet({ variables: { amount } });
          await refetchSingle();
        } catch (error) {
          console.error('Error placing bet:', error);
        }
      },
      fold: async () => {
        try {
          await fold();
          await refetchSingle();
        } catch (error) {
          console.error('Error folding:', error);
        }
      },
      call: async () => {
        try {
          await call();
          await refetchSingle();
        } catch (error) {
          console.error('Error calling:', error);
        }
      },
      raise: async (amount: string) => {
        try {
          await raise({ variables: { amount } });
          await refetchSingle();
        } catch (error) {
          console.error('Error raising:', error);
        }
      },
      createLobby: async (maxPlayers: number) => {
        try {
          await createLobby({ variables: { maxPlayers } });
          await refetchLobbies();
        } catch (error) {
          console.error('Error creating lobby:', error);
        }
      },
      joinLobby: async (lobbyId: string, name: string) => {
        try {
          await joinLobby({ variables: { lobbyId, name } });
          await refetchLobbies();
        } catch (error) {
          console.error('Error joining lobby:', error);
        }
      },
      startLobby: async (lobbyId: string) => {
        try {
          await startLobby({ variables: { lobbyId } });
          await refetchLobbies();
        } catch (error) {
          console.error('Error starting lobby:', error);
        }
      },
      // Multiplayer helpers
      createTable: async (smallBlind: string, bigBlind: string, maxPlayers: number) => {
        try {
          await createTable({ variables: { smallBlind, bigBlind, maxPlayers } });
          await refetchMulti();
        } catch (error) {
          console.error('Error creating table:', error);
        }
      },
      sitAtTable: async (name: string) => {
        try {
          const tableChain = config.defaultChain;
          await sit({ variables: { tableChain, name } });
          await refetchMulti();
        } catch (error) {
          console.error('Error sitting at table:', error);
        }
      },
      leaveTable: async () => {
        try {
          const tableChain = config.defaultChain;
          await leave({ variables: { tableChain } });
          await refetchMulti();
        } catch (error) {
          console.error('Error leaving table:', error);
        }
      },
      playerAction: async (opts: { seatId: number; action: string; amount?: string }) => {
        try {
          const tableChain = config.defaultChain;
          const handId = multiPlayerData?.multiPlayerData?.game?.handId?.toString() ?? '0';
          await playerAction({
            variables: {
              tableChain,
              handId,
              seatId: opts.seatId,
              action: opts.action,
              amount: opts.amount,
            },
          });
          await refetchMulti();
        } catch (error) {
          console.error('Error performing player action:', error);
        }
      },
      sendHeartbeat: async () => {
        try {
          const tableChain = config.defaultChain;
          await heartbeat({ variables: { tableChain } });
          await refetchMulti();
        } catch (error) {
          console.error('Error sending heartbeat:', error);
        }
      },
    },
    refetch: () => {
      refetchSingle();
      refetchMulti();
      refetchProfile();
      refetchLobbies();
    },
  };
}
