// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import {TestIntegrator} from "./TestIntegrator.s.sol";
import "forge-std/Script.sol";

// DeployTestIntegrator is a forge script to deploy the TestIntegrator contract. Use ./sh/deployTestIntegrator.sh to invoke this.
contract DeployTestIntegrator is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun(address router, address transceiver) public {
        _deploy(router, transceiver);
    }

    function run(address router, address transceiver) public returns (address deployedAddress) {
        vm.startBroadcast();
        (deployedAddress) = _deploy(router, transceiver);
        vm.stopBroadcast();
    }

    function _deploy(address router, address transceiver) internal returns (address deployedAddress) {
        TestIntegrator testIntegrator = new TestIntegrator(router, transceiver);

        return (address(testIntegrator));
    }
}
