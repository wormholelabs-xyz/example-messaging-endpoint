use anchor_lang::prelude::*;

/// UniversalAddress represents a 32-byte address that can be used across different chains
#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq, InitSpace,
)]
pub struct UniversalAddress {
    /// The raw 32-byte address
    pub bytes: [u8; 32],
}

impl UniversalAddress {
    /// Creates a UniversalAddress from a Solana Pubkey
    pub fn from_pubkey(pubkey: &Pubkey) -> Self {
        Self {
            bytes: pubkey.to_bytes(),
        }
    }

    /// Converts the UniversalAddress back to a Solana Pubkey
    pub fn to_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.bytes)
    }

    /// Creates a UniversalAddress from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Returns the raw bytes of the UniversalAddress
    pub fn to_bytes(&self) -> [u8; 32] {
        self.bytes
    }
}

impl From<Pubkey> for UniversalAddress {
    fn from(pubkey: Pubkey) -> Self {
        Self::from_pubkey(&pubkey)
    }
}

impl From<UniversalAddress> for Pubkey {
    fn from(addr: UniversalAddress) -> Self {
        addr.to_pubkey()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pubkey_roundtrip() {
        let original_pubkey = Pubkey::new_unique();
        let universal = UniversalAddress::from_pubkey(&original_pubkey);
        let recovered_pubkey = universal.to_pubkey();
        assert_eq!(original_pubkey, recovered_pubkey);
    }

    #[test]
    fn test_bytes_roundtrip() {
        let original_bytes = [42u8; 32];
        let universal = UniversalAddress::from_bytes(original_bytes);
        let recovered_bytes = universal.to_bytes();
        assert_eq!(original_bytes, recovered_bytes);
    }

    #[test]
    fn test_from_into_traits() {
        let original_pubkey = Pubkey::new_unique();

        // Test From<Pubkey>
        let universal: UniversalAddress = original_pubkey.into();
        assert_eq!(universal.bytes, original_pubkey.to_bytes());

        // Test Into<Pubkey>
        let recovered_pubkey: Pubkey = universal.into();
        assert_eq!(original_pubkey, recovered_pubkey);
    }

    #[test]
    fn test_default() {
        let universal = UniversalAddress::default();
        assert_eq!(universal.bytes, [0u8; 32]);
    }

    #[test]
    fn test_debug_format() {
        let bytes = [1u8; 32];
        let universal = UniversalAddress::from_bytes(bytes);
        let debug_str = format!("{:?}", universal);
        assert!(debug_str.contains("UniversalAddress"));
    }

    #[test]
    fn test_clone_and_copy() {
        let original = UniversalAddress::from_bytes([1u8; 32]);

        // Test Clone
        let cloned = original.clone();
        assert_eq!(original.bytes, cloned.bytes);

        // Test Copy
        let copied = original;
        assert_eq!(original.bytes, copied.bytes);
    }

    #[test]
    fn test_equality() {
        let addr1 = UniversalAddress::from_bytes([1u8; 32]);
        let addr2 = UniversalAddress::from_bytes([1u8; 32]);
        let addr3 = UniversalAddress::from_bytes([2u8; 32]);

        assert_eq!(addr1, addr2);
        assert_ne!(addr1, addr3);
    }

    #[test]
    fn test_serialization() {
        let original = UniversalAddress::from_bytes([1u8; 32]);

        // Test serialization
        let serialized = original.try_to_vec().unwrap();

        // Test deserialization
        let deserialized: UniversalAddress = UniversalAddress::try_from_slice(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}
