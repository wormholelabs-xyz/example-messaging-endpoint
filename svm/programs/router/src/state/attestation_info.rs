use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use universal_address::UniversalAddress;

use crate::error::RouterError;
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
    pub dst_addr: UniversalAddress,

    /// Payload hash (32 bytes)
    pub payload_hash: [u8; 32],

    /// Replay protection flag
    pub executed: bool,

    /// The bitmap of receive-enabled transceivers for this source chain that have attested to the message
    pub attested_transceivers: Bitmap,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AttestationInfoArgs {
    pub bump: u8,
    pub src_chain: u16,
    pub src_addr: UniversalAddress,
    pub sequence: u64,
    pub dst_chain: u16,
    pub dst_addr: UniversalAddress,
    pub payload_hash: [u8; 32],
    pub message_hash: [u8; 32],
}

impl AttestationInfo {
    /// Seed prefix for deriving AttestionInfo PDAs
    pub const SEED_PREFIX: &'static [u8] = b"attestation_info";

    pub fn new(args: AttestationInfoArgs) -> Result<Self> {
        let mut info = Self {
            bump: args.bump,
            src_chain: args.src_chain,
            src_addr: args.src_addr,
            sequence: args.sequence,
            dst_chain: args.dst_chain,
            dst_addr: args.dst_addr,
            payload_hash: args.payload_hash,
            message_hash: [0; 32],
            executed: false,
            attested_transceivers: Bitmap::new(),
        };

        let computed_hash = info.compute_own_message_hash();

        require!(
            computed_hash == args.message_hash,
            RouterError::InvalidMessageHash
        );

        info.message_hash = computed_hash;
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
        Pubkey::find_program_address(&[Self::SEED_PREFIX, &message_hash], &crate::ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_message_hash() {
        // Mock data
        let src_chain: u16 = 2;
        let src_addr = UniversalAddress::from_bytes([
            0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78,
            0x90, 0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56,
            0x78, 0x90, 0x12, 0x34,
        ]);
        let sequence: u64 = 42;
        let dst_chain: u16 = 1;
        let dst_addr = UniversalAddress::from_bytes([
            0x98, 0x76, 0x54, 0x32, 0x10, 0x98, 0x76, 0x54, 0x32, 0x10, 0x98, 0x76, 0x54, 0x32,
            0x10, 0x98, 0x76, 0x54, 0x32, 0x10, 0x98, 0x76, 0x54, 0x32, 0x10, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x98, 0x76,
        ]);
        let payload_hash: [u8; 32] = [
            0xaa, 0xbb, 0xcc, 0xdd, 0xaa, 0xbb, 0xcc, 0xdd, 0xaa, 0xbb, 0xcc, 0xdd, 0xaa, 0xbb,
            0xcc, 0xdd, 0xaa, 0xbb, 0xcc, 0xdd, 0xaa, 0xbb, 0xcc, 0xdd, 0xaa, 0xbb, 0xcc, 0xdd,
            0xaa, 0xbb, 0xcc, 0xdd,
        ];
        // Compute message hash
        let message_hash = AttestationInfo::compute_message_hash(
            src_chain,
            src_addr,
            sequence,
            dst_chain,
            dst_addr,
            payload_hash,
        );

        // Expected result from ethers.solidityPacked()
        let expected_hash: [u8; 32] = [
            0x2e, 0x02, 0x9b, 0x42, 0xd3, 0x2e, 0xe8, 0x76, 0x87, 0xda, 0xa5, 0xb1, 0xc8, 0x62,
            0x8f, 0xbb, 0xb0, 0xbf, 0xd3, 0x48, 0xed, 0x8c, 0xe8, 0x12, 0xf3, 0xc7, 0xbe, 0x63,
            0xcb, 0x9e, 0x31, 0xdf,
        ];

        assert_eq!(
            message_hash, expected_hash,
            "Computed hash does not match expected hash"
        );
    }
}
