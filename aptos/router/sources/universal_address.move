/// 32 byte, left-padded address representing an arbitrary address, to be used to refer to addresses across chains.
/// from https://github.com/wormhole-foundation/wormhole/blob/a0dd60f8a0e4b9cfe26593c379d25eaf9f73c43c/aptos/wormhole/sources/external_address.move
module router::universal_address {
    use std::bcs;
    use std::vector;

    const E_VECTOR_TOO_LONG: u64 = 0;

    struct UniversalAddress has drop, copy, store {
        universal_address: vector<u8>,
    }

    public fun get_bytes(e: &UniversalAddress): vector<u8> {
        e.universal_address
    }

    public fun pad_left_32(input: &vector<u8>): vector<u8> {
        let len = vector::length<u8>(input);
        assert!(len <= 32, E_VECTOR_TOO_LONG);
        let ret = vector::empty<u8>();
        let zeros_remaining = 32 - len;
        while (zeros_remaining > 0) {
            vector::push_back<u8>(&mut ret, 0);
            zeros_remaining = zeros_remaining - 1;
        };
        vector::append<u8>(&mut ret, *input);
        ret
    }

    public fun left_pad(s: &vector<u8>): UniversalAddress {
        let padded_vector = pad_left_32(s);
        UniversalAddress { universal_address: padded_vector}
    }

    public fun from_bytes(bytes: vector<u8>): UniversalAddress {
        left_pad(&bytes)
    }

    public fun from_address(addr: address): UniversalAddress {
        from_bytes(bcs::to_bytes(&addr))
    }

}

#[test_only]
module router::universal_address_test {
    use router::universal_address::{get_bytes, left_pad, from_bytes, pad_left_32, from_address};
    use aptos_framework::vector::{Self};

    // test get_bytes and left_pad
    #[test]
    public fun test_left_pad() {
        let v = x"123456789123456789123456789123451234567891234567891234"; // less than 32 bytes
        let res = left_pad(&v);
        let bytes = get_bytes(&res);
        let m = x"0000000000";
        vector::append(&mut m, v);
        assert!(bytes==m, 0);
    }

    #[test]
    public fun test_left_pad_length_32_vector() {
        let v = x"1234567891234567891234567891234512345678912345678912345678912345"; //32 bytes
        let res = left_pad(&v);
        let bytes = get_bytes(&res);
        assert!(bytes==v, 0);
    }

    #[test]
    #[expected_failure(abort_code = 0, location = router::universal_address)]
    public fun test_left_pad_vector_too_long() {
        let v = x"123456789123456789123456789123451234567891234567891234567891234500"; //33 bytes
        let res = left_pad(&v);
        let bytes = get_bytes(&res);
        assert!(bytes==v, 0);
    }

    #[test]
    public fun test_from_bytes() {
        let v = x"1234";
        let ea = from_bytes(v);
        let bytes = get_bytes(&ea);
        let w = x"000000000000000000000000000000000000000000000000000000000000";
        vector::append(&mut w, v);
        assert!(bytes==w, 0);
    }

    #[test]
    #[expected_failure(abort_code = 0, location = router::universal_address)]
    public fun test_from_bytes_over_32_bytes() {
        let v = x"00000000000000000000000000000000000000000000000000000000000000001234";
        let ea = from_bytes(v);
        let _bytes = get_bytes(&ea);
    }

    #[test]
    fun test_pad_left_short() {
        let v = x"11";
        let pad_left_v = pad_left_32(&v);
        assert!(pad_left_v==x"0000000000000000000000000000000000000000000000000000000000000011", 0);
    }

    #[test]
    fun test_pad_left_exact() {
        let v = x"5555555555555555555555555555555555555555555555555555555555555555";
        let pad_left_v = pad_left_32(&v);
        assert!(pad_left_v==x"5555555555555555555555555555555555555555555555555555555555555555", 0);
    }

    #[test]
    #[expected_failure(abort_code = 0, location = router::universal_address)]
    fun test_pad_left_long() {
        let v = x"665555555555555555555555555555555555555555555555555555555555555555";
        pad_left_32(&v);
    }

    #[test(addr = @0x000000000000000000000000000000000000000000000000000000000000cafe)]
    public fun test_from_address(addr: address) {
        let v = x"000000000000000000000000000000000000000000000000000000000000cafe";
        let res = from_address(addr);
        let bytes = get_bytes(&res);
        assert!(bytes == v);
    }
}
