// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "../libraries/UniversalAddress.sol";

interface ITransceiver {
    /// @notice The caller is not the NttManager.
    /// @dev Selector: 0xc5aa6153.
    /// @param caller The address of the caller.
    error CallerNotRouter(address caller);

    /// @notice Returns the string type of the transceiver. E.g. "wormhole", "axelar", etc.
    function getTransceiverType() external view returns (string memory);

    /// @notice Fetch the delivery price for a given recipient chain transfer.
    /// @param recipientChain The Wormhole chain ID of the target chain.
    /// @return deliveryPrice The cost of delivering a message to the recipient chain,
    ///         in this chain's native token.
    function quoteDeliveryPrice(uint16 recipientChain) external view returns (uint256);

    /// @dev Send a message to another chain.
    /// @param recipientChain The Wormhole chain ID of the recipient.
    /// @param messageHash The hash of the message to be sent to the recipient chain.
    /// @param recipientAddress The Wormhole formatted address of the recipient chain.
    /// @param refundAddress The address of the refund recipient
    function sendMessage(
        uint16 recipientChain,
        bytes32 messageHash,
        UniversalAddress recipientAddress,
        bytes32 refundAddress
    ) external payable;
}
