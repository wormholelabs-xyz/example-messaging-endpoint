// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

interface IMessageSequence {
    /// @notice Returns the next message sequence for a given sender.
    /// @param sender The address of the sender,
    function nextMessageSequence(address sender) external view returns (uint64);
}
