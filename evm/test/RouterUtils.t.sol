// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "../src/libraries/RouterUtils.sol";

contract UtilsTest is Test {
    address addr = address(0x123);
    bytes32 whFormatAddress = bytes32(uint256(0x123));
    bytes32 invalidWhFormatAddress = bytes32(hex"000000000000000000000001");

    function setUp() public {}

    function test_fromWormholeFormat() public view {
        assertEq(RouterUtils.fromWormholeFormat(whFormatAddress), addr);
    }

    function test_fromWormholeFormatInvalid() public {
        vm.expectRevert(abi.encodeWithSelector(RouterUtils.NotAnEvmAddress.selector, invalidWhFormatAddress));
        RouterUtils.fromWormholeFormat(invalidWhFormatAddress);
    }

    function test_toWormholeFormat() public view {
        assertEq(RouterUtils.toWormholeFormat(addr), whFormatAddress);
    }

    function testFuzz_fromAndTo(bytes32 _whFormatAddress) public {
        if (uint256(_whFormatAddress) >> 160 != 0) {
            vm.expectRevert(abi.encodeWithSelector(RouterUtils.NotAnEvmAddress.selector, _whFormatAddress));
        }
        assertEq(RouterUtils.toWormholeFormat(RouterUtils.fromWormholeFormat(_whFormatAddress)), _whFormatAddress);
    }

    function testFuzz_toAndFrom(address _addr) public pure {
        assertEq(RouterUtils.fromWormholeFormat(RouterUtils.toWormholeFormat(_addr)), _addr);
    }
}
