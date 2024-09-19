// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "./IMessageSequence.sol";
import "../libraries/UniversalAddress.sol";

interface IRouter is IMessageSequence {
    /// @dev Send a message to another chain.
    /// @param recipientChain The Wormhole chain ID of the recipient.
    /// @param recipientAddress The universal address of the peer on the recipient chain.
    /// @param refundAddress The source chain address to refund the message fee to.
    /// @param message A message to be sent to the recipient chain.
    /// @return uint64 The sequence number of the message.
    function sendMessage(
        uint16 recipientChain,
        UniversalAddress recipientAddress,
        address refundAddress,
        bytes memory message
    ) external payable returns (uint64);
}
