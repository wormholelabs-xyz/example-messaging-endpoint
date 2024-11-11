// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import "forge-std/console.sol";
import "../src/libraries/UniversalAddress.sol";
import {Endpoint} from "../src/Endpoint.sol";
import {AdapterRegistry} from "../src/AdapterRegistry.sol";
import {IAdapter} from "../src/interfaces/IAdapter.sol";

contract EndpointImpl is Endpoint {
    uint16 public constant OurChainId = 0x2714;

    constructor() Endpoint(OurChainId) {}
}

// This contract does send/receive operations
contract Integrator {
    EndpointImpl public endpoint;
    address myAdmin;

    constructor(address _endpoint) {
        endpoint = EndpointImpl(_endpoint);
    }
}

// This contract can only do adapter operations
contract Admin {
    address public integrator;
    EndpointImpl public endpoint;

    constructor(address _integrator, address _endpoint) {
        integrator = _integrator;
        endpoint = EndpointImpl(_endpoint);
    }
}

contract AdapterImpl is IAdapter {
    uint256 _deliveryPrice = 0;
    //======================= Interface =================================================
    // add this to be excluded from coverage report

    function test() public {}

    function getAdapterType() public pure override returns (string memory) {
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

contract EndpointTest is Test {
    uint16 constant OurChainId = 0x2714;

    EndpointImpl public endpoint;
    AdapterImpl public adapterImpl;

    address userA = address(0x123);
    address userB = address(0x456);
    address refundAddr = address(0x789);
    bytes32 payloadHash = keccak256("hello, world");

    function setUp() public {
        endpoint = new EndpointImpl();
        adapterImpl = new AdapterImpl();
    }

    function test_register() public {
        address integrator = address(new Integrator(address(endpoint)));
        vm.startPrank(integrator);

        // Can't update the admin until we've set it.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.IntegratorNotRegistered.selector));
        endpoint.updateAdmin(integrator, address(0));

        // Can't register admin of zero.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.InvalidAdminZeroAddress.selector));
        endpoint.register(address(0));

        // But a real address should work.
        Admin admin = new Admin(integrator, address(endpoint));
        vm.expectEmit(true, true, false, true);
        emit Endpoint.IntegratorRegistered(address(integrator), address(admin));
        endpoint.register(address(admin));
        require(endpoint.getAdmin(integrator) == address(admin), "admin address is wrong");

        // Can't register twice.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.IntegratorAlreadyRegistered.selector));
        endpoint.register(address(admin));

        // Test updateAdmin().
        Admin newAdmin = new Admin(integrator, address(endpoint));

        // Only the admin can update. The integrator can't.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.CallerNotAuthorized.selector));
        endpoint.updateAdmin(integrator, address(newAdmin));

        vm.startPrank(address(admin));

        // We can set the admin to a new address.
        vm.expectEmit(true, true, false, true);
        emit Endpoint.AdminUpdated(address(integrator), address(admin), address(newAdmin));
        endpoint.updateAdmin(integrator, address(newAdmin));
        require(endpoint.getAdmin(integrator) == address(newAdmin), "failed to update admin address");

        // And the old admin should no longer be able to update.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.CallerNotAuthorized.selector));
        endpoint.updateAdmin(integrator, address(admin));

        // But the new admin can.
        vm.startPrank(address(newAdmin));
        Admin newerAdmin = new Admin(integrator, address(endpoint));
        vm.expectEmit(true, true, false, true);
        emit Endpoint.AdminUpdated(address(integrator), address(newAdmin), address(newerAdmin));
        endpoint.updateAdmin(integrator, address(newerAdmin));
        require(endpoint.getAdmin(integrator) == address(newerAdmin), "failed to update admin address");

        // Cannot claim if there is no transfer in progress.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.NoAdminTransferInProgress.selector));
        endpoint.claimAdmin(integrator);

        vm.startPrank(address(newerAdmin));
        // Two step update to first admin.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.InvalidAdminZeroAddress.selector));
        endpoint.transferAdmin(integrator, address(0));
        vm.expectEmit(true, true, false, true);
        emit Endpoint.AdminUpdateRequested(address(integrator), address(newerAdmin), address(admin));
        endpoint.transferAdmin(integrator, address(admin));
        require(endpoint.getAdmin(integrator) == address(newerAdmin), "updated admin address too early");
        require(endpoint.getPendingAdmin(integrator) == address(admin), "incorrect pending address");
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdminTransferInProgress.selector));
        endpoint.transferAdmin(integrator, address(admin));
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdminTransferInProgress.selector));
        endpoint.updateAdmin(integrator, address(newerAdmin));
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdminTransferInProgress.selector));
        endpoint.discardAdmin(integrator);
        // Test that the initiator can cancel the update.
        vm.expectEmit(true, true, false, true);
        emit Endpoint.AdminUpdated(address(integrator), address(newerAdmin), address(newerAdmin));
        endpoint.claimAdmin(integrator);
        require(endpoint.getAdmin(integrator) == address(newerAdmin), "failed to update to first admin address");

        // Two step update to new admin.
        vm.expectEmit(true, true, false, true);
        emit Endpoint.AdminUpdateRequested(address(integrator), address(newerAdmin), address(newAdmin));
        endpoint.transferAdmin(integrator, address(newAdmin));
        require(endpoint.getAdmin(integrator) == address(newerAdmin), "updated admin address too early");
        vm.startPrank(address(admin));
        vm.expectRevert(abi.encodeWithSelector(Endpoint.CallerNotAuthorized.selector));
        endpoint.claimAdmin(integrator);
        // Test that the new admin can claim the update.
        vm.startPrank(address(newAdmin));
        vm.expectEmit(true, true, false, true);
        emit Endpoint.AdminUpdated(address(integrator), address(newerAdmin), address(newAdmin));
        endpoint.claimAdmin(integrator);
        require(endpoint.getAdmin(integrator) == address(newAdmin), "failed to update to new admin address");

        // One step update to zero.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.InvalidAdminZeroAddress.selector));
        endpoint.updateAdmin(integrator, address(0));

        vm.expectEmit(true, true, false, true);
        emit Endpoint.AdminUpdated(address(integrator), address(newAdmin), address(0));
        endpoint.discardAdmin(integrator);
    }

    function test_addSendAdapter() public {
        address integrator = address(new Integrator(address(endpoint)));
        address admin = address(new Admin(integrator, address(endpoint)));
        address imposter = address(new Admin(integrator, address(endpoint)));
        AdapterImpl adapter1 = new AdapterImpl();
        AdapterImpl adapter2 = new AdapterImpl();
        AdapterImpl adapter3 = new AdapterImpl();
        address taddr1 = address(adapter1);
        address taddr2 = address(adapter2);
        address taddr3 = address(adapter3);
        vm.startPrank(integrator);

        // Can't enable an adapter until we've set the admin.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.IntegratorNotRegistered.selector));
        endpoint.addAdapter(integrator, taddr1);

        // Register the integrator and set the admin.
        vm.expectEmit(true, true, false, true);
        emit Endpoint.IntegratorRegistered(address(integrator), address(admin));
        endpoint.register(admin);

        // The admin can add an adapter.
        vm.startPrank(admin);
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.AdapterAdded(integrator, taddr1, 1);
        endpoint.addAdapter(integrator, taddr1);

        // Others cannot add an adapter.
        vm.startPrank(imposter);
        vm.expectRevert(abi.encodeWithSelector(Endpoint.CallerNotAuthorized.selector));
        endpoint.addAdapter(integrator, taddr1);

        // Can't register the adapter twice.
        vm.startPrank(admin);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyRegistered.selector, taddr1));
        endpoint.addAdapter(integrator, taddr1);
        // Can't enable the adapter twice.
        endpoint.enableSendAdapter(integrator, 1, taddr1);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyEnabled.selector, taddr1));
        endpoint.enableSendAdapter(integrator, 1, taddr1);

        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.AdapterAdded(integrator, taddr2, 2);
        endpoint.addAdapter(integrator, taddr2);
        address[] memory adapters = endpoint.getSendAdaptersByChain(integrator, 1);
        require(adapters.length == 1, "Wrong number of adapters enabled on chain one, should be 1");
        // Enable another adapter on chain one and one on chain two.
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.SendAdapterEnabledForChain(integrator, 1, taddr2);
        endpoint.enableSendAdapter(integrator, 1, taddr2);
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.AdapterAdded(integrator, taddr3, 3);
        endpoint.addAdapter(integrator, taddr3);
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.SendAdapterEnabledForChain(integrator, 2, taddr3);
        endpoint.enableSendAdapter(integrator, 2, taddr3);

        // And verify they got set properly.
        adapters = endpoint.getSendAdaptersByChain(integrator, 1);
        require(adapters.length == 2, "Wrong number of adapters enabled on chain one");
        require(adapters[0] == taddr1, "Wrong adapter one on chain one");
        require(adapters[1] == taddr2, "Wrong adapter two on chain one");
        adapters = endpoint.getSendAdaptersByChain(integrator, 2);
        require(adapters.length == 1, "Wrong number of adapters enabled on chain two");
        require(adapters[0] == taddr3, "Wrong adapter one on chain two");
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.SendAdapterDisabledForChain(integrator, 2, taddr3);
        endpoint.disableSendAdapter(integrator, 2, taddr3);
        require(adapters.length == 1, "Wrong number of adapters enabled on chain two");
    }

    function test_addRecvAdapter() public {
        address integrator = address(new Integrator(address(endpoint)));
        address admin = address(new Admin(integrator, address(endpoint)));
        address imposter = address(new Admin(integrator, address(endpoint)));
        AdapterImpl adapter1 = new AdapterImpl();
        AdapterImpl adapter2 = new AdapterImpl();
        AdapterImpl adapter3 = new AdapterImpl();
        address taddr1 = address(adapter1);
        address taddr2 = address(adapter2);
        address taddr3 = address(adapter3);
        vm.startPrank(integrator);

        // Can't enable an adapter until we've set the admin.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.IntegratorNotRegistered.selector));
        endpoint.addAdapter(integrator, taddr1);

        // Register the integrator and set the admin.
        vm.expectEmit(true, true, false, true);
        emit Endpoint.IntegratorRegistered(address(integrator), address(admin));
        endpoint.register(admin);

        // The admin can add an adapter.
        vm.startPrank(admin);
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.AdapterAdded(integrator, taddr1, 1);
        endpoint.addAdapter(integrator, taddr1);

        // Others cannot add an adapter.
        vm.startPrank(imposter);
        vm.expectRevert(abi.encodeWithSelector(Endpoint.CallerNotAuthorized.selector));
        endpoint.addAdapter(integrator, taddr1);

        // Can't register the adapter twice.
        vm.startPrank(admin);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyRegistered.selector, taddr1));
        endpoint.addAdapter(integrator, taddr1);
        // Can't enable the adapter twice.
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.RecvAdapterEnabledForChain(integrator, 1, taddr1);
        endpoint.enableRecvAdapter(integrator, 1, taddr1);
        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.AdapterAlreadyEnabled.selector, taddr1));
        endpoint.enableRecvAdapter(integrator, 1, taddr1);

        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.AdapterAdded(integrator, taddr2, 2);
        endpoint.addAdapter(integrator, taddr2);
        address[] memory adapters = endpoint.getRecvAdaptersByChain(integrator, 1);
        require(adapters.length == 1, "Wrong number of adapters enabled on chain one, should be 1");
        // Enable another adapter on chain one and one on chain two.
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.RecvAdapterEnabledForChain(integrator, 1, taddr2);
        endpoint.enableRecvAdapter(integrator, 1, taddr2);
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.AdapterAdded(integrator, taddr3, 3);
        endpoint.addAdapter(integrator, taddr3);
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.RecvAdapterEnabledForChain(integrator, 2, taddr3);
        endpoint.enableRecvAdapter(integrator, 2, taddr3);

        // And verify they got set properly.
        adapters = endpoint.getRecvAdaptersByChain(integrator, 1);
        require(adapters.length == 2, "Wrong number of adapters enabled on chain one");
        require(adapters[0] == taddr1, "Wrong adapter one on chain one");
        require(adapters[1] == taddr2, "Wrong adapter two on chain one");
        adapters = endpoint.getRecvAdaptersByChain(integrator, 2);
        require(adapters.length == 1, "Wrong number of adapters enabled on chain two");
        require(adapters[0] == taddr3, "Wrong adapter one on chain two");
    }

    function test_sendMessage() public {
        address integrator = address(new Integrator(address(endpoint)));
        address admin = address(new Admin(integrator, address(endpoint)));
        uint16 chain = 2;
        uint16 zeroChain = 0;
        AdapterImpl adapter1 = new AdapterImpl();
        AdapterImpl adapter2 = new AdapterImpl();
        AdapterImpl adapter3 = new AdapterImpl();
        vm.startPrank(integrator);
        vm.expectEmit(true, true, false, true);
        emit Endpoint.IntegratorRegistered(address(integrator), address(admin));
        endpoint.register(admin);

        // Sending with no adapters should revert.
        vm.startPrank(integrator);
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdapterNotEnabled.selector));
        uint64 sequence = endpoint.sendMessage(2, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);

        // Now enable some adapters.
        vm.startPrank(admin);
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.AdapterAdded(integrator, address(adapter1), 1);
        endpoint.addAdapter(integrator, address(adapter1));
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.SendAdapterEnabledForChain(integrator, 2, address(adapter1));
        endpoint.enableSendAdapter(integrator, 2, address(adapter1));
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.AdapterAdded(integrator, address(adapter2), 2);
        endpoint.addAdapter(integrator, address(adapter2));
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.SendAdapterEnabledForChain(integrator, 2, address(adapter2));
        endpoint.enableSendAdapter(integrator, 2, address(adapter2));
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.AdapterAdded(integrator, address(adapter3), 3);
        endpoint.addAdapter(integrator, address(adapter3));
        vm.expectEmit(true, true, false, true);
        emit AdapterRegistry.SendAdapterEnabledForChain(integrator, 3, address(adapter3));
        endpoint.enableSendAdapter(integrator, 3, address(adapter3));

        // Only an integrator can call send.
        vm.startPrank(userA);
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdapterNotEnabled.selector));
        sequence = endpoint.sendMessage(chain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);

        // Send a message on chain two. It should go out on the first two adapters, but not the third one.
        vm.startPrank(integrator);
        vm.expectEmit(true, true, false, true);
        emit Endpoint.MessageSent(
            endpoint.computeMessageDigest(
                OurChainId,
                UniversalAddressLibrary.fromAddress(integrator),
                0,
                chain,
                UniversalAddressLibrary.fromAddress(userA),
                payloadHash
            ),
            UniversalAddressLibrary.fromAddress(integrator),
            0,
            UniversalAddressLibrary.fromAddress(userA),
            chain,
            payloadHash
        );
        sequence = endpoint.sendMessage(chain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);
        require(sequence == 0, "Sequence number is wrong");
        require(adapter1.getMessagesSent() == 1, "Failed to send a message on adapter 1");
        require(adapter2.getMessagesSent() == 1, "Failed to send a message on adapter 2");
        require(adapter3.getMessagesSent() == 0, "Should not have sent a message on adapter 3");

        sequence = endpoint.sendMessage(chain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);
        require(sequence == 1, "Second sequence number is wrong");
        require(adapter1.getMessagesSent() == 2, "Failed to send second message on adapter 1");
        require(adapter2.getMessagesSent() == 2, "Failed to send second message on adapter 2");
        require(adapter3.getMessagesSent() == 0, "Should not have sent second message on adapter 3");

        vm.expectRevert(abi.encodeWithSelector(AdapterRegistry.InvalidChain.selector, zeroChain));
        sequence = endpoint.sendMessage(zeroChain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);
        require(sequence == 0, "Failed sequence number is wrong"); // 0 because of the revert

        sequence = endpoint.sendMessage(chain, UniversalAddressLibrary.fromAddress(userA), payloadHash, refundAddr);
        require(sequence == 2, "Third sequence number is wrong");
    }

    function test_attestMessage() public {
        UniversalAddress sourceIntegrator = UniversalAddressLibrary.fromAddress(address(userA));
        address integrator = address(new Integrator(address(endpoint)));
        UniversalAddress destIntegrator = UniversalAddressLibrary.fromAddress(address(integrator));
        address admin = address(new Admin(integrator, address(endpoint)));
        AdapterImpl adapter1 = new AdapterImpl();
        AdapterImpl adapter2 = new AdapterImpl();
        AdapterImpl adapter3 = new AdapterImpl();
        uint16 chain = 2;
        uint16 sequence = 1;
        vm.startPrank(integrator);
        endpoint.register(admin);

        // Attesting with no adapters should revert.
        vm.startPrank(integrator);
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdapterNotEnabled.selector));
        endpoint.attestMessage(2, sourceIntegrator, 1, OurChainId, destIntegrator, payloadHash);

        // Now enable some adapters.
        vm.startPrank(admin);
        endpoint.addAdapter(integrator, address(adapter1));
        endpoint.enableRecvAdapter(integrator, chain, address(adapter1));
        endpoint.addAdapter(integrator, address(adapter2));
        endpoint.enableRecvAdapter(integrator, chain, address(adapter2));
        endpoint.addAdapter(integrator, address(adapter3));
        endpoint.enableRecvAdapter(integrator, chain + 1, address(adapter3));

        // Only an adapter can call attest.
        vm.startPrank(userB);
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdapterNotEnabled.selector));
        endpoint.attestMessage(chain, sourceIntegrator, sequence, OurChainId, destIntegrator, payloadHash);

        // Attesting a message destined for the wrong chain should revert.
        vm.startPrank(address(adapter2));
        vm.expectRevert(abi.encodeWithSelector(Endpoint.InvalidDestinationChain.selector));
        endpoint.attestMessage(chain, sourceIntegrator, sequence, OurChainId + 1, destIntegrator, payloadHash);

        // This attest should work.
        vm.startPrank(address(adapter2));
        vm.expectEmit(true, true, false, true);
        emit Endpoint.MessageAttestedTo(
            endpoint.computeMessageDigest(chain, sourceIntegrator, sequence, OurChainId, destIntegrator, payloadHash),
            chain,
            sourceIntegrator,
            sequence,
            OurChainId,
            destIntegrator,
            payloadHash,
            0x2, // attested bitmap
            UniversalAddressLibrary.fromAddress(address(adapter2))
        );
        endpoint.attestMessage(chain, sourceIntegrator, sequence, OurChainId, destIntegrator, payloadHash);

        // Multiple Attests from same adapter should revert.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.DuplicateMessageAttestation.selector));
        endpoint.attestMessage(chain, sourceIntegrator, sequence, OurChainId, destIntegrator, payloadHash);

        // Receive what we just attested to mark it executed.
        vm.startPrank(integrator);
        vm.expectEmit(true, true, false, true);
        emit Endpoint.MessageReceived(
            endpoint.computeMessageDigest(chain, sourceIntegrator, sequence, OurChainId, destIntegrator, payloadHash),
            chain,
            sourceIntegrator,
            sequence,
            OurChainId,
            destIntegrator,
            payloadHash,
            0x3, // enabled bitmap
            0x2 // attested bitmap
        );
        endpoint.recvMessage(chain, sourceIntegrator, sequence, payloadHash);

        // Attesting after receive should still work on a different adapter.
        vm.startPrank(address(adapter1));
        endpoint.attestMessage(chain, sourceIntegrator, sequence, OurChainId, destIntegrator, payloadHash);

        // Attesting on a disabled adapter should revert.
        vm.startPrank(admin);
        endpoint.disableRecvAdapter(integrator, 2, address(adapter1));
        vm.startPrank(address(adapter1));
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdapterNotEnabled.selector));
        endpoint.attestMessage(chain, sourceIntegrator, sequence, OurChainId, destIntegrator, payloadHash);
    }

    function test_recvMessage() public {
        UniversalAddress sourceIntegrator = UniversalAddressLibrary.fromAddress(address(userA));
        address integrator = address(new Integrator(address(endpoint)));
        UniversalAddress destIntegrator = UniversalAddressLibrary.fromAddress(address(integrator));
        address admin = address(new Admin(integrator, address(endpoint)));
        AdapterImpl adapter1 = new AdapterImpl();
        AdapterImpl adapter2 = new AdapterImpl();
        AdapterImpl adapter3 = new AdapterImpl();
        vm.startPrank(integrator);
        endpoint.register(admin);

        // Receiving with no adapters should revert.
        vm.startPrank(integrator);
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdapterNotEnabled.selector));
        endpoint.recvMessage(2, sourceIntegrator, 1, payloadHash);

        // Now enable some adapters so we can attest. Receive doesn't use the adapters.
        vm.startPrank(admin);
        endpoint.addAdapter(integrator, address(adapter1));
        endpoint.enableRecvAdapter(integrator, 2, address(adapter1));
        endpoint.addAdapter(integrator, address(adapter2));
        endpoint.enableRecvAdapter(integrator, 2, address(adapter2));
        endpoint.addAdapter(integrator, address(adapter3));
        endpoint.enableRecvAdapter(integrator, 3, address(adapter3));

        // Only an integrator can call receive.
        vm.startPrank(userB);
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AdapterNotEnabled.selector));
        endpoint.recvMessage(2, sourceIntegrator, 1, payloadHash);

        // Receiving before there are any attestations should revert.
        vm.startPrank(integrator);
        vm.expectRevert(abi.encodeWithSelector(Endpoint.UnknownMessageAttestation.selector));
        endpoint.recvMessage(2, sourceIntegrator, 1, payloadHash);

        // Attest so we can receive.
        vm.startPrank(address(adapter2));
        endpoint.attestMessage(2, sourceIntegrator, 1, OurChainId, destIntegrator, payloadHash);

        // This receive should work.
        vm.startPrank(integrator);
        (uint128 enabledBitmap, uint128 attestedBitmap) = endpoint.recvMessage(2, sourceIntegrator, 1, payloadHash);

        // Make sure it return the right bitmaps.
        require(enabledBitmap == 0x03, "Enabled bitmap is wrong");
        require(attestedBitmap == 0x02, "Attested bitmap is wrong");

        // But doing it again should revert.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AlreadyExecuted.selector));
        endpoint.recvMessage(2, sourceIntegrator, 1, payloadHash);
    }

    function test_getMessageStatus() public {
        UniversalAddress sourceIntegrator = UniversalAddressLibrary.fromAddress(address(userA));
        address integrator = address(new Integrator(address(endpoint)));
        UniversalAddress destIntegrator = UniversalAddressLibrary.fromAddress(address(integrator));
        address admin = address(new Admin(integrator, address(endpoint)));
        AdapterImpl adapter1 = new AdapterImpl();
        AdapterImpl adapter2 = new AdapterImpl();
        AdapterImpl adapter3 = new AdapterImpl();
        vm.startPrank(integrator);
        endpoint.register(admin);

        // Nothing is attested, yet.
        vm.startPrank(integrator);
        (uint128 enabledBitmap, uint128 attestedBitmap, bool executed) =
            endpoint.getMessageStatus(2, sourceIntegrator, 1, destIntegrator, payloadHash);
        require(enabledBitmap == 0, "Enabled bitmap is wrong1");
        require(attestedBitmap == 0, "Attested bitmap is wrong1");
        require(executed == false, "executed flag is wrong1");

        // Now enable some adapters so we can attest.
        vm.startPrank(admin);
        endpoint.addAdapter(integrator, address(adapter1));
        endpoint.enableRecvAdapter(integrator, 2, address(adapter1));
        endpoint.addAdapter(integrator, address(adapter2));
        endpoint.enableRecvAdapter(integrator, 2, address(adapter2));
        endpoint.addAdapter(integrator, address(adapter3));
        endpoint.enableRecvAdapter(integrator, 3, address(adapter3));

        vm.startPrank(userB);
        (enabledBitmap, attestedBitmap, executed) =
            endpoint.getMessageStatus(2, sourceIntegrator, 1, destIntegrator, payloadHash);
        // 00000011 bitmap = 3 decimal
        require(enabledBitmap == 3, "Enabled bitmap is wrong2");
        require(attestedBitmap == 0, "Attested bitmap is wrong2");
        require(executed == false, "executed flag is wrong2");

        // Should get the same values as above
        vm.startPrank(integrator);
        (enabledBitmap, attestedBitmap, executed) =
            endpoint.getMessageStatus(2, sourceIntegrator, 1, destIntegrator, payloadHash);
        require(enabledBitmap == 3, "Enabled bitmap is wrong4");
        require(attestedBitmap == 0, "Attested bitmap is wrong4");
        require(executed == false, "executed flag is wrong4");

        // Attest
        vm.startPrank(address(adapter2));
        endpoint.attestMessage(2, sourceIntegrator, 1, OurChainId, destIntegrator, payloadHash);

        // Should now have a non zero value for attested bitmap
        vm.startPrank(integrator);
        (enabledBitmap, attestedBitmap, executed) =
            endpoint.getMessageStatus(2, sourceIntegrator, 1, destIntegrator, payloadHash);
        require(enabledBitmap == 3, "Enabled bitmap is wrong5");
        // 00000010 bitmap = 2 decimal
        require(attestedBitmap == 2, "Attested bitmap is wrong5");
        require(executed == false, "executed flag is wrong5");

        // Test the second version of getMessageStatus.
        (enabledBitmap, attestedBitmap, executed) = endpoint.getMessageStatus(2, sourceIntegrator, 1, payloadHash);
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

        EndpointImpl endpoint = new EndpointImpl();
        vm.startPrank(dstIntegrator);
        endpoint.register(dstIntegrator);

        // Register and enable recv adapter.
        AdapterImpl recvTrans = new AdapterImpl();
        endpoint.addAdapter(dstIntegrator, address(recvTrans));
        endpoint.enableRecvAdapter(dstIntegrator, srcChain, address(recvTrans));

        // Dest adapter needs to attest.
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
        address integrator = address(new Integrator(address(endpoint)));
        address admin = address(new Admin(integrator, address(endpoint)));
        AdapterImpl adapter1 = new AdapterImpl();
        uint16 chain1 = 1;
        uint64 sequence = 1;
        uint128 enabledBitmap;
        uint128 attestedBitmap;
        bool executed;

        // Register the integrator and set the admin.
        vm.startPrank(integrator);
        endpoint.register(admin);

        (enabledBitmap, attestedBitmap, executed) = endpoint.getMessageStatus(
            chain1,
            UniversalAddressLibrary.fromAddress(address(adapter1)),
            sequence,
            UniversalAddressLibrary.fromAddress(address(integrator)),
            payloadHash
        );
        require(executed == false, "executed flag should be false before execMessage");
        endpoint.execMessage(chain1, UniversalAddressLibrary.fromAddress(address(adapter1)), sequence, payloadHash);
        (enabledBitmap, attestedBitmap, executed) = endpoint.getMessageStatus(
            chain1,
            UniversalAddressLibrary.fromAddress(address(adapter1)),
            sequence,
            UniversalAddressLibrary.fromAddress(address(integrator)),
            payloadHash
        );
        require(executed == true, "executed flag should be true after execMessage");
        // Second execMessage should revert.
        vm.expectRevert(abi.encodeWithSelector(Endpoint.AlreadyExecuted.selector));
        endpoint.execMessage(chain1, UniversalAddressLibrary.fromAddress(address(adapter1)), sequence, payloadHash);
    }

    function test_computepayloadHash() public view {
        UniversalAddress sourceIntegrator = UniversalAddressLibrary.fromAddress(address(userA));
        UniversalAddress destIntegrator = UniversalAddressLibrary.fromAddress(address(userB));
        uint16 srcChain = 2;
        uint16 dstChain = 42;
        uint64 sequence = 3;
        bytes32 mypayloadHash =
            endpoint.computeMessageDigest(srcChain, sourceIntegrator, sequence, dstChain, destIntegrator, payloadHash);
        bytes32 expectedHash =
            keccak256(abi.encodePacked(srcChain, sourceIntegrator, sequence, dstChain, destIntegrator, payloadHash));
        require(mypayloadHash == expectedHash, "Message hash is wrong");
        require(
            mypayloadHash == 0xf589999616054a74b876390c4eb6e067da272da5cd313a9657d33ec3cab06760,
            "Message hash literal is wrong"
        );
    }

    function test_quoteDeliveryPrice() public {
        address integrator = address(new Integrator(address(endpoint)));
        address admin = address(new Admin(integrator, address(endpoint)));
        uint16 chain = 2;
        AdapterImpl adapter1 = new AdapterImpl();
        AdapterImpl adapter2 = new AdapterImpl();
        AdapterImpl adapter3 = new AdapterImpl();
        vm.startPrank(integrator);
        endpoint.register(admin);

        // Set the delivery price.
        adapter1.setDeliveryPrice(100);
        adapter2.setDeliveryPrice(200);
        adapter3.setDeliveryPrice(300);

        // Now enable some adapters.
        vm.startPrank(admin);
        endpoint.addAdapter(integrator, address(adapter1));
        endpoint.enableSendAdapter(integrator, chain, address(adapter1));
        uint256 price = endpoint.quoteDeliveryPrice(integrator, chain);
        require(price == 100, "Single price is wrong");
        endpoint.addAdapter(integrator, address(adapter2));
        endpoint.enableSendAdapter(integrator, chain, address(adapter2));
        price = endpoint.quoteDeliveryPrice(integrator, chain);
        require(price == 300, "Double price is wrong");
        endpoint.addAdapter(integrator, address(adapter3));
        endpoint.enableSendAdapter(integrator, 3, address(adapter3));
        price = endpoint.quoteDeliveryPrice(integrator, chain);
        require(price == 300, "Triple price is wrong");
        vm.startPrank(integrator);
        price = endpoint.quoteDeliveryPrice(chain);
        require(price == 300, "Triple price is wrong");
    }
}
