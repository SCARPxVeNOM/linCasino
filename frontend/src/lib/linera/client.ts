import { ApolloClient, InMemoryCache, createHttpLink } from '@apollo/client';

export interface LineraConfig {
  nodeServiceURL: string;
  pokerAppId: string;
  rummyAppId: string;
  rouletteAppId: string;
  bankrollAppId: string;
  defaultChain: string;
  userChain1?: string;
  userChain2?: string;
  userChain3?: string;
  userChain4?: string;
  userChain5?: string;
  userChain6?: string;
  userChain7?: string;
  userChain8?: string;
}

let config: LineraConfig | null = null;
let pokerClient: ApolloClient<any> | null = null;
let rummyClient: ApolloClient<any> | null = null;
let rouletteClient: ApolloClient<any> | null = null;

export function initializeLineraClient(lineraConfig: LineraConfig) {
  config = lineraConfig;
  
  // Create separate clients for each game with their specific application endpoints
  // Format: http://localhost:8081/chains/{chainId}/applications/{appId}
  const baseURL = lineraConfig.nodeServiceURL;
  const chainId = lineraConfig.defaultChain;
  
  // Poker client
  if (lineraConfig.pokerAppId && chainId) {
    pokerClient = new ApolloClient({
      link: createHttpLink({
        uri: `${baseURL}/chains/${chainId}/applications/${lineraConfig.pokerAppId}`,
      }),
      cache: new InMemoryCache(),
      defaultOptions: {
        query: {
          fetchPolicy: 'network-only',
        },
      },
    });
  }
  
  // Rummy client
  if (lineraConfig.rummyAppId && chainId) {
    rummyClient = new ApolloClient({
      link: createHttpLink({
        uri: `${baseURL}/chains/${chainId}/applications/${lineraConfig.rummyAppId}`,
      }),
      cache: new InMemoryCache(),
      defaultOptions: {
        query: {
          fetchPolicy: 'network-only',
        },
      },
    });
  }
  
  // Roulette client
  if (lineraConfig.rouletteAppId && chainId) {
    rouletteClient = new ApolloClient({
      link: createHttpLink({
        uri: `${baseURL}/chains/${chainId}/applications/${lineraConfig.rouletteAppId}`,
      }),
      cache: new InMemoryCache(),
      defaultOptions: {
        query: {
          fetchPolicy: 'network-only',
        },
      },
    });
  }

  return pokerClient || rummyClient || rouletteClient;
}

export function getConfig(): LineraConfig {
  if (!config) {
    throw new Error('Linera client not initialized. Call initializeLineraClient first.');
  }
  return config;
}

export function getPokerClient(): ApolloClient<any> {
  if (!pokerClient) {
    throw new Error('Poker client not initialized. Make sure pokerAppId and defaultChain are set in config.json');
  }
  return pokerClient;
}

export function getRummyClient(): ApolloClient<any> {
  if (!rummyClient) {
    throw new Error('Rummy client not initialized. Make sure rummyAppId and defaultChain are set in config.json');
  }
  return rummyClient;
}

export function getRouletteClient(): ApolloClient<any> {
  if (!rouletteClient) {
    throw new Error('Roulette client not initialized. Make sure rouletteAppId and defaultChain are set in config.json');
  }
  return rouletteClient;
}
