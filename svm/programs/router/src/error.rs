use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq)]
pub enum RouterError {
    #[msg("Invalid integrator authority")]
    InvalidIntegratorAuthority,

    #[msg("Bitmap index is out of bounds")]
    BitmapIndexOutOfBounds,
}
