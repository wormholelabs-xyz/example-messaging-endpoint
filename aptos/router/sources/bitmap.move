module router::bitmap {
    const MAX_U128: u128 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    const E_ALREADY_ENABLED: u64 = 0;
    const E_ALREADY_DISABLED: u64 = 1;

    public fun enable(bitmap: u128, index: u8): u128 {
        let bitmask = 1 << index;
        assert!(bitmap & bitmask == 0, E_ALREADY_ENABLED);
        bitmap | bitmask
    }

    public fun disable(bitmap: u128, index: u8): u128 {
        let bitmask = 1 << index;
        assert!(bitmap & bitmask > 0, E_ALREADY_DISABLED);
        bitmap & (bitmask ^ MAX_U128)
    }

    public fun get(bitmap: u128, index: u8): bool {
        let bitmask = 1 << index;
        bitmap & bitmask > 0
    }

    #[test]
    public fun enable_test() {
        assert!(enable(0, 0) == 0x00000000000000000000000000000001);
        assert!(enable(0, 127) == 0x80000000000000000000000000000000);
    }

    #[test]
    #[expected_failure(arithmetic_error, location = Self)]
    public fun enable_fails_with_overflow() {
        enable(0, 128);
    }

    #[test]
    #[expected_failure(abort_code = E_ALREADY_ENABLED, location = Self)]
    public fun enable_fails_with_already_enabled_bit() {
        enable(0x00000000000000000000000000000001, 0);
    }

    #[test]
    public fun disable_test() {
        assert!(disable(0x00000000000000000000000000000001, 0) == 0);
        assert!(disable(0x80000000000000000000000000000000, 127) == 0);
    }

    #[test]
    #[expected_failure(arithmetic_error, location = Self)]
    public fun disable_fails_with_overflow() {
        disable(0, 128);
    }

    #[test]
    #[expected_failure(abort_code = E_ALREADY_DISABLED, location = Self)]
    public fun disable_fails_with_already_disabled_bit() {
        disable(0, 0);
    }

    #[test]
    public fun get_test() {
        assert!(get(0, 0) == false);
        assert!(get(1, 0) == true);
        assert!(get(0x80000000000000000000000000000000, 0) == false);
        assert!(get(0x80000000000000000000000000000000, 127) == true);
    }

    #[test]
    #[expected_failure(arithmetic_error, location = Self)]
    public fun get_fails_with_overflow() {
        get(0, 128);
    }
}
