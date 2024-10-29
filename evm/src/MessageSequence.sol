// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "./interfaces/IMessageSequence.sol";

abstract contract MessageSequence is IMessageSequence {
    // =============== Storage ==============================================================

    bytes32 private constant MESSAGE_SEQUENCE_SLOT = bytes32(uint256(keccak256("MessageSequence.slot")) - 1);

    // =============== Internal Storage Getters ==============================================

    function _getMessageSequenceStorage() internal pure returns (mapping(address => uint64) storage $) {
        uint256 slot = uint256(MESSAGE_SEQUENCE_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    // ==================== External Interface ===============================================

    /// @notice Returns the next message sequence for a given sender.
    /// @param sender The address of the sender.
    function nextMessageSequence(address sender) external view returns (uint64) {
        return _getMessageSequenceStorage()[sender];
    }

    // ==================== Internal Interface ===============================================

    /// @notice Returns the next message sequence for a given sender and increments the sequence number.
    ///     This function is internal and is only used by the contract that inherits this contract.
    ///     The message sequence needs to be unique for each sender.
    /// @param sender The address of the sender.
    function _useMessageSequence(address sender) internal returns (uint64 currentSequence) {
        currentSequence = _getMessageSequenceStorage()[sender];
        _getMessageSequenceStorage()[sender]++;
    }
}
