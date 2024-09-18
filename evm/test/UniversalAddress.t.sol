// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "../src/libraries/UniversalAddress.sol";

contract UtilsTest is Test {
    address addr = address(0x123);
    bytes32 bytes32Address = bytes32(uint256(0x123));
    bytes32 invalidBytes32Address = bytes32(hex"000000000000000000000001");

    function setUp() public {}

    function test_fromAndToBytes32() public view {
        assertEq((UniversalAddressLibrary.fromBytes32(bytes32Address)).toBytes32(), bytes32Address);
    }

    function test_fromAndToAddress() public view {
        assertEq((UniversalAddressLibrary.fromAddress(addr)).toAddress(), addr);
    }

    function test_equals() public view {
        assert(UniversalAddressLibrary.fromAddress(addr) == UniversalAddressLibrary.fromBytes32(bytes32Address));
        assert(UniversalAddressLibrary.fromAddress(addr) != UniversalAddressLibrary.fromAddress(address(0x456)));
    }

    function test_UniversalAddressInvalid() public {
        UniversalAddress invalid = UniversalAddressLibrary.fromBytes32(invalidBytes32Address);
        vm.expectRevert(abi.encodeWithSelector(UniversalAddressLibrary.NotAnEvmAddress.selector, invalidBytes32Address));
        invalid.toAddress();
    }

    function testFuzz_address(bytes32 _addr) public {
        if (uint256(_addr) >> 160 != 0) {
            vm.expectRevert(abi.encodeWithSelector(UniversalAddressLibrary.NotAnEvmAddress.selector, _addr));
        }
        address addrFromBytes32 = UniversalAddressLibrary.fromBytes32(_addr).toAddress();
        assertEq(UniversalAddressLibrary.fromAddress(addrFromBytes32).toBytes32(), _addr);
    }

    function testFuzz_toAndFromAddress(address _addr) public pure {
        assert(UniversalAddressLibrary.fromAddress(_addr).toAddress() == _addr);
    }

    function testFuzz_toAndFromBytes32(bytes32 _addr) public pure {
        assert(UniversalAddressLibrary.fromBytes32(_addr).toBytes32() == _addr);
    }
}
