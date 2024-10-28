use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq)]
pub enum RouterError {
    #[msg("Caller is not authorized")]
    CallerNotAuthorized,

    #[msg("Bitmap index is out of bounds")]
    BitmapIndexOutOfBounds,

    #[msg("Maximum number of transceivers reached")]
    MaxTransceiversReached,

    #[msg("Transceiver was already enabled")]
    TransceiverAlreadyEnabled,

    #[msg("Transceiver was already disabled")]
    TransceiverAlreadyDisabled,

    #[msg("An admin transfer is in progress")]
    AdminTransferInProgress,

    #[msg("No admin transfer is in progress")]
    NoAdminTransferInProgress,

    #[msg("Invalid Chain Id")]
    InvalidChainId,

    #[msg("No Transceivers Enabled")]
    TransceiverNotEnabled,

    #[msg("Duplicate Message Attestation")]
    DuplicateMessageAttestation,

    #[msg("Message has already been picked up")]
    MessageAlreadyPickedUp,
}
