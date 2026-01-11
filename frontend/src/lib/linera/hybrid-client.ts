/**
 * Hybrid Linera Client
 * 
 * This module provides a unified interface that works with both:
 * 1. Croissant browser extension (window.linera)
 * 2. Direct GraphQL connections (Apollo Client)
 */

import { ApolloClient, InMemoryCache, createHttpLink } from '@apollo/client';
import {
  isCroissantAvailable,
  queryWithCroissant,
  graphqlQueryWithCroissant,
  connectCroissantWallet,
} from './croissant';
import { LineraConfig } from './client';

export type ConnectionMode = 'croissant' | 'direct' | 'auto';

export interface HybridClientConfig extends LineraConfig {
  connectionMode?: ConnectionMode;
  preferCroissant?: boolean;
}

class HybridLineraClient {
  private config: HybridClientConfig;
  private apolloClients: {
    poker?: ApolloClient<any>;
    rummy?: ApolloClient<any>;
    roulette?: ApolloClient<any>;
  } = {};
  private connectionMode: ConnectionMode = 'auto';
  private usingCroissant: boolean = false;

  constructor(config: HybridClientConfig) {
    this.config = config;
    this.connectionMode = config.connectionMode || 'auto';
    this.initialize();
  }

  private async initialize() {
    // Determine connection mode
    if (this.connectionMode === 'auto') {
      this.usingCroissant = isCroissantAvailable() && (this.config.preferCroissant !== false);
    } else {
      this.usingCroissant = this.connectionMode === 'croissant';
    }

    if (this.usingCroissant) {
      console.log('Using Croissant extension for Linera connections');
      try {
        await connectCroissantWallet();
      } catch (error) {
        console.warn('Failed to connect to Croissant, falling back to direct connection:', error);
        this.usingCroissant = false;
        this.initializeApolloClients();
      }
    } else {
      console.log('Using direct GraphQL connection');
      this.initializeApolloClients();
    }
  }

  private initializeApolloClients() {
    const baseURL = this.config.nodeServiceURL || 'http://localhost:8081';
    const chainId = this.config.userChain1 || this.config.defaultChain;

    if (this.config.pokerAppId && chainId) {
      this.apolloClients.poker = new ApolloClient({
        link: createHttpLink({
          uri: `${baseURL}/chains/${chainId}/applications/${this.config.pokerAppId}`,
        }),
        cache: new InMemoryCache(),
        defaultOptions: {
          query: {
            fetchPolicy: 'network-only',
          },
        },
      });
    }

    if (this.config.rummyAppId && chainId) {
      this.apolloClients.rummy = new ApolloClient({
        link: createHttpLink({
          uri: `${baseURL}/chains/${chainId}/applications/${this.config.rummyAppId}`,
        }),
        cache: new InMemoryCache(),
        defaultOptions: {
          query: {
            fetchPolicy: 'network-only',
          },
        },
      });
    }

    if (this.config.rouletteAppId && chainId) {
      this.apolloClients.roulette = new ApolloClient({
        link: createHttpLink({
          uri: `${baseURL}/chains/${chainId}/applications/${this.config.rouletteAppId}`,
        }),
        cache: new InMemoryCache(),
        defaultOptions: {
          query: {
            fetchPolicy: 'network-only',
          },
        },
      });
    }
  }

  /**
   * Query application (works with both Croissant and direct connection)
   */
  async query(
    appType: 'poker' | 'rummy' | 'roulette',
    query: string,
    variables?: Record<string, any>
  ): Promise<any> {
    const appId = this.getAppId(appType);

    if (this.usingCroissant) {
      return graphqlQueryWithCroissant(appId, query, variables);
    } else {
      const client = this.getApolloClient(appType);
      const result = await client.query({
        query: require('graphql-tag')(query),
        variables,
        fetchPolicy: 'network-only',
      });
      return result.data;
    }
  }

  /**
   * Mutate application (works with both Croissant and direct connection)
   */
  async mutate(
    appType: 'poker' | 'rummy' | 'roulette',
    mutation: string,
    variables?: Record<string, any>
  ): Promise<any> {
    const appId = this.getAppId(appType);

    if (this.usingCroissant) {
      // For mutations, we still use query endpoint in Croissant
      return graphqlQueryWithCroissant(appId, mutation, variables);
    } else {
      const client = this.getApolloClient(appType);
      const result = await client.mutate({
        mutation: require('graphql-tag')(mutation),
        variables,
      });
      return result.data;
    }
  }

  /**
   * Get Apollo client for direct connection mode
   */
  getApolloClient(appType: 'poker' | 'rummy' | 'roulette'): ApolloClient<any> {
    const client = this.apolloClients[appType];
    if (!client) {
      throw new Error(`${appType} client not initialized`);
    }
    return client;
  }

  /**
   * Check if using Croissant
   */
  isUsingCroissant(): boolean {
    return this.usingCroissant;
  }

  /**
   * Get current connection mode
   */
  getConnectionMode(): 'croissant' | 'direct' {
    return this.usingCroissant ? 'croissant' : 'direct';
  }

  private getAppId(appType: 'poker' | 'rummy' | 'roulette'): string {
    switch (appType) {
      case 'poker':
        return this.config.pokerAppId || '';
      case 'rummy':
        return this.config.rummyAppId || '';
      case 'roulette':
        return this.config.rouletteAppId || '';
      default:
        throw new Error(`Unknown app type: ${appType}`);
    }
  }
}

// Singleton instance
let hybridClient: HybridLineraClient | null = null;

export function initializeHybridClient(config: HybridClientConfig): HybridLineraClient {
  hybridClient = new HybridLineraClient(config);
  return hybridClient;
}

export function getHybridClient(): HybridLineraClient {
  if (!hybridClient) {
    throw new Error('Hybrid client not initialized. Call initializeHybridClient first.');
  }
  return hybridClient;
}

