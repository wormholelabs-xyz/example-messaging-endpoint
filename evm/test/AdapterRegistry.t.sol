// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "../src/AdapterRegistry.sol";

contract ConcreteAdapterRegistry is AdapterRegistry {
    function addAdapter(address integrator, address adapter) public returns (uint8 index) {
        return _addAdapter(integrator, adapter);
    }

    function disableSendAdapter(address integrator, uint16 chain, address adapter) public {
        _disableSendAdapter(integrator, chain, adapter);
    }

    function disableRecvAdapter(address integrator, uint16 chain, address adapter) public {
        _disableRecvAdapter(integrator, chain, adapter);
    }

    function getRegisteredAdaptersStorage(address integrator) public view returns (address[] memory $) {
        return _getRegisteredAdaptersStorage()[integrator];
    }

    function getEnabledSendAdaptersBitmapForChain(address integrator, uint16 chain)
        public
        view
        returns (address[] memory adapters)
    {
        return _getEnabledSendAdaptersArrayForChain(integrator, chain);
    }

    function getEnabledRecvAdaptersBitmapForChain(address integrator, uint16 chain)
        public
        view
        returns (uint128 bitmap)
    {
        return _getEnabledRecvAdaptersBitmapForChain(integrator, chain);
    }

    function enableSendAdapter(address integrator, uint16 chainId, address adapter) public {
        _enableSendAdapter(integrator, chainId, adapter);
    }

    function enableRecvAdapter(address integrator, uint16 chainId, address adapter) public {
        _enableRecvAdapter(integrator, chainId, adapter);
    }

    function isSendAdapterEnabledForChain(address integrator, uint16 chainId, address adapter)
        public
        view
        returns (bool)
    {
        return _isSendAdapterEnabledForChainWithCheck(integrator, chainId, adapter);
    }

    function isRecvAdapterEnabledForChain(address integrator, uint16 chainId, address adapter)
        public
        view
        returns (bool)
    {
        return _isRecvAdapterEnabledForChainWithCheck(integrator, chainId, adapter);
    }
}

contract AdapterRegistryTest is Test {
    ConcreteAdapterRegistry public adapterRegistry;
    address integrator1 = address(0x1);
    address integrator2 = address(0x2);
    address zeroAdapter = address(0);
    address sendAdapter = address(0x123);
    address recvAdapter = address(0x234);
    uint16 zeroChain = 0;
    uint16 chain = 2;
    uint16 wrongChain = 3;

    function setUp() public {
        adapterRegistry = new ConcreteAdapterRegistry();
    }

    function test1() public view {
        assertEq(adapterRegistry.getAdapters(integrator1).length, 0);
        assertEq(adapterRegistry.getAdapters(integrator2).length, 0);
    }

    function test2() public {
        address me = address(this);
        // Send side
        assertEq(adapterRegistry.getAdapters(me).length, 0);
        adapterRegistry.addAdapter(me, sendAdapter);

        // Recv side
        // An Adapter was registered on the send side
        assertEq(adapterRegistry.getAdapters(me).length, 1);
        uint8 index = adapterRegistry.addAdapter(me, recvAdapter);
        require(index == 1, "Invalid index");
        address adapter = adapterRegistry.getAdapterByIndex(me, index);
        require(adapter == recvAdapter, "Invalid adapter");
    }

    function test3() public {
        // Need to add adapter, then enable it, then disable it
        address me = address(this);
        // Send side
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.NonRegisteredAdapter.selector, sendAdapter));
        adapterRegistry.disableSendAdapter(me, chain, sendAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidAdapterZeroAddress.selector));
        adapterRegistry.addAdapter(me, zeroAdapter);
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 0, "S1");
        adapterRegistry.addAdapter(me, sendAdapter);
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 1, "S3");
        // assertEq(adapterRegistry.getSendAdapterInfos(integrator1).length, 1);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidChain.selector, zeroChain));
        adapterRegistry.disableSendAdapter(me, zeroChain, sendAdapter);
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 1, "S5");
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyDisabled.selector, sendAdapter));
        adapterRegistry.disableSendAdapter(me, chain, sendAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidChain.selector, zeroChain));
        adapterRegistry.enableSendAdapter(me, zeroChain, sendAdapter);
        adapterRegistry.enableSendAdapter(me, chain, sendAdapter);
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 1, "S7");
        adapterRegistry.disableSendAdapter(me, chain, sendAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyDisabled.selector, sendAdapter));
        adapterRegistry.disableSendAdapter(me, chain, sendAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidAdapterZeroAddress.selector));
        adapterRegistry.disableSendAdapter(me, chain, zeroAdapter);
        // assertEq(adapterRegistry.getSendAdapterInfos(integrator1).length, 0);

        // Recv side
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.NonRegisteredAdapter.selector, recvAdapter));
        adapterRegistry.disableRecvAdapter(me, chain, recvAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidAdapterZeroAddress.selector));
        adapterRegistry.addAdapter(me, zeroAdapter);
        // Carry over from send side test
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 1, "R1");
        adapterRegistry.addAdapter(me, recvAdapter);
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 2, "R3");
        // assertEq(adapterRegistry.getRecvAdapterInfos(integrator1).length, 1);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidChain.selector, zeroChain));
        adapterRegistry.disableRecvAdapter(me, zeroChain, recvAdapter);
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 2, "R5");
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyDisabled.selector, recvAdapter));
        adapterRegistry.disableRecvAdapter(me, chain, recvAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidChain.selector, zeroChain));
        adapterRegistry.enableRecvAdapter(me, zeroChain, recvAdapter);
        adapterRegistry.enableRecvAdapter(me, chain, recvAdapter);
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 2, "R7");
        adapterRegistry.disableRecvAdapter(me, chain, recvAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyDisabled.selector, recvAdapter));
        adapterRegistry.disableRecvAdapter(me, chain, recvAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidAdapterZeroAddress.selector));
        adapterRegistry.disableRecvAdapter(me, chain, zeroAdapter);
        // assertEq(adapterRegistry.getRecvAdapterInfos(integrator1).length, 0);
    }

    function test4() public {
        // Send side
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(integrator1).length, 0);
        assertEq(adapterRegistry.getEnabledSendAdaptersBitmapForChain(integrator1, chain).length, 0);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidChain.selector, zeroChain));
        assertEq(adapterRegistry.getEnabledSendAdaptersBitmapForChain(integrator1, zeroChain).length, 0);

        // Recv side
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(integrator1).length, 0);
        assertEq(adapterRegistry.getEnabledRecvAdaptersBitmapForChain(integrator1, chain), 0);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidChain.selector, zeroChain));
        assertEq(adapterRegistry.getEnabledRecvAdaptersBitmapForChain(integrator1, zeroChain), 0);
    }

    // This is a redudant test, as the previous tests already cover this
    function test5() public view {
        // Send side
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(integrator1).length, 0);

        // Recv side
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(integrator1).length, 0);
    }

    function test7() public {
        address me = address(this);
        // Send side
        address sAdapter = address(0x456);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.NonRegisteredAdapter.selector, sAdapter));
        adapterRegistry.enableSendAdapter(me, chain, sAdapter);

        // Recv side
        address rAdapter = address(0x567);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.NonRegisteredAdapter.selector, rAdapter));
        adapterRegistry.enableRecvAdapter(me, chain, rAdapter);
    }

    function test8() public {
        uint16 zeroChainId = 0;
        uint16 chainId = 3;
        address me = address(this);

        // Send side
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidAdapterZeroAddress.selector));
        adapterRegistry.enableSendAdapter(me, chainId, zeroAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidAdapterZeroAddress.selector));
        adapterRegistry.isSendAdapterEnabledForChain(me, chainId, zeroAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidChain.selector, zeroChainId));
        adapterRegistry.isSendAdapterEnabledForChain(me, zeroChainId, me);

        // // Recv side
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidAdapterZeroAddress.selector));
        adapterRegistry.enableRecvAdapter(me, chainId, zeroAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidAdapterZeroAddress.selector));
        adapterRegistry.isRecvAdapterEnabledForChain(me, chainId, zeroAdapter);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidChain.selector, zeroChainId));
        adapterRegistry.isRecvAdapterEnabledForChain(me, zeroChainId, me);
    }

    function test9() public {
        uint16 chainId = 4;
        address me = address(this);

        // Send side
        address sAdapter = address(0x345);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.NonRegisteredAdapter.selector, sAdapter));
        require(adapterRegistry.isSendAdapterEnabledForChain(me, chainId, sAdapter) == false, "S1");
        adapterRegistry.addAdapter(me, sAdapter);
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 1, "S2");
        adapterRegistry.enableSendAdapter(me, chainId, sAdapter);
        bool enabled = adapterRegistry.isSendAdapterEnabledForChain(me, chainId, sAdapter);
        require(enabled == true, "S4");
        adapterRegistry.enableSendAdapter(me, chain, sAdapter);

        // Recv side
        address rAdapter = address(0x453);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.NonRegisteredAdapter.selector, rAdapter));
        require(adapterRegistry.isRecvAdapterEnabledForChain(me, chainId, rAdapter) == false, "R1");
        adapterRegistry.addAdapter(me, rAdapter);
        require(adapterRegistry.getRegisteredAdaptersStorage(me).length == 2, "R2");
        adapterRegistry.enableRecvAdapter(me, chainId, rAdapter);
        enabled = adapterRegistry.isRecvAdapterEnabledForChain(me, chainId, rAdapter);
        require(enabled == true, "R4");
        adapterRegistry.enableRecvAdapter(me, chain, rAdapter);
    }

    function test10() public {
        address me = address(this);
        uint8 maxAdapters = adapterRegistry.maxAdapters();

        // Send side
        for (uint8 i = 0; i < maxAdapters; i++) {
            adapterRegistry.addAdapter(me, address(uint160(i + 1)));
        }
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(me).length, maxAdapters);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.TooManyAdapters.selector));
        adapterRegistry.addAdapter(me, address(0x111));
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(me).length, maxAdapters);
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(me).length, maxAdapters);
        for (uint8 i = 0; i < maxAdapters; i++) {
            adapterRegistry.enableSendAdapter(me, chain, address(uint160(i + 1)));
        }
        adapterRegistry.disableSendAdapter(me, chain, address(uint160(30)));
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyDisabled.selector, address(uint160(30))));
        adapterRegistry.disableSendAdapter(me, chain, address(uint160(30)));
        adapterRegistry.getSendAdaptersByChain(me, chain);
    }

    function test11() public {
        address me = address(this);
        uint8 maxAdapters = adapterRegistry.maxAdapters();

        // Recv side
        for (uint8 i = 0; i < maxAdapters; i++) {
            adapterRegistry.addAdapter(me, address(uint160(i + 1)));
        }
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(me).length, maxAdapters);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.TooManyAdapters.selector));
        adapterRegistry.addAdapter(me, address(0x111));
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(me).length, maxAdapters);
        adapterRegistry.enableRecvAdapter(me, chain, address(0x1));
        adapterRegistry.enableRecvAdapter(me, chain, address(0x2));
        adapterRegistry.disableRecvAdapter(me, chain, address(0x2));
        adapterRegistry.disableRecvAdapter(me, chain, address(0x1));
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyDisabled.selector, address(0x1)));
        adapterRegistry.disableSendAdapter(me, chain, address(0x1));
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(me).length, maxAdapters);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.TooManyAdapters.selector));
        adapterRegistry.addAdapter(me, address(0x111));
    }

    function test_getSendAdaptersByChain() public {
        address me = address(this);
        uint16 chain1 = 1;
        uint16 chain2 = 2;
        address adapter1 = address(0x1); // enabled, chain 1
        address adapter2 = address(0x2); // enabled, chain 1
        address adapter3 = address(0x3); // enabled, chain 2
        address adapter4 = address(0x4); // disabled, chain 2

        adapterRegistry.addAdapter(me, adapter1);
        adapterRegistry.enableSendAdapter(me, chain1, adapter1);
        adapterRegistry.addAdapter(me, adapter2);
        adapterRegistry.enableSendAdapter(me, chain1, adapter2);
        adapterRegistry.addAdapter(me, adapter3);
        adapterRegistry.enableSendAdapter(me, chain2, adapter3);
        adapterRegistry.addAdapter(me, adapter4);
        address[] memory chain1Addrs = adapterRegistry.getSendAdaptersByChain(me, chain1);
        require(chain1Addrs.length == 2, "Wrong number of adapters enabled on chain one");
        address[] memory chain2Addrs = adapterRegistry.getSendAdaptersByChain(me, chain2);
        require(chain2Addrs.length == 1, "Wrong number of adapters enabled on chain two");
        adapterRegistry.enableSendAdapter(me, chain2, adapter4);
        adapterRegistry.disableSendAdapter(me, chain2, adapter3);
        require(chain2Addrs.length == 1, "Wrong number of adapters enabled on chain two");
    }

    function test_getRecvAdaptersByChain() public {
        address me = address(this);
        uint16 chain1 = 1;
        uint16 chain2 = 2;
        address adapter1 = address(0x1); // enabled, chain 1
        address adapter2 = address(0x2); // enabled, chain 1
        address adapter3 = address(0x3); // enabled, chain 2
        address adapter4 = address(0x4); // disabled, chain 2

        adapterRegistry.addAdapter(me, adapter1);
        adapterRegistry.enableRecvAdapter(me, chain1, adapter1);
        adapterRegistry.addAdapter(me, adapter2);
        adapterRegistry.enableRecvAdapter(me, chain1, adapter2);
        adapterRegistry.addAdapter(me, adapter3);
        adapterRegistry.enableRecvAdapter(me, chain2, adapter3);
        adapterRegistry.addAdapter(me, adapter4);
        address[] memory chain1Addrs = adapterRegistry.getRecvAdaptersByChain(me, chain1);
        require(chain1Addrs.length == 2, "Wrong number of adapters enabled on chain one");
        address[] memory chain2Addrs = adapterRegistry.getRecvAdaptersByChain(me, chain2);
        require(chain2Addrs.length == 1, "Wrong number of adapters enabled on chain two");
        adapterRegistry.enableRecvAdapter(me, chain2, adapter4);
        adapterRegistry.disableRecvAdapter(me, chain2, adapter3);
        require(chain2Addrs.length == 1, "Wrong number of adapters enabled on chain two");
    }

    function test_recvPerformance() public {
        address me = address(this);
        uint8 maxAdapters = adapterRegistry.maxAdapters();

        // Recv side
        for (uint8 i = 0; i < maxAdapters; i++) {
            adapterRegistry.addAdapter(me, address(uint160(i + 1)));
        }
        assertEq(adapterRegistry.getRegisteredAdaptersStorage(me).length, maxAdapters);
        for (uint8 i = 0; i < maxAdapters; i++) {
            adapterRegistry.enableRecvAdapter(me, chain, address(uint160(i + 1)));
        }
        address[] memory chainAddrs = adapterRegistry.getRecvAdaptersByChain(me, chain);
        require(chainAddrs.length == maxAdapters, "Wrong number of adapters enabled on chain one");
        address[] memory chain2Addrs = adapterRegistry.getRecvAdaptersByChain(me, wrongChain);
        require(chain2Addrs.length == 0, "Wrong number of adapters enabled on chain two");
    }

    function test_getAdapterIndex() public {
        address me = address(this);

        // Add some adapters
        adapterRegistry.addAdapter(me, address(0x1));
        adapterRegistry.addAdapter(me, address(0x2));
        adapterRegistry.addAdapter(me, address(0x3));
        require(adapterRegistry.getAdapters(me).length == 3, "Invalid number of adapters");
        require(adapterRegistry.getAdapterIndex(me, address(0x1)) == 0, "Invalid index");
        require(adapterRegistry.getAdapterIndex(me, address(0x2)) == 1, "Invalid index");
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.NonRegisteredAdapter.selector, address(0x4)));
        adapterRegistry.getAdapterIndex(me, address(0x4));
    }

    function test_maxAdapters() public view {
        assertEq(adapterRegistry.maxAdapters(), 128);
    }
}
