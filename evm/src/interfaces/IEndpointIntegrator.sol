// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "./IMessageSequence.sol";
import "../libraries/UniversalAddress.sol";

interface IEndpointIntegrator is IMessageSequence {
    /// @notice This is the first thing an integrator should do. It registers the integrator with the endpoint
    ///         and sets the administrator contract for that integrator. The admin address is used to manage the adapters.
    /// @dev The msg.sender needs to be the integrator contract.
    /// @param initialAdmin The address of the admin.
    function register(address initialAdmin) external;

    /// @notice Sends a message to another chain.
    /// @param dstChain The Wormhole chain ID of the recipient.
    /// @param dstAddr The universal address of the peer on the recipient chain.
    /// @param payloadHash keccak256 of a message to be sent to the recipient chain.
    /// @return uint64 The sequence number of the message.
    /// @param refundAddress The source chain refund address passed to the Adapter.
    function sendMessage(uint16 dstChain, UniversalAddress dstAddr, bytes32 payloadHash, address refundAddress)
        external
        payable
        returns (uint64);

    /// @notice Receives a message and marks it as executed.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @return (uint128, uint128) The enabled bitmap, and the attested bitmap, respectively.
    function recvMessage(uint16 srcChain, UniversalAddress srcAddr, uint64 sequence, bytes32 payloadHash)
        external
        payable
        returns (uint128, uint128);

    /// @notice Executes a message without requiring any attestations.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param payloadHash The keccak256 of payload from the integrator.
    function execMessage(uint16 srcChain, UniversalAddress srcAddr, uint64 sequence, bytes32 payloadHash) external;

    /// @notice Retrieves the status of a message.
    /// @dev This version can be called by anyone.
    ///      However, it is expected that the dstAddr is the Integrator's UniversalAddress on this chain.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the message.
    /// @param sequence The sequence number of the message.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @return (uint128, uint128, bool) The enabled bitmap, the attested bitmap, if the message was executed.
    function getMessageStatus(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external view returns (uint128, uint128, bool);

    /// @notice Retrieves the status of a message.
    /// @dev This version is expected to be called by the integrator on the destination chain.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the message.
    /// @param sequence The sequence number of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @return (uint128, uint128, bool) The enabled bitmap, the attested bitmap, if the message was executed.
    function getMessageStatus(uint16 srcChain, UniversalAddress srcAddr, uint64 sequence, bytes32 payloadHash)
        external
        view
        returns (uint128, uint128, bool);

    /// @notice Retrieves the quote for message delivery.
    /// @dev This version does not need to be called by the integrator.
    /// @dev This sums up all the individual sendAdapter's quoteDeliveryPrice calls.
    /// @param integrator The address of the integrator.
    /// @param dstChain The Wormhole chain ID of the recipient.
    /// @return uint256 The total cost of delivering a message to the recipient chain in this chain's native token.
    function quoteDeliveryPrice(address integrator, uint16 dstChain) external returns (uint256);

    /// @notice Retrieves the quote for message delivery.
    /// @dev This version must be called by the integrator.
    /// @dev This sums up all the individual sendAdapter's quoteDeliveryPrice calls.
    /// @param dstChain The Wormhole chain ID of the recipient.
    /// @return uint256 The total cost of delivering a message to the recipient chain in this chain's native token.
    function quoteDeliveryPrice(uint16 dstChain) external view returns (uint256);
}
