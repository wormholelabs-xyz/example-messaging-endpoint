// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import {Endpoint} from "../src/Endpoint.sol";
import "forge-std/Script.sol";

// AddAdapter is a forge script to add an adapter for a given chain for an integrator to the Endpoint contract. Use ./sh/addAdapter.sh to invoke this.
contract AddAdapter is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun(address endpoint, address integrator, address adapter) public {
        _addAdapter(endpoint, integrator, adapter);
    }

    function run(address endpoint, address integrator, address adapter) public {
        vm.startBroadcast();
        _addAdapter(endpoint, integrator, adapter);
        vm.stopBroadcast();
    }

    function _addAdapter(address endpoint, address integrator, address adapter) internal {
        Endpoint endpointContract = Endpoint(endpoint);
        endpointContract.addAdapter(integrator, adapter);
    }
}
