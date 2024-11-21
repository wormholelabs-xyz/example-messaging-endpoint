use anchor_lang::prelude::*;
use universal_address::UniversalAddress;

use crate::utils::bitmap::Bitmap;

#[derive(InitSpace, Debug)]
#[account]
pub struct OutboxMessage {
    /// The sending integrator as a 32-byte universal address
    pub src_addr: [u8; 32],

    /// The sequence number of the message
    pub sequence: u64,

    /// The destination chain's Wormhole Chain ID
    pub dst_chain: u16,

    /// The destination address as a 32-byte universal address
    pub dst_addr: [u8; 32],

    /// The keccak256 of an arbitrary payload (32 bytes)
    pub payload_hash: [u8; 32],

    /// The bitmap of send-enabled adapters for this destination chain that have not picked up the message
    pub outstanding_adapters: Bitmap,

    /// The recipient of the lamports when this account is closed
    pub refund_recipient: Pubkey,
}
