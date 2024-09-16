// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "./interfaces/IRouter.sol";

contract Router is IRouter {
    string public constant ROUTER_VERSION = "0.0.1";

    // =============== Storage ==============================================================

    bytes32 private constant MESSAGE_SEQUENCE_SLOT = bytes32(uint256(keccak256("router.messageSequence")) - 1);

    // =============== Storage Getters/Setters ==============================================

    function _getMessageSequenceStorage() internal pure returns (mapping(address => _Sequence) storage $) {
        uint256 slot = uint256(MESSAGE_SEQUENCE_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    // =============== Public Getters ========================================================

    /// @inheritdoc IRouter
    function nextMessageSequence(address sender) external view returns (uint64) {
        return _getMessageSequenceStorage()[sender].num;
    }

    // ==================== External Interface ===============================================

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

    function _useMessageSequence(address sender) internal returns (uint64 currentSequence) {
        currentSequence = _getMessageSequenceStorage()[sender].num;
        _getMessageSequenceStorage()[sender].num++;
    }

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
