use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use universal_address::UniversalAddress;

use crate::utils::bitmap::Bitmap;

#[derive(InitSpace, Debug)]
#[account]
pub struct OutboxMessage {
    /// The sending integrator as a 32-byte universal address
    pub src_addr: UniversalAddress,

    /// The sequence number of the message
    pub sequence: u64,

    /// The destination chain's Wormhole Chain ID
    pub dst_chain: u16,

    /// The destination address as a 32-byte universal address
    pub dst_addr: UniversalAddress,

    /// The keccak256 of an arbitrary payload (32 bytes)
    pub payload_hash: [u8; 32],

    /// The bitmap of send-enabled transceivers for this destination chain that have not picked up the message
    pub outstanding_transceivers: Bitmap,
}

impl OutboxMessage {
    pub fn compute_message_hash(
        src_chain: u16,
        src_addr: &UniversalAddress,
        sequence: u64,
        dst_chain: u16,
        dst_addr: &UniversalAddress,
        payload_hash: &[u8],
    ) -> [u8; 32] {
        // Ensure payload hash is correct length
        assert_eq!(payload_hash.len(), 32, "payload_hash must be 32 bytes");

        // Create buffer for concatenation
        let mut bytes = Vec::new();

        // Add source chain (big-endian)
        bytes.extend_from_slice(&src_chain.to_be_bytes());
        // Add source address
        bytes.extend_from_slice(&src_addr.to_bytes());
        // Add sequence (big-endian)
        bytes.extend_from_slice(&sequence.to_be_bytes());
        // Add destination chain (big-endian)
        bytes.extend_from_slice(&dst_chain.to_be_bytes());
        // Add destination address
        bytes.extend_from_slice(&dst_addr.to_bytes());
        // Add payload hash
        bytes.extend_from_slice(payload_hash);

        // Compute keccak256
        keccak::hash(&bytes).to_bytes()
    }
}
