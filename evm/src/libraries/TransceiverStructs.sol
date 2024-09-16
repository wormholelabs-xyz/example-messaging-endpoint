// SPDX-License-Identifier: Apache 2
pragma solidity ^0.8.13;

library TransceiverStructs {
    /// @notice Error thrown when the payload length exceeds the allowed maximum.
    /// @dev Selector 0xa3419691.
    /// @param size The size of the payload.
    error PayloadTooLong(uint256 size);

    /// @dev Variable-length transceiver-specific instruction that can be passed by the caller to the Router.
    ///      The index field refers to the index of the registeredTransceiver that this instruction should be passed to.
    ///      The serialization format is:
    ///      - index - 1 byte
    ///      - payloadLength - 1 byte
    ///      - payload - `payloadLength` bytes
    struct TransceiverInstruction {
        uint8 index;
        bytes payload;
    }
}
