// SPDX-License-Identifier: Apache 2
pragma solidity >=0.8.8 <0.9.0;

import "wormhole-solidity-sdk/libraries/BytesParsing.sol";

library AdapterInstructions {
    using BytesParsing for bytes;

    /// @notice Error thrown when there are too many instructions in the array.
    /// @dev Selector 0x3c46992e.
    error TooManyInstructions();

    /// @notice Error thrown when the payload length exceeds the allowed maximum.
    /// @dev Selector 0xa3419691.
    /// @param size The size of the payload.
    error PayloadTooLong(uint256 size);

    /// @notice Error thrown when an Adapter instruction index
    ///         is greater than the number of registered Adapters
    /// @dev We index from 0 so if providedIndex == numAdapters then we're out-of-bounds too
    /// @dev Selector 0x689f5016.
    /// @param providedIndex The index specified in the instruction
    /// @param numAdapters The number of registered Adapters
    error InvalidInstructionIndex(uint256 providedIndex, uint256 numAdapters);

    /// @dev Variable-length Adapter-specific instruction that can be passed by the integrator to the endpoint
    ///      and by the endpoint to the adapter.
    ///      The index field refers to the index of the adapter that this instruction should be passed to.
    ///      The serialization format is:
    ///      - index - 1 byte
    ///      - payloadLength - 2 bytes
    ///      - payload - `payloadLength` bytes
    struct Instruction {
        uint8 index;
        bytes payload;
    }

    /// @notice Encodes an adapter instruction.
    /// @param instruction The instruction to be encoded.
    /// @return encoded The encoded bytes, where the first byte is the index and the next two bytes are the instruction length.
    function encodeInstruction(Instruction calldata instruction) public pure returns (bytes memory encoded) {
        if (instruction.payload.length > type(uint16).max) {
            revert PayloadTooLong(instruction.payload.length);
        }
        uint16 payloadLength = uint16(instruction.payload.length);
        encoded = abi.encodePacked(instruction.index, payloadLength, instruction.payload);
    }

    /// @notice Encodes an array of adapter instructions.
    /// @param instructions The array of instructions to be encoded.
    /// @return address The encoded bytes, where the first byte is the number of entries.
    function encodeInstructions(Instruction[] calldata instructions) public pure returns (bytes memory) {
        if (instructions.length > type(uint8).max) {
            revert TooManyInstructions();
        }
        uint256 instructionsLength = instructions.length;

        bytes memory encoded;
        for (uint256 i = 0; i < instructionsLength;) {
            bytes memory innerEncoded = encodeInstruction(instructions[i]);
            encoded = bytes.concat(encoded, innerEncoded);
            unchecked {
                ++i;
            }
        }
        return abi.encodePacked(uint8(instructionsLength), encoded);
    }

    /// @notice Parses a byte array into an adapter instruction.
    /// @param encoded The encoded instruction.
    /// @return instruction The parsed instruction.
    function parseInstruction(bytes calldata encoded) public pure returns (Instruction memory instruction) {
        uint256 offset = 0;
        (instruction, offset) = parseInstructionUnchecked(encoded, offset);
        encoded.checkLength(offset);
    }

    /// @notice Parses a byte array into an adapter instruction without checking for leftover bytes.
    /// @param encoded The buffer being parsed.
    /// @param offset The current offset into the encoded buffer.
    /// @return instruction The parsed instruction.
    /// @return nextOffset The next index into the array (used for further parsing).
    function parseInstructionUnchecked(bytes calldata encoded, uint256 offset)
        public
        pure
        returns (Instruction memory instruction, uint256 nextOffset)
    {
        (instruction.index, nextOffset) = encoded.asUint8Unchecked(offset);
        uint16 instructionLength;
        (instructionLength, nextOffset) = encoded.asUint16Unchecked(nextOffset);
        (instruction.payload, nextOffset) = encoded.sliceUnchecked(nextOffset, instructionLength);
    }

    /// @notice Parses a byte array into an array of adapter instructions.
    /// @param encoded The encoded instructions.
    /// @param numRegisteredAdapters The total number of registered adapters.
    /// @return instructions A sparse array of adapter instructions, where the index into the array is the adapter index.
    function parseInstructions(bytes calldata encoded, uint256 numRegisteredAdapters)
        public
        pure
        returns (Instruction[] memory instructions)
    {
        // We allocate an array with the length of the number of registered Adapters
        // This gives us the flexibility to not have to pass instructions for Adapters that
        // don't need them.
        instructions = new Instruction[](numRegisteredAdapters);

        if (encoded.length == 0) {
            return instructions;
        }

        uint256 offset = 0;
        uint256 instructionsLength;
        (instructionsLength, offset) = encoded.asUint8Unchecked(offset);

        for (uint256 i = 0; i < instructionsLength;) {
            Instruction memory instruction;
            (instruction, offset) = parseInstructionUnchecked(encoded, offset);

            uint8 instructionIndex = instruction.index;

            // Instruction index is out of bounds
            if (instructionIndex >= numRegisteredAdapters) {
                revert InvalidInstructionIndex(instructionIndex, numRegisteredAdapters);
            }

            instructions[instructionIndex] = instruction;
            unchecked {
                ++i;
            }
        }

        encoded.checkLength(offset);
    }
}
