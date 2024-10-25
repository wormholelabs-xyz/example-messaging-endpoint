// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import {Router, routerVersion} from "../src/Router.sol";
import "forge-std/Script.sol";

// DeployRouter is a forge script to deploy the Router contract. Use ./sh/deployRouter.sh to invoke this.
contract DeployRouter is Script {
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
        bytes32 salt = keccak256(abi.encodePacked(routerVersion));
        Router router = new Router{salt: salt}(ourChain);

        return (address(router));
    }
}
