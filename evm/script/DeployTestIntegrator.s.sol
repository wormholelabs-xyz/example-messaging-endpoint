// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import {TestIntegrator} from "./TestIntegrator.s.sol";
import "forge-std/Script.sol";

// DeployTestIntegrator is a forge script to deploy the TestIntegrator contract. Use ./sh/deployTestIntegrator.sh to invoke this.
contract DeployTestIntegrator is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun(address endpoint, uint16 chain, address adapter) public {
        _deploy(endpoint, chain, adapter);
    }

    function run(address endpoint, uint16 chain, address adapter) public returns (address deployedAddress) {
        vm.startBroadcast();
        (deployedAddress) = _deploy(endpoint, chain, adapter);
        vm.stopBroadcast();
    }

    function _deploy(address endpoint, uint16 chain, address adapter) internal returns (address deployedAddress) {
        TestIntegrator testIntegrator = new TestIntegrator(endpoint, chain, adapter);

        return (address(testIntegrator));
    }
}
