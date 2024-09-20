// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "../src/libraries/UniversalAddress.sol";
import {Router} from "../src/Router.sol";
import {TransceiverRegistry} from "../src/TransceiverRegistry.sol";
import {ITransceiver} from "../src/interfaces/ITransceiver.sol";

contract RouterImpl is Router {
// function getDelegate(address delegator) public view returns (address) {
//     return _getDelegateStorage()[delegator];
// }

// function registerDelegate(address delegate) public {
//     _registerDelegate(delegate);
// }
}

// This contract does send/receive operations
contract Integrator {
    RouterImpl public router;
    address myAdmin;

    constructor(address _router) {
        router = RouterImpl(_router);
    }

    function setMeAsAdmin(address admin) public {
        myAdmin = admin;
    }

    function registerWithRouter() public {
        router.registerAdmin(myAdmin);
    }

    function sendMessage(
        uint16 recipientChain,
        UniversalAddress recipientAddress,
        address refundAddress,
        bytes32 payloadHash
    ) public payable returns (uint64) {
        return router.sendMessage(recipientChain, recipientAddress, refundAddress, payloadHash);
    }
}

// This contract can only do transceiver operations
contract Admin {
    address public integrator;
    RouterImpl public router;

    constructor(address _integrator, address _router) {
        integrator = _integrator;
        router = RouterImpl(_router);
    }

    function requestAdmin() public {
        Integrator(integrator).setMeAsAdmin(address(this));
    }

    function setSendTransceiver(address transceiver, uint16 chain) public {
        router.setSendTransceiver(integrator, transceiver, chain);
    }

    function setRecvTransceiver(address transceiver, uint16 chain) public {
        router.setRecvTransceiver(integrator, transceiver, chain);
    }
}

contract TransceiverImpl is ITransceiver {
    function getTransceiverType() public pure override returns (string memory) {
        return "test";
    }

    function quoteDeliveryPrice(uint16 /*recipientChain*/ ) public pure override returns (uint256) {
        return 0;
    }

    function sendMessage(
        uint16 recipientChain,
        bytes32 messageHash,
        UniversalAddress recipientAddress,
        bytes32 refundAddress
    ) public payable override {
        // Do nothing
    }
}

contract RouterTest is Test {
    RouterImpl public router;
    TransceiverImpl public transceiverImpl;

    address userA = address(0x123);
    address userB = address(0x456);
    address refundAddr = address(0x789);
    bytes32 messageHash = keccak256("hello, world");

    function setUp() public {
        router = new RouterImpl();
        transceiverImpl = new TransceiverImpl();
    }

    function test_setSendTransceiver() public {
        Integrator integrator = new Integrator(address(router));
        Admin admin = new Admin(address(integrator), address(router));
        Admin imposter = new Admin(address(integrator), address(router));
        address transceiver1 = address(0x111);
        uint16 chain = 2;

        admin.requestAdmin();
        integrator.registerWithRouter();
        admin.setSendTransceiver(transceiver1, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyEnabled.selector, transceiver1));
        admin.setSendTransceiver(transceiver1, chain);

        vm.expectRevert(abi.encodeWithSelector(Router.CallerNotAdmin.selector));
        imposter.setSendTransceiver(transceiver1, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyEnabled.selector, transceiver1));
        admin.setSendTransceiver(transceiver1, chain);
    }

    function test_setRecvTransceiver() public {
        Integrator integrator = new Integrator(address(router));
        Admin admin = new Admin(address(integrator), address(router));
        Admin imposter = new Admin(address(integrator), address(router));
        address transceiver1 = address(0x111);
        uint16 chain = 2;

        admin.requestAdmin();
        integrator.registerWithRouter();
        admin.setRecvTransceiver(transceiver1, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyEnabled.selector, transceiver1));
        admin.setRecvTransceiver(transceiver1, chain);

        vm.expectRevert(abi.encodeWithSelector(Router.CallerNotAdmin.selector));
        imposter.setRecvTransceiver(transceiver1, chain);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyEnabled.selector, transceiver1));
        admin.setRecvTransceiver(transceiver1, chain);
    }

    function test_sendMessageIncrementsSequence() public {
        Integrator integrator = new Integrator(address(router));
        Admin admin = new Admin(address(integrator), address(router));
        address transceiver1 = address(0x111);
        uint16 chain = 2;
        admin.requestAdmin();
        integrator.registerWithRouter();
        admin.setSendTransceiver(transceiver1, chain);
        assertEq(router.nextMessageSequence(address(integrator)), 0);
        // Send inital message from userA, going from unset to 1
        // vm.startPrank(userA);
        // vm.expectRevert(abi.encodeWithSelector(Router.TransceiverNotEnabled.selector));
        // router.sendMessage(1, UniversalAddressLibrary.fromAddress(userB), refundAddr, messageHash);
        // assertEq(router.nextMessageSequence(userA), 0);
        // address me = address(this);
        // transceiverRegistry.registerAdmin(me);
        // // Send additional message from userA, incrementing the existing sequence
        // router.sendMessage(1, UniversalAddressLibrary.fromAddress(userB), refundAddr, messageHash);
        // assertEq(router.nextMessageSequence(userA), 2);
    }

    function testFuzz_sendMessage(address user) public {
        // uint16 chainId = 2;
        vm.startPrank(user);
        // Register a transceiver
        // Integrator integrator = new Integrator(address(router));
        // uint64 beforeSequence = router.nextMessageSequence(address(this));
        // address transceiver = address(transceiverImpl);
        // integrator.setMeAsDelegate();
        // uint8 index = router.setSendTransceiver(transceiver, chainId);
        // assert(index == 0);
        // address[] memory enabledTransceivers = router.getSendTransceiversByChain(address(this), chainId);
        // assert(enabledTransceivers.length == 1);
        // router.sendMessage(chainId, UniversalAddressLibrary.fromAddress(user), refundAddr, messageHash);
        // assertEq(router.nextMessageSequence(address(this)), beforeSequence + 1);
    }

    function testFuzz_receiveMessage(address user) public {
        // uint16 chainId = 2;
        vm.startPrank(user);
        // Integrator integrator = new Integrator(address(router));
        // address transceiver = address(0x111);
        // integrator.setRecvTransceiver(transceiver, chainId);
        // address[] memory enabledTransceivers = router.getRecvTransceiversByChain(address(integrator), chainId);
        // assert(enabledTransceivers.length == 1);
        // router.receiveMessage(1, UniversalAddressLibrary.fromAddress(user), refundAddr, messageHash);
    }

    function testFuzz_attestMessage(address user) public {
        // uint16 srcChain = 2;
        // uint16 dstChain = 3;
        // uint64 sequence = 1;
        // bytes32 payloadHash = keccak256("hello, world");
        // sourceAddress = UniversalAddressLibrary.fromAddress(user);
        // destinationAddress = UniversalAddressLibrary.fromAddress(user);
        // vm.startPrank(user);
        // Integrator integrator = new Integrator(address(router));
        // address transceiver = address(0x111);
        // integrator.setRecvTransceiver(transceiver, dstChain);
        // address[] memory enabledTransceivers = router.getRecvTransceiversByChain(address(integrator), dstChain);
        // assert(enabledTransceivers.length == 1);
        // router.attestMessage(
        //     srcChain,
        //     UniversalAddressLibrary.fromAddress(user),
        //     sequence,
        //     dstChain,
        //     UniversalAddressLibrary.fromAddress(user),
        //     payloadHash
        // );
    }
}
