pragma solidity ^0.8.19;

import {IEndpointIntegrator} from "../src/interfaces/IEndpointIntegrator.sol";
import {IEndpointAdmin} from "../src/interfaces/IEndpointAdmin.sol";
import "../src/libraries/UniversalAddress.sol";

// This is a test integrator to use with the endpoint. It sets itself as the administrator.
contract TestIntegrator {
    function test() public {} // Exclude this from coverage report.

    address endpoint;
    address adapter;
    uint16 chain;

    constructor(address _endpoint, uint16 _chain, address _adapter) {
        endpoint = _endpoint;
        adapter = _adapter;
        chain = _chain;

        address integrator = address(this);
        IEndpointIntegrator(endpoint).register(integrator);
        IEndpointAdmin(endpoint).addAdapter(integrator, adapter);
        IEndpointAdmin(endpoint).enableSendAdapter(integrator, chain, adapter);
        IEndpointAdmin(endpoint).enableRecvAdapter(integrator, chain, adapter);
    }

    function sendMessage(address dstAddr) public payable {
        address refundAddress = address(this);
        bytes32 payloadHash = keccak256("hello, world");
        IEndpointIntegrator(endpoint).sendMessage(
            chain, UniversalAddressLibrary.fromAddress(dstAddr), payloadHash, refundAddress
        );
    }

    function recvMessage(uint16 srcChain, UniversalAddress srcAddr, uint64 sequence, bytes32 payloadHash) public {
        IEndpointIntegrator(endpoint).recvMessage(srcChain, srcAddr, sequence, payloadHash);
    }
}
