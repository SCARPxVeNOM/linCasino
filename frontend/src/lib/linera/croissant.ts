/**
 * Croissant Extension Integration
 * 
 * This module provides integration with the Croissant browser extension
 * which exposes window.linera API for interacting with Linera blockchain.
 */

// Type definitions for Croissant extension
declare global {
  interface Window {
    linera?: LineraProvider;
  }
}

interface LineraProvider extends EventTarget {
  request(request: WalletRequest): Promise<WalletResponse>;
  on(event: 'notification', callback: (data: any) => void): void;
  off(event: 'notification', callback: (data: any) => void): void;
}

interface QueryApplicationRequest {
  type: 'QUERY';
  applicationId: string;
  query: string;
}

interface AssignmentRequest {
  type: 'ASSIGNMENT';
  chainId: string;
  timestamp: string;
}

type WalletRequest = QueryApplicationRequest | AssignmentRequest;

interface WalletResponse {
  id: string;
  result?: any;
  error?: string;
}

/**
 * Check if Croissant extension is available
 */
export function isCroissantAvailable(): boolean {
  return typeof window !== 'undefined' && !!window.linera;
}

/**
 * Connect to Croissant wallet
 */
export async function connectCroissantWallet(): Promise<boolean> {
  if (!isCroissantAvailable()) {
    throw new Error('Croissant extension not found. Please install the Croissant browser extension.');
  }

  try {
    // Request wallet connection
    const response = await window.linera!.request({
      type: 'ASSIGNMENT',
      chainId: '', // Will be set by extension
      timestamp: new Date().toISOString(),
    });

    return !!response.result;
  } catch (error) {
    console.error('Failed to connect to Croissant wallet:', error);
    throw error;
  }
}

/**
 * Query application using Croissant
 */
export async function queryWithCroissant(
  applicationId: string,
  query: string
): Promise<any> {
  if (!isCroissantAvailable()) {
    throw new Error('Croissant extension not available');
  }

  try {
    const response = await window.linera!.request({
      type: 'QUERY',
      applicationId,
      query,
    });

    if (response.error) {
      throw new Error(response.error);
    }

    return response.result;
  } catch (error) {
    console.error('Croissant query failed:', error);
    throw error;
  }
}

/**
 * GraphQL query wrapper for Croissant
 */
export async function graphqlQueryWithCroissant(
  applicationId: string,
  query: string,
  variables?: Record<string, any>
): Promise<any> {
  const graphqlQuery = JSON.stringify({
    query,
    variables: variables || {},
  });

  const result = await queryWithCroissant(applicationId, graphqlQuery);
  
  // Parse the result if it's a string
  if (typeof result === 'string') {
    try {
      return JSON.parse(result);
    } catch {
      return result;
    }
  }

  return result;
}

/**
 * Listen to Croissant notifications
 */
export function onCroissantNotification(
  callback: (data: any) => void
): () => void {
  if (!isCroissantAvailable()) {
    return () => {};
  }

  window.linera!.on('notification', callback);

  // Return unsubscribe function
  return () => {
    window.linera!.off('notification', callback);
  };
}





