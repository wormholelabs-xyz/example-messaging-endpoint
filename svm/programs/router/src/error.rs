use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq)]
pub enum RouterError {
    #[msg("The program is paused")]
    ProgramPaused,

    #[msg("Invalid integrator authority")]
    InvalidIntegratorAuthority,

    #[msg("Maximum transceivers reached")]
    MaxTransceiversReached,

    #[msg("Bitmap index is out of bounds")]
    BitmapIndexOutOfBounds,
}
