pragma solidity ^0.8.19;

import {IRouterIntegrator} from "../src/interfaces/IRouterIntegrator.sol";
import {IRouterAdmin} from "../src/interfaces/IRouterAdmin.sol";
import "../src/libraries/UniversalAddress.sol";

// This is a test integrator to use with the router. It sets itself as the administrator.
contract TestIntegrator {
    function test() public {} // Exclude this from coverage report.

    address router;
    address transceiver;

    constructor(address _router, address _transceiver) {
        router = _router;
        transceiver = _transceiver;

        address integrator = address(this);
        IRouterIntegrator(router).register(integrator);
        IRouterAdmin(router).addTransceiver(integrator, transceiver);
        IRouterAdmin(router).enableSendTransceiver(integrator, 4, transceiver);
        IRouterAdmin(router).enableRecvTransceiver(integrator, 4, transceiver);
    }

    function sendMessage(address dstAddr) public payable {
        address refundAddress = address(this);
        bytes32 payloadHash = keccak256("hello, world");
        IRouterIntegrator(router).sendMessage(
            4, UniversalAddressLibrary.fromAddress(dstAddr), payloadHash, refundAddress
        );
    }
}
