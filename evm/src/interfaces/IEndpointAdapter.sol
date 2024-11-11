// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../libraries/UniversalAddress.sol";

interface IEndpointAdapter {
    /// @notice Called by an Adapter contract to attest to a message.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param dstChain The Wormhole chain ID of the destination.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    function attestMessage(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external;
}
