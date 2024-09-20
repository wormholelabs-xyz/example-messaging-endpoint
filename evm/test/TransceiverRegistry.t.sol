// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "../src/TransceiverRegistry.sol";

contract ConcreteTransceiverRegistry is TransceiverRegistry {
    function rmvSendTransceiver(address integrator, address transceiver, uint16 chain) public {
        _disableSendTransceiver(integrator, transceiver, chain);
    }

    function rmvRecvTransceiver(address integrator, address transceiver, uint16 chain) public {
        _disableRecvTransceiver(integrator, transceiver, chain);
    }

    function setSendTransceiver(address integrator, address transceiver, uint16 chain) public returns (uint8 index) {
        return _setSendTransceiver(integrator, transceiver, chain);
    }

    function setRecvTransceiver(address integrator, address transceiver, uint16 chain) public returns (uint8 index) {
        return _setRecvTransceiver(integrator, transceiver, chain);
    }

    function disableSendTransceiver(address integrator, address transceiver, uint16 chain) public {
        _disableSendTransceiver(integrator, transceiver, chain);
    }

    function disableRecvTransceiver(address integrator, address transceiver, uint16 chain) public {
        _disableRecvTransceiver(integrator, transceiver, chain);
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
        returns (uint128 bitmap)
    {
        return _getEnabledSendTransceiversBitmapForChain(integrator, chain);
    }

    function getEnabledRecvTransceiversBitmapForChain(address integrator, uint16 chain)
        public
        view
        returns (uint128 bitmap)
    {
        return _getEnabledRecvTransceiversBitmapForChain(integrator, chain);
    }

    function enableSendTransceiverForChain(address integrator, address transceiver, uint16 chainId) public {
        _enableSendTransceiverForChain(integrator, transceiver, chainId);
    }

    function enableRecvTransceiverForChain(address integrator, address transceiver, uint16 chainId) public {
        _enableRecvTransceiverForChain(integrator, transceiver, chainId);
    }

    function isSendTransceiverEnabledForChain(address integrator, address transceiver, uint16 chainId)
        public
        view
        returns (bool)
    {
        return _isSendTransceiverEnabledForChain(integrator, transceiver, chainId);
    }

    function isRecvTransceiverEnabledForChain(address integrator, address transceiver, uint16 chainId)
        public
        view
        returns (bool)
    {
        return _isRecvTransceiverEnabledForChain(integrator, transceiver, chainId);
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
        transceiverRegistry.setSendTransceiver(me, sendTransceiver, zeroChain);
        transceiverRegistry.setSendTransceiver(me, sendTransceiver, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        transceiverRegistry.disableSendTransceiver(me, sendTransceiver, zeroChain);

        // Recv side
        // Transceiver was registered on the send side
        assertEq(transceiverRegistry.getTransceivers(me).length, 1);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        transceiverRegistry.setRecvTransceiver(me, sendTransceiver, zeroChain);
        transceiverRegistry.setRecvTransceiver(me, recvTransceiver, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        transceiverRegistry.disableRecvTransceiver(me, recvTransceiver, zeroChain);
    }

    function test3() public {
        address me = address(this);
        // Send side
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.NonRegisteredTransceiver.selector, sendTransceiver));
        transceiverRegistry.rmvSendTransceiver(me, sendTransceiver, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.setSendTransceiver(me, zeroTransceiver, chain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 0);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 0);
        transceiverRegistry.setSendTransceiver(me, sendTransceiver, chain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 1);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 1);
        // assertEq(transceiverRegistry.getSendTransceiverInfos(integrator1).length, 1);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        transceiverRegistry.disableSendTransceiver(me, sendTransceiver, zeroChain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 1);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 1);
        transceiverRegistry.disableSendTransceiver(me, sendTransceiver, chain);
        // disabled, but stays registered
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 1);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 1);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.DisabledTransceiver.selector, sendTransceiver));
        transceiverRegistry.disableSendTransceiver(me, sendTransceiver, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.disableSendTransceiver(me, zeroTransceiver, chain);
        // assertEq(transceiverRegistry.getSendTransceiverInfos(integrator1).length, 0);

        // Recv side
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.NonRegisteredTransceiver.selector, recvTransceiver));
        transceiverRegistry.rmvRecvTransceiver(me, recvTransceiver, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.setRecvTransceiver(me, zeroTransceiver, chain);
        // Carry over from send side test
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 1);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 1);
        transceiverRegistry.setRecvTransceiver(me, recvTransceiver, chain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 2);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 2);
        // assertEq(transceiverRegistry.getRecvTransceiverInfos(integrator1).length, 1);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        transceiverRegistry.disableRecvTransceiver(me, recvTransceiver, zeroChain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 2);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 2);
        transceiverRegistry.disableRecvTransceiver(me, recvTransceiver, chain);
        // disabled, but stays registered
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 2);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 2);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.DisabledTransceiver.selector, recvTransceiver));
        transceiverRegistry.disableRecvTransceiver(me, recvTransceiver, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.disableRecvTransceiver(me, zeroTransceiver, chain);
        // assertEq(transceiverRegistry.getRecvTransceiverInfos(integrator1).length, 0);
    }

    function test4() public {
        // Send side
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(integrator1).length, 0);
        assertEq(transceiverRegistry.getEnabledSendTransceiversBitmapForChain(integrator1, chain), 0);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        assertEq(transceiverRegistry.getEnabledSendTransceiversBitmapForChain(integrator1, zeroChain), 0);

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
        transceiverRegistry.enableSendTransceiverForChain(me, sTransceiver, chain);

        // Recv side
        address rTransceiver = address(0x567);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.NonRegisteredTransceiver.selector, rTransceiver));
        transceiverRegistry.enableRecvTransceiverForChain(me, rTransceiver, chain);
    }

    function test8() public {
        uint16 chainId = 3;
        address me = address(this);

        // Send side
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.enableSendTransceiverForChain(me, zeroTransceiver, chainId);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.isSendTransceiverEnabledForChain(me, zeroTransceiver, chainId);

        // Recv side
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.enableRecvTransceiverForChain(me, zeroTransceiver, chainId);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidTransceiverZeroAddress.selector));
        transceiverRegistry.isRecvTransceiverEnabledForChain(me, zeroTransceiver, chainId);
    }

    function test9() public {
        uint16 chainId = 4;
        address me = address(this);

        // Send side
        address sTransceiver = address(0x345);
        assertEq(transceiverRegistry.isSendTransceiverEnabledForChain(me, sTransceiver, chainId), false);
        transceiverRegistry.setSendTransceiver(me, sTransceiver, chain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 1);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 1);
        transceiverRegistry.enableSendTransceiverForChain(me, sTransceiver, chainId);
        bool enabled = transceiverRegistry.isSendTransceiverEnabledForChain(me, sTransceiver, chainId);
        assertEq(enabled, true);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyEnabled.selector, sTransceiver));
        transceiverRegistry.setSendTransceiver(me, sTransceiver, chain);

        // Recv side
        address rTransceiver = address(0x453);
        assertEq(transceiverRegistry.isRecvTransceiverEnabledForChain(me, rTransceiver, chainId), false);
        transceiverRegistry.setRecvTransceiver(me, rTransceiver, chain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, 2);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, 2);
        transceiverRegistry.enableRecvTransceiverForChain(me, rTransceiver, chainId);
        enabled = transceiverRegistry.isRecvTransceiverEnabledForChain(me, rTransceiver, chainId);
        assertEq(enabled, true);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyEnabled.selector, rTransceiver));
        transceiverRegistry.setRecvTransceiver(me, rTransceiver, chain);
    }

    function test10() public {
        address me = address(this);
        uint8 maxTransceivers = transceiverRegistry.getMaxTransceivers();

        // Send side
        for (uint8 i = 0; i < maxTransceivers; i++) {
            transceiverRegistry.setSendTransceiver(me, address(uint160(i + 1)), chain);
        }
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TooManyTransceivers.selector));
        transceiverRegistry.setSendTransceiver(me, address(0x111), chain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        transceiverRegistry.disableSendTransceiver(me, address(0x1), chain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TooManyTransceivers.selector));
        transceiverRegistry.setSendTransceiver(me, address(0x111), chain);
    }

    function test11() public {
        address me = address(this);
        uint8 maxTransceivers = transceiverRegistry.getMaxTransceivers();

        // Recv side
        for (uint8 i = 0; i < maxTransceivers; i++) {
            transceiverRegistry.setRecvTransceiver(me, address(uint160(i + 1)), chain);
        }
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TooManyTransceivers.selector));
        transceiverRegistry.setRecvTransceiver(me, address(0x111), chain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        transceiverRegistry.disableRecvTransceiver(me, address(0x1), chain);
        assertEq(transceiverRegistry.getRegisteredTransceiversStorage(me).length, maxTransceivers);
        assertEq(transceiverRegistry.getNumTransceiversStorage(me).registered, maxTransceivers);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TooManyTransceivers.selector));
        transceiverRegistry.setRecvTransceiver(me, address(0x111), chain);
    }
}
