// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import {TestIntegrator} from "./TestIntegrator.s.sol";
import "forge-std/Script.sol";
import "../src/libraries/UniversalAddress.sol";

// ReceiveTestMessage is a forge script to receive a message using the TestIntegrator contract. Use ./sh/receiveTestMessage.sh to invoke this.
contract ReceiveTestMessage is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun(address integrator, uint16 srcChain, bytes32 srcAddr, uint64 sequence, bytes32 payloadHash)
        public
    {
        _receiveTestMessage(integrator, srcChain, srcAddr, sequence, payloadHash);
    }

    function run(address integrator, uint16 srcChain, bytes32 srcAddr, uint64 sequence, bytes32 payloadHash) public {
        vm.startBroadcast();
        _receiveTestMessage(integrator, srcChain, srcAddr, sequence, payloadHash);
        vm.stopBroadcast();
    }

    function _receiveTestMessage(
        address integrator,
        uint16 srcChain,
        bytes32 srcAddr,
        uint64 sequence,
        bytes32 payloadHash
    ) internal {
        TestIntegrator(integrator).recvMessage(
            srcChain, UniversalAddressLibrary.fromBytes32(srcAddr), sequence, payloadHash
        );
    }
}
