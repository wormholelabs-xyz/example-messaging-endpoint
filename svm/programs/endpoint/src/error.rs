use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq)]
pub enum EndpointError {
    #[msg("Caller is not authorized")]
    CallerNotAuthorized,

    #[msg("Bitmap index is out of bounds")]
    BitmapIndexOutOfBounds,

    #[msg("Maximum number of adapters reached")]
    MaxAdaptersReached,

    #[msg("Adapter was already enabled")]
    AdapterAlreadyEnabled,

    #[msg("Adapter was already disabled")]
    AdapterAlreadyDisabled,

    #[msg("An admin transfer is in progress")]
    AdminTransferInProgress,

    #[msg("No admin transfer is in progress")]
    NoAdminTransferInProgress,

    #[msg("Invalid Chain Id")]
    InvalidChainId,

    #[msg("No Adapters Enabled")]
    AdapterNotEnabled,

    #[msg("Duplicate Message Attestation")]
    DuplicateMessageAttestation,

    #[msg("Message has already been picked up")]
    MessageAlreadyPickedUp,

    #[msg("Message has already been executed")]
    AlreadyExecuted,

    #[msg("Unknown Message Attestation")]
    UnknownMessageAttestation,

    #[msg("Message Hash is invalid")]
    InvalidMessageHash,
}
