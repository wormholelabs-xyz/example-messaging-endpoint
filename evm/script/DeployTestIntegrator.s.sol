// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import {TestIntegrator} from "./TestIntegrator.s.sol";
import "forge-std/Script.sol";

// DeployTestIntegrator is a forge script to deploy the TestIntegrator contract. Use ./sh/deployTestIntegrator.sh to invoke this.
contract DeployTestIntegrator is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun(address router, uint16 chain, address transceiver) public {
        _deploy(router, chain, transceiver);
    }

    function run(address router, uint16 chain, address transceiver) public returns (address deployedAddress) {
        vm.startBroadcast();
        (deployedAddress) = _deploy(router, chain, transceiver);
        vm.stopBroadcast();
    }

    function _deploy(address router, uint16 chain, address transceiver) internal returns (address deployedAddress) {
        TestIntegrator testIntegrator = new TestIntegrator(router, chain, transceiver);

        return (address(testIntegrator));
    }
}
