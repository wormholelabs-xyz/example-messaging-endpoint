// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import {Endpoint, endpointVersion} from "../src/Endpoint.sol";
import "forge-std/Script.sol";

// DeployEndpoint is a forge script to deploy the Endpoint contract. Use ./sh/deployEndpoint.sh to invoke this.
contract DeployEndpoint is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun(uint16 ourChain) public {
        _deploy(ourChain);
    }

    function run(uint16 ourChain) public returns (address deployedAddress) {
        vm.startBroadcast();
        (deployedAddress) = _deploy(ourChain);
        vm.stopBroadcast();
    }

    function _deploy(uint16 ourChain) internal returns (address deployedAddress) {
        bytes32 salt = keccak256(abi.encodePacked(endpointVersion));
        Endpoint endpoint = new Endpoint{salt: salt}(ourChain);

        return (address(endpoint));
    }
}
