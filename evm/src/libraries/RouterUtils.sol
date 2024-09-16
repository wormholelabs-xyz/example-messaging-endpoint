// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

library RouterUtils {
    //When a bytes32 field can't be converted into a 20 byte EVM address, because the 12 padding bytes
    //  are non-zero
    error NotAnEvmAddress(bytes32);

    function toWormholeFormat(address addr) internal pure returns (bytes32) {
        return bytes32(uint256(uint160(addr)));
    }

    function fromWormholeFormat(bytes32 whFormatAddress) internal pure returns (address) {
        if (uint256(whFormatAddress) >> 160 != 0) {
            revert NotAnEvmAddress(whFormatAddress);
        }
        return address(uint160(uint256(whFormatAddress)));
    }
}
