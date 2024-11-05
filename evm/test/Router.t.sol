// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "forge-std/console.sol";
import "../src/libraries/UniversalAddress.sol";
import {Router} from "../src/Router.sol";
import {TransceiverRegistry} from "../src/TransceiverRegistry.sol";
import {ITransceiver} from "../src/interfaces/ITransceiver.sol";

contract RouterImpl is Router {
    uint16 public constant OurChainId = 0x2714;

    constructor() Router(OurChainId) {}
}

// This contract does send/receive operations
contract Integrator {
    RouterImpl public router;
    address myAdmin;

    constructor(address _router) {
        router = RouterImpl(_router);
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
}

contract TransceiverImpl is ITransceiver {
    uint256 _deliveryPrice = 0;
    //======================= Interface =================================================
    // add this to be excluded from coverage report

    function test() public {}

    function getTransceiverType() public pure override returns (string memory) {
        return "test";
    }

    function quoteDeliveryPrice(uint16 /*recipientChain*/ ) public view override returns (uint256) {
        return _deliveryPrice;
    }

    function setDeliveryPrice(uint256 price) public {
        _deliveryPrice = price;
    }

    function sendMessage(
        UniversalAddress, // sourceAddress,
        uint64, // sequence,
        uint16, // recipientChain,
        UniversalAddress, // recipientAddress,
        bytes32, // payloadHash,
        address // refundAddress
    ) public payable override {
        messagesSent += 1;
    }

    //======================= Implementation =================================================

    uint256 public messagesSent;

    function getMessagesSent() public view returns (uint256) {
        return messagesSent;
    }
}

contract RouterTest is Test {
    uint16 constant OurChainId = 0x2714;

    RouterImpl public router;
    TransceiverImpl public transceiverImpl;

    address userA = address(0x123);
    address userB = address(0x456);
    address refundAddr = address(0x789);
    bytes32 payloadHash = keccak256("hello, world");

    function setUp() public {
        router = new RouterImpl();
        transceiverImpl = new TransceiverImpl();
    }

    function test_register() public {
        address integrator = address(new Integrator(address(router)));
        vm.startPrank(integrator);

        // Can't update the admin until we've set it.
        vm.expectRevert(abi.encodeWithSelector(Router.IntegratorNotRegistered.selector));
        router.updateAdmin(integrator, address(0));

        // Can't register admin of zero.
        vm.expectRevert(abi.encodeWithSelector(Router.InvalidAdminZeroAddress.selector));
        router.register(address(0));

        // But a real address should work.
        Admin admin = new Admin(integrator, address(router));
        vm.expectEmit(true, true, false, true);
        emit Router.IntegratorRegistered(address(integrator), address(admin));
        router.register(address(admin));
        require(router.getAdmin(integrator) == address(admin), "admin address is wrong");

        // Can't register twice.
        vm.expectRevert(abi.encodeWithSelector(Router.IntegratorAlreadyRegistered.selector));
        router.register(address(admin));

        // Test updateAdmin().
        Admin newAdmin = new Admin(integrator, address(router));

        // Only the admin can update. The integrator can't.
        vm.expectRevert(abi.encodeWithSelector(Router.CallerNotAuthorized.selector));
        router.updateAdmin(integrator, address(newAdmin));

        vm.startPrank(address(admin));

        // We can set the admin to a new address.
        vm.expectEmit(true, true, false, true);
        emit Router.AdminUpdated(address(integrator), address(admin), address(newAdmin));
        router.updateAdmin(integrator, address(newAdmin));
        require(router.getAdmin(integrator) == address(newAdmin), "failed to update admin address");

        // And the old admin should no longer be able to update.
        vm.expectRevert(abi.encodeWithSelector(Router.CallerNotAuthorized.selector));
        router.updateAdmin(integrator, address(admin));

        // But the new admin can.
        vm.startPrank(address(newAdmin));
        Admin newerAdmin = new Admin(integrator, address(router));
        vm.expectEmit(true, true, false, true);
        emit Router.AdminUpdated(address(integrator), address(newAdmin), address(newerAdmin));
        router.updateAdmin(integrator, address(newerAdmin));
        require(router.getAdmin(integrator) == address(newerAdmin), "failed to update admin address");

        // Cannot claim if there is no transfer in progress.
        vm.expectRevert(abi.encodeWithSelector(Router.NoAdminTransferInProgress.selector));
        router.claimAdmin(integrator);

        vm.startPrank(address(newerAdmin));
        // Two step update to first admin.
        vm.expectRevert(abi.encodeWithSelector(Router.InvalidAdminZeroAddress.selector));
        router.transferAdmin(integrator, address(0));
        vm.expectEmit(true, true, false, true);
        emit Router.AdminUpdateRequested(address(integrator), address(newerAdmin), address(admin));
        router.transferAdmin(integrator, address(admin));
        require(router.getAdmin(integrator) == address(newerAdmin), "updated admin address too early");
        require(router.getPendingAdmin(integrator) == address(admin), "incorrect pending address");
        vm.expectRevert(abi.encodeWithSelector(Router.AdminTransferInProgress.selector));
        router.transferAdmin(integrator, address(admin));
        vm.expectRevert(abi.encodeWithSelector(Router.AdminTransferInProgress.selector));
        router.updateAdmin(integrator, address(newerAdmin));
        vm.expectRevert(abi.encodeWithSelector(Router.AdminTransferInProgress.selector));
        router.discardAdmin(integrator);
        // Test that the initiator can cancel the update.
        vm.expectEmit(true, true, false, true);
        emit Router.AdminUpdated(address(integrator), address(newerAdmin), address(newerAdmin));
        router.claimAdmin(integrator);
        require(router.getAdmin(integrator) == address(newerAdmin), "failed to update to first admin address");

        // Two step update to new admin.
        vm.expectEmit(true, true, false, true);
        emit Router.AdminUpdateRequested(address(integrator), address(newerAdmin), address(newAdmin));
        router.transferAdmin(integrator, address(newAdmin));
        require(router.getAdmin(integrator) == address(newerAdmin), "updated admin address too early");
        vm.startPrank(address(admin));
        vm.expectRevert(abi.encodeWithSelector(Router.CallerNotAuthorized.selector));
        router.claimAdmin(integrator);
        // Test that the new admin can claim the update.
        vm.startPrank(address(newAdmin));
        vm.expectEmit(true, true, false, true);
        emit Router.AdminUpdated(address(integrator), address(newerAdmin), address(newAdmin));
        router.claimAdmin(integrator);
        require(router.getAdmin(integrator) == address(newAdmin), "failed to update to new admin address");

        // One step update to zero.
        vm.expectRevert(abi.encodeWithSelector(Router.InvalidAdminZeroAddress.selector));
        router.updateAdmin(integrator, address(0));

        vm.expectEmit(true, true, false, true);
        emit Router.AdminUpdated(address(integrator), address(newAdmin), address(0));
        router.discardAdmin(integrator);
    }

    function test_addSendTransceiver() public {
        address integrator = address(new Integrator(address(router)));
        address admin = address(new Admin(integrator, address(router)));
        address imposter = address(new Admin(integrator, address(router)));
        TransceiverImpl transceiver1 = new TransceiverImpl();
        TransceiverImpl transceiver2 = new TransceiverImpl();
        TransceiverImpl transceiver3 = new TransceiverImpl();
        address taddr1 = address(transceiver1);
        address taddr2 = address(transceiver2);
        address taddr3 = address(transceiver3);
        vm.startPrank(integrator);

        // Can't enable a transceiver until we've set the admin.
        vm.expectRevert(abi.encodeWithSelector(Router.IntegratorNotRegistered.selector));
        router.addTransceiver(integrator, taddr1);

        // Register the integrator and set the admin.
        vm.expectEmit(true, true, false, true);
        emit Router.IntegratorRegistered(address(integrator), address(admin));
        router.register(admin);

        // The admin can add a transceiver.
        vm.startPrank(admin);
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.TransceiverAdded(integrator, taddr1, 1);
        router.addTransceiver(integrator, taddr1);

        // Others cannot add a transceiver.
        vm.startPrank(imposter);
        vm.expectRevert(abi.encodeWithSelector(Router.CallerNotAuthorized.selector));
        router.addTransceiver(integrator, taddr1);

        // Can't register the transceiver twice.
        vm.startPrank(admin);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyRegistered.selector, taddr1));
        router.addTransceiver(integrator, taddr1);
        // Can't enable the transceiver twice.
        router.enableSendTransceiver(integrator, 1, taddr1);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyEnabled.selector, taddr1));
        router.enableSendTransceiver(integrator, 1, taddr1);

        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.TransceiverAdded(integrator, taddr2, 2);
        router.addTransceiver(integrator, taddr2);
        address[] memory transceivers = router.getSendTransceiversByChain(integrator, 1);
        require(transceivers.length == 1, "Wrong number of transceivers enabled on chain one, should be 1");
        // Enable another transceiver on chain one and one on chain two.
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.SendTransceiverEnabledForChain(integrator, 1, taddr2);
        router.enableSendTransceiver(integrator, 1, taddr2);
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.TransceiverAdded(integrator, taddr3, 3);
        router.addTransceiver(integrator, taddr3);
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.SendTransceiverEnabledForChain(integrator, 2, taddr3);
        router.enableSendTransceiver(integrator, 2, taddr3);

        // And verify they got set properly.
        transceivers = router.getSendTransceiversByChain(integrator, 1);
        require(transceivers.length == 2, "Wrong number of transceivers enabled on chain one");
        require(transceivers[0] == taddr1, "Wrong transceiver one on chain one");
        require(transceivers[1] == taddr2, "Wrong transceiver two on chain one");
        transceivers = router.getSendTransceiversByChain(integrator, 2);
        require(transceivers.length == 1, "Wrong number of transceivers enabled on chain two");
        require(transceivers[0] == taddr3, "Wrong transceiver one on chain two");
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.SendTransceiverDisabledForChain(integrator, 2, taddr3);
        router.disableSendTransceiver(integrator, 2, taddr3);
        require(transceivers.length == 1, "Wrong number of transceivers enabled on chain two");
    }

    function test_addRecvTransceiver() public {
        address integrator = address(new Integrator(address(router)));
        address admin = address(new Admin(integrator, address(router)));
        address imposter = address(new Admin(integrator, address(router)));
        TransceiverImpl transceiver1 = new TransceiverImpl();
        TransceiverImpl transceiver2 = new TransceiverImpl();
        TransceiverImpl transceiver3 = new TransceiverImpl();
        address taddr1 = address(transceiver1);
        address taddr2 = address(transceiver2);
        address taddr3 = address(transceiver3);
        vm.startPrank(integrator);

        // Can't enable a transceiver until we've set the admin.
        vm.expectRevert(abi.encodeWithSelector(Router.IntegratorNotRegistered.selector));
        router.addTransceiver(integrator, taddr1);

        // Register the integrator and set the admin.
        vm.expectEmit(true, true, false, true);
        emit Router.IntegratorRegistered(address(integrator), address(admin));
        router.register(admin);

        // The admin can add a transceiver.
        vm.startPrank(admin);
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.TransceiverAdded(integrator, taddr1, 1);
        router.addTransceiver(integrator, taddr1);

        // Others cannot add a transceiver.
        vm.startPrank(imposter);
        vm.expectRevert(abi.encodeWithSelector(Router.CallerNotAuthorized.selector));
        router.addTransceiver(integrator, taddr1);

        // Can't register the transceiver twice.
        vm.startPrank(admin);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyRegistered.selector, taddr1));
        router.addTransceiver(integrator, taddr1);
        // Can't enable the transceiver twice.
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.RecvTransceiverEnabledForChain(integrator, 1, taddr1);
        router.enableRecvTransceiver(integrator, 1, taddr1);
        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.TransceiverAlreadyEnabled.selector, taddr1));
        router.enableRecvTransceiver(integrator, 1, taddr1);

        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.TransceiverAdded(integrator, taddr2, 2);
        router.addTransceiver(integrator, taddr2);
        address[] memory transceivers = router.getRecvTransceiversByChain(integrator, 1);
        require(transceivers.length == 1, "Wrong number of transceivers enabled on chain one, should be 1");
        // Enable another transceiver on chain one and one on chain two.
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.RecvTransceiverEnabledForChain(integrator, 1, taddr2);
        router.enableRecvTransceiver(integrator, 1, taddr2);
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.TransceiverAdded(integrator, taddr3, 3);
        router.addTransceiver(integrator, taddr3);
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.RecvTransceiverEnabledForChain(integrator, 2, taddr3);
        router.enableRecvTransceiver(integrator, 2, taddr3);

        // And verify they got set properly.
        transceivers = router.getRecvTransceiversByChain(integrator, 1);
        require(transceivers.length == 2, "Wrong number of transceivers enabled on chain one");
        require(transceivers[0] == taddr1, "Wrong transceiver one on chain one");
        require(transceivers[1] == taddr2, "Wrong transceiver two on chain one");
        transceivers = router.getRecvTransceiversByChain(integrator, 2);
        require(transceivers.length == 1, "Wrong number of transceivers enabled on chain two");
        require(transceivers[0] == taddr3, "Wrong transceiver one on chain two");
    }

    function test_sendMessage() public {
        address integrator = address(new Integrator(address(router)));
        address admin = address(new Admin(integrator, address(router)));
        uint16 chain = 2;
        uint16 zeroChain = 0;
        TransceiverImpl transceiver1 = new TransceiverImpl();
        TransceiverImpl transceiver2 = new TransceiverImpl();
        TransceiverImpl transceiver3 = new TransceiverImpl();
        vm.startPrank(integrator);
        vm.expectEmit(true, true, false, true);
        emit Router.IntegratorRegistered(address(integrator), address(admin));
        router.register(admin);

        // Sending with no transceivers should revert.
        vm.startPrank(integrator);
        vm.expectRevert(abi.encodeWithSelector(Router.TransceiverNotEnabled.selector));
        uint64 sequence = router.sendMessage(2, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);

        // Now enable some transceivers.
        vm.startPrank(admin);
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.TransceiverAdded(integrator, address(transceiver1), 1);
        router.addTransceiver(integrator, address(transceiver1));
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.SendTransceiverEnabledForChain(integrator, 2, address(transceiver1));
        router.enableSendTransceiver(integrator, 2, address(transceiver1));
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.TransceiverAdded(integrator, address(transceiver2), 2);
        router.addTransceiver(integrator, address(transceiver2));
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.SendTransceiverEnabledForChain(integrator, 2, address(transceiver2));
        router.enableSendTransceiver(integrator, 2, address(transceiver2));
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.TransceiverAdded(integrator, address(transceiver3), 3);
        router.addTransceiver(integrator, address(transceiver3));
        vm.expectEmit(true, true, false, true);
        emit TransceiverRegistry.SendTransceiverEnabledForChain(integrator, 3, address(transceiver3));
        router.enableSendTransceiver(integrator, 3, address(transceiver3));

        // Only an integrator can call send.
        vm.startPrank(userA);
        vm.expectRevert(abi.encodeWithSelector(Router.TransceiverNotEnabled.selector));
        sequence = router.sendMessage(chain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);

        // Send a message on chain two. It should go out on the first two transceivers, but not the third one.
        vm.startPrank(integrator);
        vm.expectEmit(true, true, false, true);
        emit Router.MessageSent(
            router.computeMessageDigest(
                OurChainId,
                UniversalAddressLibrary.fromAddress(integrator),
                sequence,
                chain,
                UniversalAddressLibrary.fromAddress(userA),
                payloadHash
            ),
            UniversalAddressLibrary.fromAddress(integrator),
            sequence,
            UniversalAddressLibrary.fromAddress(userA),
            chain,
            payloadHash
        );
        sequence = router.sendMessage(chain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);
        require(sequence == 0, "Sequence number is wrong");
        require(transceiver1.getMessagesSent() == 1, "Failed to send a message on transceiver 1");
        require(transceiver2.getMessagesSent() == 1, "Failed to send a message on transceiver 2");
        require(transceiver3.getMessagesSent() == 0, "Should not have sent a message on transceiver 3");

        sequence = router.sendMessage(chain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);
        require(sequence == 1, "Second sequence number is wrong");
        require(transceiver1.getMessagesSent() == 2, "Failed to send second message on transceiver 1");
        require(transceiver2.getMessagesSent() == 2, "Failed to send second message on transceiver 2");
        require(transceiver3.getMessagesSent() == 0, "Should not have sent second message on transceiver 3");

        vm.expectRevert(abi.encodeWithSelector(TransceiverRegistry.InvalidChain.selector, zeroChain));
        sequence = router.sendMessage(zeroChain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);
        require(sequence == 0, "Failed sequence number is wrong"); // 0 because of the revert

        sequence = router.sendMessage(chain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);
        require(sequence == 2, "Third sequence number is wrong");
    }

    function test_attestMessage() public {
        UniversalAddress sourceIntegrator = UniversalAddressLibrary.fromAddress(address(userA));
        address integrator = address(new Integrator(address(router)));
        UniversalAddress destIntegrator = UniversalAddressLibrary.fromAddress(address(integrator));
        address admin = address(new Admin(integrator, address(router)));
        TransceiverImpl transceiver1 = new TransceiverImpl();
        TransceiverImpl transceiver2 = new TransceiverImpl();
        TransceiverImpl transceiver3 = new TransceiverImpl();
        uint16 chain = 2;
        uint16 anotherChain = 1;
        vm.startPrank(integrator);
        router.register(admin);

        // Attesting with no transceivers should revert.
        vm.startPrank(integrator);
        vm.expectRevert(abi.encodeWithSelector(Router.TransceiverNotEnabled.selector));
        router.attestMessage(2, sourceIntegrator, 1, OurChainId, destIntegrator, payloadHash);

        // Now enable some transceivers.
        vm.startPrank(admin);
        router.addTransceiver(integrator, address(transceiver1));
        router.enableRecvTransceiver(integrator, chain, address(transceiver1));
        router.addTransceiver(integrator, address(transceiver2));
        router.enableRecvTransceiver(integrator, chain, address(transceiver2));
        router.addTransceiver(integrator, address(transceiver3));
        router.enableRecvTransceiver(integrator, chain + 1, address(transceiver3));

        // Only a transceiver can call attest.
        vm.startPrank(userB);
        vm.expectRevert(abi.encodeWithSelector(Router.TransceiverNotEnabled.selector));
        router.attestMessage(chain, sourceIntegrator, anotherChain, OurChainId, destIntegrator, payloadHash);

        // Attesting a message destined for the wrong chain should revert.
        vm.startPrank(address(transceiver2));
        vm.expectRevert(abi.encodeWithSelector(Router.InvalidDestinationChain.selector));
        router.attestMessage(chain, sourceIntegrator, anotherChain, OurChainId + 1, destIntegrator, payloadHash);

        // This attest should work.
        vm.startPrank(address(transceiver2));
        vm.expectEmit(true, true, false, true);
        emit Router.MessageAttestedTo(
            router.computeMessageDigest(chain, sourceIntegrator, anotherChain, OurChainId, destIntegrator, payloadHash),
            chain,
            sourceIntegrator,
            anotherChain,
            OurChainId,
            destIntegrator,
            payloadHash,
            0x2, // attested bitmap
            UniversalAddressLibrary.fromAddress(address(transceiver2))
        );
        router.attestMessage(chain, sourceIntegrator, anotherChain, OurChainId, destIntegrator, payloadHash);

        // Multiple Attests from same transceiver should revert.
        vm.expectRevert(abi.encodeWithSelector(Router.DuplicateMessageAttestation.selector));
        router.attestMessage(chain, sourceIntegrator, anotherChain, OurChainId, destIntegrator, payloadHash);

        // Receive what we just attested to mark it executed.
        vm.startPrank(integrator);
        vm.expectEmit(true, true, false, true);
        emit Router.MessageReceived(
            router.computeMessageDigest(chain, sourceIntegrator, anotherChain, OurChainId, destIntegrator, payloadHash),
            chain,
            sourceIntegrator,
            anotherChain,
            OurChainId,
            destIntegrator,
            payloadHash,
            0x3, // enabled bitmap
            0x2 // attested bitmap
        );
        router.recvMessage(chain, sourceIntegrator, anotherChain, payloadHash);

        // Attesting after receive should still work on a different transceiver.
        vm.startPrank(address(transceiver1));
        router.attestMessage(chain, sourceIntegrator, anotherChain, OurChainId, destIntegrator, payloadHash);

        // Attesting on a disabled transceiver should revert.
        vm.startPrank(admin);
        router.disableRecvTransceiver(integrator, 2, address(transceiver1));
        vm.startPrank(address(transceiver1));
        vm.expectRevert(abi.encodeWithSelector(Router.TransceiverNotEnabled.selector));
        router.attestMessage(chain, sourceIntegrator, anotherChain, OurChainId, destIntegrator, payloadHash);
    }

    function test_recvMessage() public {
        UniversalAddress sourceIntegrator = UniversalAddressLibrary.fromAddress(address(userA));
        address integrator = address(new Integrator(address(router)));
        UniversalAddress destIntegrator = UniversalAddressLibrary.fromAddress(address(integrator));
        address admin = address(new Admin(integrator, address(router)));
        TransceiverImpl transceiver1 = new TransceiverImpl();
        TransceiverImpl transceiver2 = new TransceiverImpl();
        TransceiverImpl transceiver3 = new TransceiverImpl();
        vm.startPrank(integrator);
        router.register(admin);

        // Receiving with no transceivers should revert.
        vm.startPrank(integrator);
        vm.expectRevert(abi.encodeWithSelector(Router.TransceiverNotEnabled.selector));
        router.recvMessage(2, sourceIntegrator, 1, payloadHash);

        // Now enable some transceivers so we can attest. Receive doesn't use the transceivers.
        vm.startPrank(admin);
        router.addTransceiver(integrator, address(transceiver1));
        router.enableRecvTransceiver(integrator, 2, address(transceiver1));
        router.addTransceiver(integrator, address(transceiver2));
        router.enableRecvTransceiver(integrator, 2, address(transceiver2));
        router.addTransceiver(integrator, address(transceiver3));
        router.enableRecvTransceiver(integrator, 3, address(transceiver3));

        // Only an integrator can call receive.
        vm.startPrank(userB);
        vm.expectRevert(abi.encodeWithSelector(Router.TransceiverNotEnabled.selector));
        router.recvMessage(2, sourceIntegrator, 1, payloadHash);

        // Receiving before there are any attestations should revert.
        vm.startPrank(integrator);
        vm.expectRevert(abi.encodeWithSelector(Router.UnknownMessageAttestation.selector));
        router.recvMessage(2, sourceIntegrator, 1, payloadHash);

        // Attest so we can receive.
        vm.startPrank(address(transceiver2));
        router.attestMessage(2, sourceIntegrator, 1, OurChainId, destIntegrator, payloadHash);

        // This receive should work.
        vm.startPrank(integrator);
        (uint128 enabledBitmap, uint128 attestedBitmap) = router.recvMessage(2, sourceIntegrator, 1, payloadHash);

        // Make sure it return the right bitmaps.
        require(enabledBitmap == 0x03, "Enabled bitmap is wrong");
        require(attestedBitmap == 0x02, "Attested bitmap is wrong");

        // But doing it again should revert.
        vm.expectRevert(abi.encodeWithSelector(Router.AlreadyExecuted.selector));
        router.recvMessage(2, sourceIntegrator, 1, payloadHash);
    }

    function test_getMessageStatus() public {
        UniversalAddress sourceIntegrator = UniversalAddressLibrary.fromAddress(address(userA));
        address integrator = address(new Integrator(address(router)));
        UniversalAddress destIntegrator = UniversalAddressLibrary.fromAddress(address(integrator));
        address admin = address(new Admin(integrator, address(router)));
        TransceiverImpl transceiver1 = new TransceiverImpl();
        TransceiverImpl transceiver2 = new TransceiverImpl();
        TransceiverImpl transceiver3 = new TransceiverImpl();
        vm.startPrank(integrator);
        router.register(admin);

        // Nothing is attested, yet.
        vm.startPrank(integrator);
        (uint128 enabledBitmap, uint128 attestedBitmap, bool executed) =
            router.getMessageStatus(2, sourceIntegrator, 1, destIntegrator, payloadHash);
        require(enabledBitmap == 0, "Enabled bitmap is wrong1");
        require(attestedBitmap == 0, "Attested bitmap is wrong1");
        require(executed == false, "executed flag is wrong1");

        // Now enable some transceivers so we can attest.
        vm.startPrank(admin);
        router.addTransceiver(integrator, address(transceiver1));
        router.enableRecvTransceiver(integrator, 2, address(transceiver1));
        router.addTransceiver(integrator, address(transceiver2));
        router.enableRecvTransceiver(integrator, 2, address(transceiver2));
        router.addTransceiver(integrator, address(transceiver3));
        router.enableRecvTransceiver(integrator, 3, address(transceiver3));

        vm.startPrank(userB);
        (enabledBitmap, attestedBitmap, executed) =
            router.getMessageStatus(2, sourceIntegrator, 1, destIntegrator, payloadHash);
        // 00000011 bitmap = 3 decimal
        require(enabledBitmap == 3, "Enabled bitmap is wrong2");
        require(attestedBitmap == 0, "Attested bitmap is wrong2");
        require(executed == false, "executed flag is wrong2");

        // Should get the same values as above
        vm.startPrank(integrator);
        (enabledBitmap, attestedBitmap, executed) =
            router.getMessageStatus(2, sourceIntegrator, 1, destIntegrator, payloadHash);
        require(enabledBitmap == 3, "Enabled bitmap is wrong4");
        require(attestedBitmap == 0, "Attested bitmap is wrong4");
        require(executed == false, "executed flag is wrong4");

        // Attest
        vm.startPrank(address(transceiver2));
        router.attestMessage(2, sourceIntegrator, 1, OurChainId, destIntegrator, payloadHash);

        // Should now have a non zero value for attested bitmap
        vm.startPrank(integrator);
        (enabledBitmap, attestedBitmap, executed) =
            router.getMessageStatus(2, sourceIntegrator, 1, destIntegrator, payloadHash);
        require(enabledBitmap == 3, "Enabled bitmap is wrong5");
        // 00000010 bitmap = 2 decimal
        require(attestedBitmap == 2, "Attested bitmap is wrong5");
        require(executed == false, "executed flag is wrong5");

        // Test the second version of getMessageStatus.
        (enabledBitmap, attestedBitmap, executed) = router.getMessageStatus(2, sourceIntegrator, 1, payloadHash);
        require(enabledBitmap == 3, "Enabled bitmap is wrong6");
        require(attestedBitmap == 2, "Attested bitmap is wrong6");
        require(executed == false, "executed flag is wrong6");
    }

    function test_realScenario() public {
        uint16 srcChain = 0x2712;
        UniversalAddress srcAddr =
            UniversalAddressLibrary.fromBytes32(0x000000000000000000000000701b575d65f8ecd8de43d423ce4d6a09420da8c0);
        uint64 sequence = 0x0000000000000000;
        uint16 dstChain = 0x2714;
        UniversalAddress dstAddr =
            UniversalAddressLibrary.fromBytes32(0x000000000000000000000000949d2139d42b6122a663d3e62f1582678f1458d7);
        address dstIntegrator = UniversalAddressLibrary.toAddress(dstAddr);

        RouterImpl endpoint = new RouterImpl();
        vm.startPrank(dstIntegrator);
        endpoint.register(dstIntegrator);

        // Register and enable recv transceiver.
        TransceiverImpl recvTrans = new TransceiverImpl();
        endpoint.addTransceiver(dstIntegrator, address(recvTrans));
        endpoint.enableRecvTransceiver(dstIntegrator, srcChain, address(recvTrans));

        // Dest transceiver needs to attest.
        vm.startPrank(address(recvTrans));
        endpoint.attestMessage(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash);

        // Double check the parameters.
        vm.startPrank(dstIntegrator);
        (uint128 enabledBitmap, uint128 attestedBitmap, bool executed) =
            endpoint.getMessageStatus(srcChain, srcAddr, sequence, dstAddr, payloadHash);
        require(enabledBitmap == 0x01, "Enabled bitmap is wrong");
        require(attestedBitmap == 0x01, "Attested bitmap is wrong");
        require(executed == false, "executed flag is wrong");
    }

    function test_execMessage() public {
        address integrator = address(new Integrator(address(router)));
        address admin = address(new Admin(integrator, address(router)));
        TransceiverImpl transceiver1 = new TransceiverImpl();
        uint16 chain1 = 1;
        uint64 sequence = 1;
        uint128 enabledBitmap;
        uint128 attestedBitmap;
        bool executed;

        // Register the integrator and set the admin.
        vm.startPrank(integrator);
        router.register(admin);

        (enabledBitmap, attestedBitmap, executed) = router.getMessageStatus(
            chain1,
            UniversalAddressLibrary.fromAddress(address(transceiver1)),
            sequence,
            UniversalAddressLibrary.fromAddress(address(integrator)),
            payloadHash
        );
        require(executed == false, "executed flag should be false before execMessage");
        router.execMessage(chain1, UniversalAddressLibrary.fromAddress(address(transceiver1)), sequence, payloadHash);
        (enabledBitmap, attestedBitmap, executed) = router.getMessageStatus(
            chain1,
            UniversalAddressLibrary.fromAddress(address(transceiver1)),
            sequence,
            UniversalAddressLibrary.fromAddress(address(integrator)),
            payloadHash
        );
        require(executed == true, "executed flag should be true after execMessage");
        // Second execMessage should revert.
        vm.expectRevert(abi.encodeWithSelector(Router.AlreadyExecuted.selector));
        router.execMessage(chain1, UniversalAddressLibrary.fromAddress(address(transceiver1)), sequence, payloadHash);
    }

    function test_computepayloadHash() public view {
        UniversalAddress sourceIntegrator = UniversalAddressLibrary.fromAddress(address(userA));
        UniversalAddress destIntegrator = UniversalAddressLibrary.fromAddress(address(userB));
        uint16 srcChain = 2;
        uint16 dstChain = 42;
        uint64 sequence = 3;
        bytes32 mypayloadHash =
            router.computeMessageDigest(srcChain, sourceIntegrator, sequence, dstChain, destIntegrator, payloadHash);
        bytes32 expectedHash =
            keccak256(abi.encodePacked(srcChain, sourceIntegrator, sequence, dstChain, destIntegrator, payloadHash));
        require(mypayloadHash == expectedHash, "Message hash is wrong");
        require(
            mypayloadHash == 0xf589999616054a74b876390c4eb6e067da272da5cd313a9657d33ec3cab06760,
            "Message hash literal is wrong"
        );
    }

    function test_quoteDeliveryPrice() public {
        address integrator = address(new Integrator(address(router)));
        address admin = address(new Admin(integrator, address(router)));
        uint16 chain = 2;
        TransceiverImpl transceiver1 = new TransceiverImpl();
        TransceiverImpl transceiver2 = new TransceiverImpl();
        TransceiverImpl transceiver3 = new TransceiverImpl();
        vm.startPrank(integrator);
        router.register(admin);

        // Set the delivery price.
        transceiver1.setDeliveryPrice(100);
        transceiver2.setDeliveryPrice(200);
        transceiver3.setDeliveryPrice(300);

        // Now enable some transceivers.
        vm.startPrank(admin);
        router.addTransceiver(integrator, address(transceiver1));
        router.enableSendTransceiver(integrator, chain, address(transceiver1));
        uint256 price = router.quoteDeliveryPrice(integrator, chain);
        require(price == 100, "Single price is wrong");
        router.addTransceiver(integrator, address(transceiver2));
        router.enableSendTransceiver(integrator, chain, address(transceiver2));
        price = router.quoteDeliveryPrice(integrator, chain);
        require(price == 300, "Double price is wrong");
        router.addTransceiver(integrator, address(transceiver3));
        router.enableSendTransceiver(integrator, 3, address(transceiver3));
        price = router.quoteDeliveryPrice(integrator, chain);
        require(price == 300, "Triple price is wrong");
        vm.startPrank(integrator);
        price = router.quoteDeliveryPrice(chain);
        require(price == 300, "Triple price is wrong");
    }
}
