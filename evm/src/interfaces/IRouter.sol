// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "../libraries/TransceiverStructs.sol";
import "./IMessageSequence.sol";

interface IRouter is IMessageSequence {
    /// @dev Send a message to another chain.
    /// @param recipientChain The Wormhole chain ID of the recipient.
    /// @param recipientAddress The Wormhole formatted address of the peer NTT Manager on the recipient chain.
    /// @param message A message to be sent to the nttManager on the recipient chain.
    /// @param instructions Additional instructions provided to the Transceiver.
    function sendMessage(
        uint16 recipientChain,
        bytes32 recipientAddress,
        bytes memory message,
        TransceiverStructs.TransceiverInstruction[] memory instructions
    ) external payable returns (uint64);
}
