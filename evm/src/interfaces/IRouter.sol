// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "./IMessageSequence.sol";
import "../libraries/UniversalAddress.sol";

interface IRouter is IMessageSequence {
    /// @dev Send a message to another chain.
    /// @param recipientChain The Wormhole chain ID of the recipient.
    /// @param recipientAddress The universal address of the peer on the recipient chain.
    /// @param refundAddress The source chain refund address passed to the Transceiver.
    /// @param payloadHash keccak256 of a message to be sent to the recipient chain.
    /// @return uint64 The sequence number of the message.
    function sendMessage(
        uint16 recipientChain,
        UniversalAddress recipientAddress,
        address refundAddress,
        bytes32 payloadHash
    ) external payable returns (uint64);

    // /// @dev Receive a message from another chain called by integrator.
    // /// @param sourceChain The Wormhole chain ID of the recipient.
    // /// @param senderAddress The universal address of the peer on the recipient chain.
    // /// @param refundAddress The source chain refund address passed to the Transceiver.
    // /// @param message A message to be sent to the recipient chain.
    // /// @return uint128 The bitmap
    function receiveMessage(
        uint16 sourceChain,
        UniversalAddress senderAddress,
        address refundAddress,
        bytes32 messageHash
    ) external payable returns (uint128);

    /// @notice Called by a Transceiver contract to deliver a verified attestation.
    function attestMessage(
        uint16 sourceChain, // Wormhole Chain ID
        UniversalAddress sourceAddress, // UniversalAddress of the message sender (integrator)
        uint64 sequence, // Next sequence number for that integrator (consuming the sequence number)
        uint16 destinationChainId, // Wormhole Chain ID
        UniversalAddress destinationAddress, // UniversalAddress of the messsage recipient (integrator on destination chain)
        bytes32 payloadHash // keccak256 of arbitrary payload from the integrator
    ) external;
}
