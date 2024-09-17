// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "../src/MessageSequence.sol";
import "../src/Router.sol";

contract MessageSequenceTest is Test, MessageSequence {
    address addr = address(0x123);
    address addr2 = address(0x456);

    function setUp() public {}

    function test_initialValue() public view {
        assertEq(this.nextMessageSequence(addr), 0);
    }

    function test_incrementValue() public {
        assertEq(_useMessageSequence(addr), 0);
        assertEq(this.nextMessageSequence(addr), 1);
        assertEq(_useMessageSequence(addr), 1);
        assertEq(this.nextMessageSequence(addr), 2);
        assertEq(_useMessageSequence(addr2), 0);
        assertEq(this.nextMessageSequence(addr2), 1);
    }
}
