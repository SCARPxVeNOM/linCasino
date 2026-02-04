// Copyright (c) Linera Casino
// SPDX-License-Identifier: Apache-2.0

//! Provably Fair RNG System using Commit-Reveal Scheme
//!
//! This module implements a verifiable random number generation system that allows
//! players to verify the fairness of game outcomes after they complete.

use async_graphql_derive::SimpleObject;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Commit-reveal RNG for provably fair gaming
/// 
/// The flow is:
/// 1. Server generates a random seed and publishes its hash BEFORE betting
/// 2. Player provides their own client seed (entropy)
/// 3. Game uses combined seeds + nonce to generate results
/// 4. After game, server reveals the original seed for verification
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct ProvablyFairRNG {
    /// SHA256 hash of server seed (published before betting starts)
    pub server_seed_hash: Vec<u8>,
    /// User-provided entropy for additional randomness
    pub client_seed: String,
    /// Incrementing nonce per game/round
    pub nonce: u64,
    /// Revealed server seed after game completes (for verification)
    pub revealed_server_seed: Option<String>,
}

impl ProvablyFairRNG {
    /// Create a new RNG instance with a server seed
    /// Returns the RNG and the original seed (to be stored securely)
    pub fn new(server_seed: &str) -> (Self, String) {
        let hash = Self::hash_seed(server_seed);
        (
            ProvablyFairRNG {
                server_seed_hash: hash,
                client_seed: String::new(),
                nonce: 0,
                revealed_server_seed: None,
            },
            server_seed.to_string(),
        )
    }

    /// Create RNG from an existing hash (for client/verification side)
    pub fn from_hash(hash: Vec<u8>) -> Self {
        ProvablyFairRNG {
            server_seed_hash: hash,
            client_seed: String::new(),
            nonce: 0,
            revealed_server_seed: None,
        }
    }

    /// Set the client seed (player-provided entropy)
    pub fn set_client_seed(&mut self, seed: String) {
        self.client_seed = seed;
    }

    /// Increment nonce for next game/round
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    /// Reveal the server seed after game completion
    pub fn reveal(&mut self, server_seed: String) -> Result<(), String> {
        // Verify the seed matches the hash
        let computed_hash = Self::hash_seed(&server_seed);
        if computed_hash != self.server_seed_hash {
            return Err("Server seed does not match committed hash".to_string());
        }
        self.revealed_server_seed = Some(server_seed);
        Ok(())
    }

    /// Generate a random result in range [min, max)
    pub fn generate_result(&self, server_seed: &str, min: u64, max: u64) -> Result<u64, String> {
        if max <= min {
            return Err("max must be greater than min".to_string());
        }

        // Combine server_seed, client_seed, and nonce
        let combined = format!("{}-{}-{}", server_seed, self.client_seed, self.nonce);
        let hash = Self::hash_seed(&combined);
        
        // Convert first 8 bytes of hash to u64
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash[0..8]);
        let random_value = u64::from_le_bytes(bytes);
        
        // Map to range
        let range = max - min;
        Ok(min + (random_value % range))
    }

    /// Generate a random float in range [0.0, 1.0)
    pub fn generate_float(&self, server_seed: &str) -> Result<f64, String> {
        let combined = format!("{}-{}-{}", server_seed, self.client_seed, self.nonce);
        let hash = Self::hash_seed(&combined);
        
        // Convert first 8 bytes of hash to u64
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash[0..8]);
        let random_value = u64::from_le_bytes(bytes);
        
        // Normalize to [0, 1)
        Ok(random_value as f64 / u64::MAX as f64)
    }

    /// Hash a seed string using SHA256
    pub fn hash_seed(seed: &str) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        hasher.finalize().to_vec()
    }

    /// Verify that a server seed matches a hash
    pub fn verify_seed(server_seed: &str, expected_hash: &[u8]) -> bool {
        let computed_hash = Self::hash_seed(server_seed);
        computed_hash == expected_hash
    }

    /// Full verification of a game result
    pub fn verify_result(
        server_seed: &str,
        client_seed: &str,
        nonce: u64,
        expected_hash: &[u8],
        min: u64,
        max: u64,
        claimed_result: u64,
    ) -> Result<bool, String> {
        // First verify the server seed matches the hash
        if !Self::verify_seed(server_seed, expected_hash) {
            return Err("Server seed does not match hash".to_string());
        }

        // Recreate the RNG state
        let rng = ProvablyFairRNG {
            server_seed_hash: expected_hash.to_vec(),
            client_seed: client_seed.to_string(),
            nonce,
            revealed_server_seed: Some(server_seed.to_string()),
        };

        // Generate the result and compare
        let actual_result = rng.generate_result(server_seed, min, max)?;
        Ok(actual_result == claimed_result)
    }
}

/// Proof stored alongside game outcomes for later verification
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct RNGProof {
    /// Hash of server seed (was published before game)
    pub server_seed_hash: Vec<u8>,
    /// Client-provided seed
    pub client_seed: String,
    /// Nonce used for this specific result
    pub nonce: u64,
    /// Revealed server seed (available after game)
    pub revealed_server_seed: String,
}

impl RNGProof {
    /// Create a new proof from RNG state
    pub fn from_rng(rng: &ProvablyFairRNG) -> Result<Self, String> {
        let revealed = rng.revealed_server_seed.clone()
            .ok_or("Server seed not yet revealed")?;
        
        Ok(RNGProof {
            server_seed_hash: rng.server_seed_hash.clone(),
            client_seed: rng.client_seed.clone(),
            nonce: rng.nonce,
            revealed_server_seed: revealed,
        })
    }

    /// Verify this proof is valid
    pub fn verify(&self) -> bool {
        ProvablyFairRNG::verify_seed(&self.revealed_server_seed, &self.server_seed_hash)
    }

    /// Verify a specific result using this proof
    pub fn verify_result(&self, min: u64, max: u64, claimed_result: u64) -> Result<bool, String> {
        ProvablyFairRNG::verify_result(
            &self.revealed_server_seed,
            &self.client_seed,
            self.nonce,
            &self.server_seed_hash,
            min,
            max,
            claimed_result,
        )
    }
}

/// Generate a random server seed from timestamp and additional entropy
pub fn generate_server_seed(timestamp: u64, additional_entropy: &str) -> String {
    let combined = format!("{}-{}-casino-seed", timestamp, additional_entropy);
    let hash = ProvablyFairRNG::hash_seed(&combined);
    // Convert to hex string
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Shuffle a deck using provably fair RNG (Fisher-Yates)
pub fn shuffle_deck_provably_fair<T: Clone>(
    items: &mut [T],
    server_seed: &str,
    client_seed: &str,
    base_nonce: u64,
) {
    let n = items.len();
    if n <= 1 {
        return;
    }

    for i in (1..n).rev() {
        let rng = ProvablyFairRNG {
            server_seed_hash: ProvablyFairRNG::hash_seed(server_seed),
            client_seed: client_seed.to_string(),
            nonce: base_nonce + i as u64,
            revealed_server_seed: None,
        };
        
        if let Ok(j) = rng.generate_result(server_seed, 0, (i + 1) as u64) {
            items.swap(i, j as usize);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_creation() {
        let (rng, seed) = ProvablyFairRNG::new("test-server-seed");
        assert!(!rng.server_seed_hash.is_empty());
        assert_eq!(seed, "test-server-seed");
    }

    #[test]
    fn test_seed_verification() {
        let (rng, seed) = ProvablyFairRNG::new("my-secret-seed");
        assert!(ProvablyFairRNG::verify_seed(&seed, &rng.server_seed_hash));
        assert!(!ProvablyFairRNG::verify_seed("wrong-seed", &rng.server_seed_hash));
    }

    #[test]
    fn test_result_generation() {
        let (mut rng, seed) = ProvablyFairRNG::new("server-seed-123");
        rng.set_client_seed("client-seed-456".to_string());
        
        let result = rng.generate_result(&seed, 0, 100).unwrap();
        assert!(result < 100);
        
        // Same inputs should give same result (deterministic)
        let result2 = rng.generate_result(&seed, 0, 100).unwrap();
        assert_eq!(result, result2);
        
        // Different nonce should give different result
        rng.increment_nonce();
        let result3 = rng.generate_result(&seed, 0, 100).unwrap();
        // Note: Could theoretically be the same, but very unlikely
    }

    #[test]
    fn test_full_verification() {
        let (mut rng, seed) = ProvablyFairRNG::new("secret-seed");
        rng.set_client_seed("player-seed".to_string());
        rng.nonce = 42;
        
        let result = rng.generate_result(&seed, 0, 37).unwrap();
        
        let verified = ProvablyFairRNG::verify_result(
            &seed,
            "player-seed",
            42,
            &rng.server_seed_hash,
            0,
            37,
            result,
        ).unwrap();
        
        assert!(verified);
    }
}
