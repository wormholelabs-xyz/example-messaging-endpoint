// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "../src/TransceiverRegistry.sol";

contract ConcreteTransceiverRegistry is TransceiverRegistry {
    function addTransceiver(address integrator, uint16 chain, address transceiver) public returns (uint8 index) {
        return _addTransceiver(integrator, chain, transceiver);
    }

    function disableSendTransceiver(address integrator, uint16 chain, address transceiver) public {
        _disableSendTransceiver(integrator, chain, transceiver);
    }

    function disableRecvTransceiver(address integrator, uint16 chain, address transceiver) public {
        _disableRecvTransceiver(integrator, chain, transceiver);
    }

    function getRegisteredTransceiversStorage(address integrator) public view returns (address[] memory $) {
        return _getRegisteredTransceiversStorage()[integrator];
    }

    function getNumTransceiversStorage(address integrator) public view returns (_NumTransceivers memory $) {
        return _getNumTransceiversStorage()[integrator];
    }

    function getEnabledSendTransceiversBitmapForChain(address integrator, uint16 chain)
        public
        view
        returns (address[] memory transceivers)
    {
        return _getEnabledSendTransceiversArrayForChain(integrator, chain);
    }

    function getEnabledRecvTransceiversBitmapForChain(address integrator, uint16 chain)
        public
        view
        returns (uint128 bitmap)
    {
        return _getEnabledRecvTransceiversBitmapForChain(integrator, chain);
    }

    function enableSendTransceiver(address integrator, uint16 chainId, address transceiver) public {
        _enableSendTransceiver(integrator, chainId, transceiver);
    }

    function enableRecvTransceiver(address integrator, uint16 chainId, address transceiver) public {
        _enableRecvTransceiver(integrator, chainId, transceiver);
    }

    function isSendTransceiverEnabledForChain(address integrator, uint16 chainId, address transceiver)
        public
        view
        returns (bool)
    {
        return _isSendTransceiverEnabledForChain(integrator, chainId, transceiver);
    }

    function isRecvTransceiverEnabledForChain(address integrator, uint16 chainId, address transceiver)
        public
        view
        returns (bool)
    {
        return _isRecvTransceiverEnabledForChain(integrator, chainId, transceiver);
    }

    function getMaxTransceivers() public pure returns (uint8) {
        return MAX_TRANSCEIVERS;
    }
}

contract TransceiverRegistryTest is Test {
    ConcreteTransceiverRegistry public transceiverRegistry;
    address integrator1 = address(0x1);
    address integrator2 = address(0x2);
    address zeroTransceiver = address(0);
    address sendTransceiver = address(0x123);
    address recvTransceiver = address(0x234);
    uint16 zeroChain = 0;
    uint16 chain = 2;
    uint16 wrongChain = 3;

    function setUp() public {
        transceiverRegistry = new ConcreteTransceiverRegistry();
    }

    function test1() public view {
        assertEq(transceiverRegistry.getTransceivers(integrator1).length, 0);
        assertEq(transceiverRegistry.getTransceivers(integrator2).length, 0);
    }

    function test2() public {
        address me = address(this);
        // Send side
        assertEq(transceiverRegistry.getTransceivers(me).length, 0);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        transceiverRegistry.addTransceiver(me, zeroChain, sendTransceiver);
        transceiverRegistry.addTransceiver(me, chain, sendTransceiver);

        // Recv side
        // A transceiver was registered on the send side
        assertEq(transceiverRegistry.getTransceivers(me).length, 1);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        transceiverRegistry.addTransceiver(me, zeroChain, recvTransceiver);
        transceiverRegistry.addTransceiver(me, chain, recvTransceiver);
    }

    function test3() public {
        // Need to add transceiver, then enable it, then disable it
        address me = address(this);
        // Send side
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.NonRegisteredTransceiver.selector, sendTransceiver));
        transceiverRegistry.disableSendTransceiver(me, chain, sendTransceiver);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.addTransceiver(me, chain, zeroTransceiver);
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 0, "S1");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 0, "S2");
        transceiverRegistry.addTransceiver(me, chain, sendTransceiver);
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 1, "S3");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 1, "S4");
        // assertEq(transceiverRegistry.getSendTransceiverInfos(integrator1).length, 1);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        transceiverRegistry.disableSendTransceiver(me, zeroChain, sendTransceiver);
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 1, "S5");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 1, "S6");
        vm.expectRevert(
            abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyDisabled.selector, sendTransceiver)
        );
        transceiverRegistry.disableSendTransceiver(me, chain, sendTransceiver);
        transceiverRegistry.enableSendTransceiver(me, chain, sendTransceiver);
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 1, "S7");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 1, "S8");
        transceiverRegistry.disableSendTransceiver(me, chain, sendTransceiver);
        vm.expectRevert(
            abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyDisabled.selector, sendTransceiver)
        );
        transceiverRegistry.disableSendTransceiver(me, chain, sendTransceiver);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.disableSendTransceiver(me, chain, zeroTransceiver);
        // assertEq(transceiverRegistry.getSendTransceiverInfos(integrator1).length, 0);

        // Recv side
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.NonRegisteredTransceiver.selector, recvTransceiver));
        transceiverRegistry.disableRecvTransceiver(me, chain, recvTransceiver);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.addTransceiver(me, chain, zeroTransceiver);
        // Carry over from send side test
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 1, "R1");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 1, "R2");
        transceiverRegistry.addTransceiver(me, chain, recvTransceiver);
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 2, "R3");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 2, "R4");
        // assertEq(transceiverRegistry.getRecvTransceiverInfos(integrator1).length, 1);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        transceiverRegistry.disableRecvTransceiver(me, zeroChain, recvTransceiver);
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 2, "R5");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 2, "R6");
        vm.expectRevert(
            abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyDisabled.selector, recvTransceiver)
        );
        transceiverRegistry.disableRecvTransceiver(me, chain, recvTransceiver);
        transceiverRegistry.enableRecvTransceiver(me, chain, recvTransceiver);
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 2, "R7");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 2, "R8");
        transceiverRegistry.disableRecvTransceiver(me, chain, recvTransceiver);
        vm.expectRevert(
            abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyDisabled.selector, recvTransceiver)
        );
        transceiverRegistry.disableRecvTransceiver(me, chain, recvTransceiver);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.disableRecvTransceiver(me, chain, zeroTransceiver);
        // assertEq(transceiverRegistry.getRecvTransceiverInfos(integrator1).length, 0);
    }

    function test4() public {
        // Send side
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(integrator1).length, 0);
        assertEq(transceiverRegistry.getEnabledSendTransceiversBitmapForChain(integrator1, chain).length, 0);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        assertEq(transceiverRegistry.getEnabledSendTransceiversBitmapForChain(integrator1, zeroChain).length, 0);

        // Recv side
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(integrator1).length, 0);
        assertEq(transceiverRegistry.getEnabledRecvTransceiversBitmapForChain(integrator1, chain), 0);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        assertEq(transceiverRegistry.getEnabledRecvTransceiversBitmapForChain(integrator1, zeroChain), 0);
    }

    // This is a redudant test, as the previous tests already cover this
    function test5() public view {
        // Send side
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(integrator1).length, 0);

        // Recv side
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(integrator1).length, 0);
    }

    // This is a redudant test, as the previous tests already cover this
    function test6() public view {
        // Send side
        TransceiverRegistry._NumTransceivers memory numSendTransceivers =
            transceiverRegistry.getNumTransceiversStorage(integrator1);
        assertEq(numSendTransceivers.registered, 0);

        // Recv side
        TransceiverRegistry._NumTransceivers memory numRecvTransceivers =
            transceiverRegistry.getNumTransceiversStorage(integrator1);
        assertEq(numRecvTransceivers.registered, 0);
    }

    function test7() public {
        address me = address(this);
        // Send side
        address sTransceiver = address(0x456);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.NonRegisteredTransceiver.selector, sTransceiver));
        transceiverRegistry.enableSendTransceiver(me, chain, sTransceiver);

        // Recv side
        address rTransceiver = address(0x567);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.NonRegisteredTransceiver.selector, rTransceiver));
        transceiverRegistry.enableRecvTransceiver(me, chain, rTransceiver);
    }

    function test8() public {
        uint16 zeroChainId = 0;
        uint16 chainId = 3;
        address me = address(this);

        // Send side
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.enableSendTransceiver(me, chainId, zeroTransceiver);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.isSendTransceiverEnabledForChain(me, chainId, zeroTransceiver);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChainId));
        transceiverRegistry.isSendTransceiverEnabledForChain(me, zeroChainId, me);

        // // Recv side
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.enableRecvTransceiver(me, chainId, zeroTransceiver);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.isRecvTransceiverEnabledForChain(me, chainId, zeroTransceiver);
    }

    function test9() public {
        uint16 chainId = 4;
        address me = address(this);

        // Send side
        address sTransceiver = address(0x345);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.NonRegisteredTransceiver.selector, sTransceiver));
        require(transceiverRegistry.isSendTransceiverEnabledForChain(me, chainId, sTransceiver) == false, "S1");
        transceiverRegistry.addTransceiver(me, chain, sTransceiver);
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 1, "S2");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 1, "S3");
        transceiverRegistry.enableSendTransceiver(me, chainId, sTransceiver);
        bool enabled = transceiverRegistry.isSendTransceiverEnabledForChain(me, chainId, sTransceiver);
        require(enabled == true, "S4");
        transceiverRegistry.enableSendTransceiver(me, chain, sTransceiver);

        // Recv side
        address rTransceiver = address(0x453);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.NonRegisteredTransceiver.selector, rTransceiver));
        require(transceiverRegistry.isRecvTransceiverEnabledForChain(me, chainId, rTransceiver) == false, "R1");
        transceiverRegistry.addTransceiver(me, chain, rTransceiver);
        require(transceiverRegistry.getRegisteredTransceiversStorage(me).length == 2, "R2");
        require(transceiverRegistry.getNumTransceiversStorage(me).registered == 2, "R3");
        transceiverRegistry.enableRecvTransceiver(me, chainId, rTransceiver);
        enabled = transceiverRegistry.isRecvTransceiverEnabledForChain(me, chainId, rTransceiver);
        require(enabled == true, "R4");
        transceiverRegistry.enableRecvTransceiver(me, chain, rTransceiver);
    }

    function test10() public {
        address me = address(this);
        uint8 maxTransceivers = transceiverRegistry.getMaxTransceivers();

        // Send side
        for (uint8 i = 0; i < maxTransceivers; i++) {
            transceiverRegistry.addTransceiver(me, chain, address(uint160(i + 1)));
        }
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TooManyTransceivers.selector));
        transceiverRegistry.addTransceiver(me, chain, address(0x111));
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        for (uint8 i = 0; i < maxTransceivers; i++) {
            transceiverRegistry.enableSendTransceiver(me, chain, address(uint160(i + 1)));
        }
        transceiverRegistry.disableSendTransceiver(me, chain, address(uint160(30)));
        vm.expectRevert(
            abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyDisabled.selector, address(uint160(30)))
        );
        transceiverRegistry.disableSendTransceiver(me, chain, address(uint160(30)));
        transceiverRegistry.getSendTransceiversByChain(me, chain);
    }

    function test11() public {
        address me = address(this);
        uint8 maxTransceivers = transceiverRegistry.getMaxTransceivers();

        // Recv side
        for (uint8 i = 0; i < maxTransceivers; i++) {
            transceiverRegistry.addTransceiver(me, chain, address(uint160(i + 1)));
        }
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TooManyTransceivers.selector));
        transceiverRegistry.addTransceiver(me, chain, address(0x111));
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        transceiverRegistry.enableRecvTransceiver(me, chain, address(0x1));
        transceiverRegistry.enableRecvTransceiver(me, chain, address(0x2));
        transceiverRegistry.disableRecvTransceiver(me, chain, address(0x2));
        transceiverRegistry.disableRecvTransceiver(me, chain, address(0x1));
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyDisabled.selector, address(0x1)));
        transceiverRegistry.disableSendTransceiver(me, chain, address(0x1));
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TooManyTransceivers.selector));
        transceiverRegistry.addTransceiver(me, chain, address(0x111));
    }

    function test_getSendTransceiversByChain() public {
        address me = address(this);
        uint16 chain1 = 1;
        uint16 chain2 = 2;
        address transceiver1 = address(0x1); // enabled, chain 1
        address transceiver2 = address(0x2); // enabled, chain 1
        address transceiver3 = address(0x3); // enabled, chain 2
        address transceiver4 = address(0x4); // disabled, chain 2

        transceiverRegistry.addTransceiver(me, chain1, transceiver1);
        transceiverRegistry.enableSendTransceiver(me, chain1, transceiver1);
        transceiverRegistry.addTransceiver(me, chain1, transceiver2);
        transceiverRegistry.enableSendTransceiver(me, chain1, transceiver2);
        transceiverRegistry.addTransceiver(me, chain2, transceiver3);
        transceiverRegistry.enableSendTransceiver(me, chain2, transceiver3);
        transceiverRegistry.addTransceiver(me, chain2, transceiver4);
        address[] memory chain1Addrs = transceiverRegistry.getSendTransceiversByChain(me, chain1);
        require(chain1Addrs.length == 2, "Wrong number of transceivers enabled on chain one");
        address[] memory chain2Addrs = transceiverRegistry.getSendTransceiversByChain(me, chain2);
        require(chain2Addrs.length == 1, "Wrong number of transceivers enabled on chain two");
        transceiverRegistry.enableSendTransceiver(me, chain2, transceiver4);
        transceiverRegistry.disableSendTransceiver(me, chain2, transceiver3);
        require(chain2Addrs.length == 1, "Wrong number of transceivers enabled on chain two");
    }

    function test_getRecvTransceiversByChain() public {
        address me = address(this);
        uint16 chain1 = 1;
        uint16 chain2 = 2;
        address transceiver1 = address(0x1); // enabled, chain 1
        address transceiver2 = address(0x2); // enabled, chain 1
        address transceiver3 = address(0x3); // enabled, chain 2
        address transceiver4 = address(0x4); // disabled, chain 2

        transceiverRegistry.addTransceiver(me, chain1, transceiver1);
        transceiverRegistry.enableRecvTransceiver(me, chain1, transceiver1);
        transceiverRegistry.addTransceiver(me, chain1, transceiver2);
        transceiverRegistry.enableRecvTransceiver(me, chain1, transceiver2);
        transceiverRegistry.addTransceiver(me, chain2, transceiver3);
        transceiverRegistry.enableRecvTransceiver(me, chain2, transceiver3);
        transceiverRegistry.addTransceiver(me, chain2, transceiver4);
        address[] memory chain1Addrs = transceiverRegistry.getRecvTransceiversByChain(me, chain1);
        require(chain1Addrs.length == 2, "Wrong number of transceivers enabled on chain one");
        address[] memory chain2Addrs = transceiverRegistry.getRecvTransceiversByChain(me, chain2);
        require(chain2Addrs.length == 1, "Wrong number of transceivers enabled on chain two");
        transceiverRegistry.enableRecvTransceiver(me, chain2, transceiver4);
        transceiverRegistry.disableRecvTransceiver(me, chain2, transceiver3);
        require(chain2Addrs.length == 1, "Wrong number of transceivers enabled on chain two");
    }
}
