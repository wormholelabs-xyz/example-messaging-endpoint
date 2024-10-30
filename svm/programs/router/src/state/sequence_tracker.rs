use anchor_lang::prelude::*;

/// Tracks the sequence number for an integrator program
/// We could have put this in the `IntegratorConfig` account,
/// but due to the frequent writes to the `sequence` field, we
/// made it another account to prevent unnecessary account write locks.
/// This way we separate the concerns of Integrator config vs sequence
/// tracking better.
#[account]
#[derive(InitSpace)]
pub struct SequenceTracker {
    /// Bump seed for PDA derivation
    pub bump: u8,

    /// The program ID of the integrator
    /// This is used as a seed for PDA derivation
    pub integrator_program_id: Pubkey,

    /// The current sequence number for this integrator
    pub sequence: u64,
}

impl SequenceTracker {
    /// Seed prefix for deriving SequenceTracker PDAs
    pub const SEED_PREFIX: &'static [u8] = b"sequence_tracker";

    pub fn pda(integrator_program: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::SEED_PREFIX, integrator_program.as_ref()],
            &crate::ID,
        )
    }

    /// Increments and returns the next sequence number
    pub fn next_sequence(&mut self) -> u64 {
        let sequence = self.sequence;
        self.sequence = self.sequence.checked_add(1).unwrap();
        sequence
    }
}
