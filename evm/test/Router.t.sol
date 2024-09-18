// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "../src/libraries/UniversalAddress.sol";
import {Router} from "../src/Router.sol";

contract RouterTest is Test {
    Router public router;

    address userA = address(0x123);
    address userB = address(0x456);
    bytes message = "hello, world";

    function setUp() public {
        router = new Router();
    }

    function test_sendMessageIncrementsSequence() public {
        assertEq(router.nextMessageSequence(userA), 0);
        // Send inital message from userA, going from unset to 1
        vm.startPrank(userA);
        router.sendMessage(1, UniversalAddressLibrary.fromAddress(userB), message);
        assertEq(router.nextMessageSequence(userA), 1);
        // Send additional message from userA, incrementing the existing sequence
        router.sendMessage(1, UniversalAddressLibrary.fromAddress(userB), message);
        assertEq(router.nextMessageSequence(userA), 2);
    }

    function testFuzz_sendMessage(address user) public {
        uint64 beforeSequence = router.nextMessageSequence(user);
        vm.startPrank(user);
        router.sendMessage(1, UniversalAddressLibrary.fromAddress(user), message);
        assertEq(router.nextMessageSequence(user), beforeSequence + 1);
    }
}
