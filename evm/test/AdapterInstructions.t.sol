// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "../src/libraries/AdapterInstructions.sol";

contract AdapterInstructionsTest is Test {
    function setUp() public {}

    function test_encodeInstruction() public {
        // Success case.
        bytes memory payload = "The payload";
        bytes memory expected = abi.encodePacked(uint8(0), uint16(payload.length), payload);
        AdapterInstructions.Instruction memory inst = AdapterInstructions.Instruction(0, payload);
        bytes memory encoded = AdapterInstructions.encodeInstruction(inst);
        assertEq(keccak256(expected), keccak256(encoded));

        // Payload too long.
        inst = AdapterInstructions.Instruction(0, new bytes(65537));
        vm.expectRevert(abi.encodeWithSelector(AdapterInstructions.PayloadTooLong.selector, 65537));
        AdapterInstructions.encodeInstruction(inst);
    }

    function test_encodeInstructions() public {
        // Success case.
        bytes memory expected = abi.encodePacked(
            uint8(3),
            uint8(0),
            uint16(29),
            "Instructions for adapter zero",
            uint8(3),
            uint16(30),
            "Instructions for adapter three",
            uint8(2),
            uint16(28),
            "Instructions for adapter two"
        );

        AdapterInstructions.Instruction[] memory insts = new AdapterInstructions.Instruction[](3);
        insts[0] = AdapterInstructions.Instruction(0, "Instructions for adapter zero");
        insts[1] = AdapterInstructions.Instruction(3, "Instructions for adapter three");
        insts[2] = AdapterInstructions.Instruction(2, "Instructions for adapter two");
        bytes memory encoded = AdapterInstructions.encodeInstructions(insts);
        assertEq(keccak256(expected), keccak256(encoded));

        // Too many instructions should revert.
        insts = new AdapterInstructions.Instruction[](257);
        for (uint256 idx = 0; idx < 257; ++idx) {
            insts[idx] = AdapterInstructions.Instruction(uint8(idx), "Some instruction");
        }
        vm.expectRevert(abi.encodeWithSelector(AdapterInstructions.TooManyInstructions.selector));
        encoded = AdapterInstructions.encodeInstructions(insts);

        // Should be able to encode an empty array.
        insts = new AdapterInstructions.Instruction[](0);
        encoded = AdapterInstructions.encodeInstructions(insts);
        assertEq(1, encoded.length);
        assertEq(0, encoded[0]);
    }

    function test_parseInstruction() public pure {
        AdapterInstructions.Instruction memory expected = AdapterInstructions.Instruction(0, "The payload");
        bytes memory encoded = AdapterInstructions.encodeInstruction(expected);
        AdapterInstructions.Instruction memory inst = AdapterInstructions.parseInstruction(encoded);
        assertEq(expected.index, inst.index);
        assertEq(keccak256(expected.payload), keccak256(inst.payload));
    }

    // We need this to make the coverage tool happy, even though this function was called in the previous test.
    function test_parseInstructionUnchecked() public pure {
        AdapterInstructions.Instruction memory expected = AdapterInstructions.Instruction(0, "The payload");
        bytes memory encoded = AdapterInstructions.encodeInstruction(expected);
        (AdapterInstructions.Instruction memory inst, uint256 nextOffset) =
            AdapterInstructions.parseInstructionUnchecked(encoded, 0);
        assertEq(expected.index, inst.index);
        assertEq(keccak256(expected.payload), keccak256(inst.payload));
        assertEq(encoded.length, nextOffset);
    }

    function test_parseInstructions() public {
        // Success case.
        bytes memory expectedInst0 = "Instructions for adapter zero";
        bytes memory expectedInst2 = "Instructions for adapter two";
        bytes memory expectedInst3 = "Instructions for adapter three";
        AdapterInstructions.Instruction[] memory expected = new AdapterInstructions.Instruction[](3);
        expected[0] = AdapterInstructions.Instruction(0, expectedInst0);
        expected[1] = AdapterInstructions.Instruction(3, expectedInst3);
        expected[2] = AdapterInstructions.Instruction(2, expectedInst2);
        bytes memory encoded = AdapterInstructions.encodeInstructions(expected);

        AdapterInstructions.Instruction[] memory insts = AdapterInstructions.parseInstructions(encoded, 4);
        assertEq(4, insts.length);

        assertEq(0, insts[0].index);
        assertEq(keccak256(expectedInst0), keccak256(insts[0].payload));

        // Entry one should be empty.
        assertEq(0, insts[1].index);
        assertEq(0, insts[1].payload.length);

        assertEq(2, insts[2].index);
        assertEq(keccak256(expectedInst2), keccak256(insts[2].payload));

        assertEq(3, insts[3].index);
        assertEq(keccak256(expectedInst3), keccak256(insts[3].payload));

        // Index out of range should revert.
        vm.expectRevert(abi.encodeWithSelector(AdapterInstructions.InvalidInstructionIndex.selector, 3, 3));
        AdapterInstructions.parseInstructions(encoded, 3);

        // Should be able to parse an encoded empty array.
        insts = new AdapterInstructions.Instruction[](0);
        encoded = AdapterInstructions.encodeInstructions(insts);
        insts = AdapterInstructions.parseInstructions(encoded, 4);
        assertEq(4, insts.length);
        for (uint256 i = 0; i < 4; ++i) {
            assertEq(0, insts[i].index);
            assertEq(0, insts[i].payload.length);
        }

        // Should be able to parse a *really* empty array.
        insts = AdapterInstructions.parseInstructions(new bytes(0), 4);
        assertEq(4, insts.length);
        for (uint256 i = 0; i < 4; ++i) {
            assertEq(0, insts[i].index);
            assertEq(0, insts[i].payload.length);
        }
    }
}
