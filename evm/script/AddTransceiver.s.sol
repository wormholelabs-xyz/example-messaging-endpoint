// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import {Router} from "../src/Router.sol";
import "forge-std/Script.sol";

// AddTransceiver is a forge script to add a transceiver for a given chain for an integrator to the Router contract. Use ./sh/addTransceiver.sh to invoke this.
contract AddTransceiver is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun(address router, address integrator, address transceiver) public {
        _addTransceiver(router, integrator, transceiver);
    }

    function run(address router, address integrator, address transceiver) public {
        vm.startBroadcast();
        _addTransceiver(router, integrator, transceiver);
        vm.stopBroadcast();
    }

    function _addTransceiver(address router, address integrator, address transceiver) internal {
        Router routerContract = Router(router);
        routerContract.addTransceiver(integrator, transceiver);
    }
}
