// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "./IMessageSequence.sol";
import "../libraries/UniversalAddress.sol";

interface IRouter is IMessageSequence {
    /// @dev Send a message to another chain.
    /// @param recipientChain The Wormhole chain ID of the recipient.
    /// @param recipientAddress The Wormhole formatted address of the peer NTT Manager on the recipient chain.
    /// @param message A message to be sent to the nttManager on the recipient chain.
    /// @return uint64 The sequence number of the message.
    function sendMessage(uint16 recipientChain, UniversalAddress recipientAddress, bytes memory message)
        external
        payable
        returns (uint64);
}
