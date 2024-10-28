use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use universal_address::UniversalAddress;

use crate::utils::bitmap::Bitmap;

#[account]
#[derive(InitSpace)]
pub struct AttestationInfo {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// Message hash (32 bytes)
    /// Used as a seed for PDA derivation
    pub message_hash: [u8; 32],

    /// Source chain ID
    pub src_chain: u16,

    /// Source address (32 bytes)
    pub src_addr: UniversalAddress,

    /// Sequence number
    pub sequence: u64,

    /// Destination chain ID
    pub dst_chain: u16,

    /// Destination address (32 bytes)
    #[max_len(32)]
    pub dst_addr: UniversalAddress,

    /// Payload hash (32 bytes)
    #[max_len(32)]
    pub payload_hash: [u8; 32],

    /// Replay protection flag
    pub executed: bool,

    /// The bitmap of receive-enabled transceivers for this source chain that have attested to the message
    pub attested_transceivers: Bitmap,
}

impl AttestationInfo {
    /// Seed prefix for deriving AttestionInfo PDAs
    pub const SEED_PREFIX: &'static [u8] = b"attestation_info";

    pub fn new(
        bump: u8,
        src_chain: u16,
        src_addr: UniversalAddress,
        sequence: u64,
        dst_chain: u16,
        dst_addr: UniversalAddress,
        payload_hash: [u8; 32],
    ) -> Result<Self> {
        let mut info = Self {
            bump,
            src_chain,
            src_addr,
            sequence,
            dst_chain,
            dst_addr,
            payload_hash,
            message_hash: [0; 32],
            executed: false,
            attested_transceivers: Bitmap::new(),
        };
        info.message_hash = info.compute_own_message_hash();
        Ok(info)
    }

    pub fn compute_own_message_hash(&self) -> [u8; 32] {
        Self::compute_message_hash(
            self.src_chain,
            self.src_addr,
            self.sequence,
            self.dst_chain,
            self.dst_addr,
            self.payload_hash,
        )
    }

    pub fn compute_message_hash(
        src_chain: u16,
        src_addr: UniversalAddress,
        sequence: u64,
        dst_chain: u16,
        dst_addr: UniversalAddress,
        payload_hash: [u8; 32],
    ) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&src_chain.to_be_bytes());
        bytes.extend_from_slice(&src_addr.to_bytes());
        bytes.extend_from_slice(&sequence.to_be_bytes());
        bytes.extend_from_slice(&dst_chain.to_be_bytes());
        bytes.extend_from_slice(&dst_addr.to_bytes());
        bytes.extend_from_slice(&payload_hash);

        keccak::hash(&bytes).to_bytes()
    }

    pub fn pda(message_hash: [u8; 32]) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::SEED_PREFIX, &message_hash[..]], &crate::ID)
    }
}
