pragma solidity ^0.8.19;

import {IRouterIntegrator} from "../src/interfaces/IRouterIntegrator.sol";
import {IRouterAdmin} from "../src/interfaces/IRouterAdmin.sol";
import "../src/libraries/UniversalAddress.sol";

// This is a test integrator to use with the router. It sets itself as the administrator.
contract TestIntegrator {
    function test() public {} // Exclude this from coverage report.

    address router;
    address transceiver;
    uint16 chain;

    constructor(address _router, uint16 _chain, address _transceiver) {
        router = _router;
        transceiver = _transceiver;
        chain = _chain;

        address integrator = address(this);
        IRouterIntegrator(router).register(integrator);
        IRouterAdmin(router).addTransceiver(integrator, transceiver);
        IRouterAdmin(router).enableSendTransceiver(integrator, chain, transceiver);
        IRouterAdmin(router).enableRecvTransceiver(integrator, chain, transceiver);
    }

    function sendMessage(address dstAddr) public payable {
        address refundAddress = address(this);
        bytes32 payloadHash = keccak256("hello, world");
        IRouterIntegrator(router).sendMessage(
            chain, UniversalAddressLibrary.fromAddress(dstAddr), payloadHash, refundAddress
        );
    }

    function recvMessage(uint16 srcChain, UniversalAddress srcAddr, uint64 sequence, bytes32 payloadHash) public {
        IRouterIntegrator(router).recvMessage(srcChain, srcAddr, sequence, payloadHash);
    }
}
