// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "./interfaces/IRouter.sol";
import "./MessageSequence.sol";

contract Router is IRouter, MessageSequence {
    string public constant ROUTER_VERSION = "0.0.1";

    /// @inheritdoc IRouter
    function sendMessage(
        uint16 recipientChain,
        bytes32 recipientAddress,
        bytes memory message,
        TransceiverStructs.TransceiverInstruction[] memory instructions
    ) external payable returns (uint64) {
        return _sendMessage(recipientChain, recipientAddress, message, instructions, msg.sender);
    }

    // =============== Internal ==============================================================

    function _sendMessage(
        uint16, // recipientChain,
        bytes32, // _recipientAddress,
        bytes memory, // _message,
        TransceiverStructs.TransceiverInstruction[] memory, // _instructions,
        address sender
    ) internal returns (uint64 sequence) {
        sequence = _useMessageSequence(sender);
    }
}
