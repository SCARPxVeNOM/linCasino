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
      throw new Error(`Failed to load config.json: ${response.status} ${response.statusText}`);
    }
    const config = await response.json();
    console.log('Config loaded successfully:', config);
    return config;
  } catch (error) {
    console.error('Error loading config:', error);
    // Fallback config for development
    const fallbackConfig = {
      nodeServiceURL: 'https://testnet-linera.lavenderfive.com',
      pokerAppId: '',
      rummyAppId: '',
      rouletteAppId: '',
      bankrollAppId: '',
      defaultChain: '',
    };
    console.warn('Using fallback config:', fallbackConfig);
    return fallbackConfig;
  }
}

async function init() {
  const rootElement = document.getElementById('root');
  if (!rootElement) {
    console.error('Root element not found!');
    return;
  }

  try {
    console.log('Initializing Linera Casino...');
    const config = await loadConfig();
    
    if (!config.pokerAppId && !config.rummyAppId && !config.rouletteAppId) {
      throw new Error('No application IDs found in config. Please check config.json');
    }

    const client = initializeLineraClient({
      nodeServiceURL: config.nodeServiceURL || 'https://testnet-linera.lavenderfive.com',
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

    if (!client) {
      throw new Error('Failed to initialize Apollo client. Check your config.json');
    }

    console.log('Rendering app...');
    ReactDOM.createRoot(rootElement).render(
      <React.StrictMode>
        <ApolloProvider client={client}>
          <App />
        </ApolloProvider>
      </React.StrictMode>
    );
  } catch (error) {
    console.error('Failed to initialize app:', error);
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    const stack = error instanceof Error ? error.stack : '';
    
    ReactDOM.createRoot(rootElement).render(
      <div style={{ 
        padding: '40px', 
        textAlign: 'center',
        fontFamily: 'system-ui, -apple-system, sans-serif',
        maxWidth: '800px',
        margin: '0 auto'
      }}>
        <h1 style={{ color: '#ef4444', marginBottom: '20px' }}>⚠️ Error Initializing App</h1>
        <p style={{ color: '#666', marginBottom: '10px' }}>{errorMessage}</p>
        {stack && (
          <details style={{ marginTop: '20px', textAlign: 'left' }}>
            <summary style={{ cursor: 'pointer', color: '#888' }}>Stack Trace</summary>
            <pre style={{ 
              background: '#f5f5f5', 
              padding: '10px', 
              borderRadius: '4px',
              overflow: 'auto',
              fontSize: '12px'
            }}>{stack}</pre>
          </details>
        )}
        <p style={{ marginTop: '20px', color: '#888', fontSize: '14px' }}>
          Check the browser console for more details.
        </p>
      </div>
    );
  }
}

init();
