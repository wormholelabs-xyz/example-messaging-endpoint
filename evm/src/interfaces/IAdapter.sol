// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../libraries/UniversalAddress.sol";

interface IAdapter {
    /// @notice The caller is not the Endpoint.
    /// @dev Selector: 0xfb217bcd.
    /// @param caller The address of the caller.
    error CallerNotEndpoint(address caller);

    /// @notice Returns the string type of the adapter. E.g. "wormhole", "axelar", etc.
    function getAdapterType() external view returns (string memory);

    /// @notice Fetch the delivery price for a given recipient chain transfer.
    /// @param recipientChain The Wormhole chain ID of the target chain.
    /// @return deliveryPrice The cost of delivering a message to the recipient chain in this chain's native token.
    function quoteDeliveryPrice(uint16 recipientChain) external view returns (uint256);

    /// @dev Send a message to another chain.
    /// @param srcAddr The universal address of the sender.
    /// @param sequence The per-integrator sequence number associated with the message.
    /// @param dstChain The Wormhole chain ID of the recipient.
    /// @param dstAddr The universal address of the recipient.
    /// @param payloadHash The hash of the message to be sent to the recipient chain.
    /// @param refundAddr The address of the refund recipient.
    function sendMessage(
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash,
        address refundAddr
    ) external payable;
}
