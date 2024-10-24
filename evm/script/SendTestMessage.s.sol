// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.4;

import {TestIntegrator} from "./TestIntegrator.s.sol";
import "forge-std/Script.sol";

// SendTestMessage is a forge script to send a message using the TestIntegrator contract. Use ./sh/sendTestMessage.sh to invoke this.
contract SendTestMessage is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun(address integrator, address dstAddr) public {
        _sendTestMessage(integrator, dstAddr);
    }

    function run(address integrator, address dstAddr) public {
        vm.startBroadcast();
        _sendTestMessage(integrator, dstAddr);
        vm.stopBroadcast();
    }

    function _sendTestMessage(address integrator, address dstAddr) internal {
        TestIntegrator(integrator).sendMessage(dstAddr);
    }
}
