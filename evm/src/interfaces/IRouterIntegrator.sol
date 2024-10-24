// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "./IMessageSequence.sol";
import "../libraries/UniversalAddress.sol";

interface IRouterIntegrator is IMessageSequence {
    /// @notice This is the first thing an integrator should do. It registers the integrator with the router
    ///         and sets the administrator contract for that integrator. The admin address is used to manage the transceivers.
    /// @dev The msg.sender needs to be the integrator contract.
    /// @param initialAdmin The address of the admin.
    function register(address initialAdmin) external;

    /// @notice Send a message to another chain.
    /// @param dstChain The Wormhole chain ID of the recipient.
    /// @param dstAddr The universal address of the peer on the recipient chain.
    /// @param payloadHash keccak256 of a message to be sent to the recipient chain.
    /// @return uint64 The sequence number of the message.
    /// @param refundAddress The source chain refund address passed to the Transceiver.
    function sendMessage(uint16 dstChain, UniversalAddress dstAddr, bytes32 payloadHash, address refundAddress)
        external
        payable
        returns (uint64);

    /// @notice Receive a message and mark it executed.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param dstChain The Wormhole chain ID of the destination.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @return (uint128, uint128) The enabled bitmap, and the attested bitmap, respectively.
    function recvMessage(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external payable returns (uint128, uint128);

    /// @notice Execute a message without requiring any attestations.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param dstChain The Wormhole chain ID of the destination.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    function execMessage(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external;

    /// @notice Retrieve the status of a message.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the message.
    /// @param sequence The sequence number of the message.
    /// @param dstChain The Wormhole chain ID of the destination.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @return (uint128, uint128, bool) The enabled bitmap, the attested bitmap, if the message was executed.
    function getMessageStatus(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external returns (uint128, uint128, bool);
}
