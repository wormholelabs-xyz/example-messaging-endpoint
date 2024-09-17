// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

type UniversalAddress is bytes32;

library UniversalAddressLibrary {
    // When a bytes32 field can't be converted into a 20 byte EVM address, because the 12 padding bytes
    // are non-zero
    error NotAnEvmAddress(bytes32);

    function fromAddress(address addr) internal pure returns (UniversalAddress) {
        return fromBytes32(bytes32(uint256(uint160(addr))));
    }

    function toAddress(UniversalAddress uAddr) internal pure returns (address) {
        bytes32 internalAddr = toBytes32(uAddr);
        // Check if the higher 96 bits (left-most 12 bytes) are non-zero
        if (uint256(internalAddr) >> 160 != 0) {
            revert NotAnEvmAddress(internalAddr);
        }
        return address(uint160(uint256(internalAddr)));
    }

    function fromBytes32(bytes32 addr) internal pure returns (UniversalAddress) {
        return UniversalAddress.wrap(addr);
    }

    function toBytes32(UniversalAddress addr) internal pure returns (bytes32) {
        return UniversalAddress.unwrap(addr);
    }
}

using UniversalAddressLibrary for UniversalAddress global;

function equals(UniversalAddress addr1, UniversalAddress addr2) pure returns (bool) {
    return UniversalAddressLibrary.toBytes32(addr1) == UniversalAddressLibrary.toBytes32(addr2);
}

function notEquals(UniversalAddress addr1, UniversalAddress addr2) pure returns (bool) {
    return UniversalAddressLibrary.toBytes32(addr1) != UniversalAddressLibrary.toBytes32(addr2);
}

using {equals as ==, notEquals as !=} for UniversalAddress global;
