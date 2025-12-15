import React from 'react';
import ReactDOM from 'react-dom/client';
import { ApolloProvider } from '@apollo/client';
import { initializeLineraClient } from './lib/linera/client';
import App from './App';
import './index.css';

// Load configuration from config.json
async function loadConfig() {
  try {
    const response = await fetch('/config.json');
    if (!response.ok) {
      throw new Error('Failed to load config.json');
    }
    const config = await response.json();
    return config;
  } catch (error) {
    console.error('Error loading config:', error);
    // Fallback config for development
    return {
      nodeServiceURL: 'http://localhost:8081',
      pokerAppId: '',
      rummyAppId: '',
      rouletteAppId: '',
      bankrollAppId: '',
      defaultChain: '',
    };
  }
}

async function init() {
  try {
    const config = await loadConfig();
    const client = initializeLineraClient({
      nodeServiceURL: config.nodeServiceURL || 'http://localhost:8081',
      pokerAppId: config.pokerAppId || '',
      rummyAppId: config.rummyAppId || '',
      rouletteAppId: config.rouletteAppId || '',
      bankrollAppId: config.bankrollAppId || '',
      defaultChain: config.defaultChain || '',
      userChain1: config.userChain1,
      userChain2: config.userChain2,
      userChain3: config.userChain3,
      userChain4: config.userChain4,
      userChain5: config.userChain5,
      userChain6: config.userChain6,
      userChain7: config.userChain7,
      userChain8: config.userChain8,
    });

    ReactDOM.createRoot(document.getElementById('root')!).render(
      <React.StrictMode>
        <ApolloProvider client={client}>
          <App />
        </ApolloProvider>
      </React.StrictMode>
    );
  } catch (error) {
    console.error('Failed to initialize app:', error);
    ReactDOM.createRoot(document.getElementById('root')!).render(
      <div style={{ padding: '20px', textAlign: 'center' }}>
        <h1>Error Initializing App</h1>
        <p>{error instanceof Error ? error.message : 'Unknown error'}</p>
      </div>
    );
  }
}

init();
