// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "./interfaces/IRouter.sol";
import "./MessageSequence.sol";

contract Router is IRouter, MessageSequence {
    string public constant ROUTER_VERSION = "0.0.1";

    // =============== External ==============================================================

    /// @inheritdoc IRouter
    function sendMessage(uint16 recipientChain, UniversalAddress recipientAddress, bytes memory message)
        external
        payable
        returns (uint64)
    {
        return _sendMessage(recipientChain, recipientAddress, message, msg.sender);
    }

    // =============== Internal ==============================================================

    function _sendMessage(
        uint16, // recipientChain,
        UniversalAddress, // _recipientAddress,
        bytes memory, // _message,
        address sender
    ) internal returns (uint64 sequence) {
        sequence = _useMessageSequence(sender);
    }
}
