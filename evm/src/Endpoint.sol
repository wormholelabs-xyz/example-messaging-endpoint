// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "./interfaces/IEndpointAdmin.sol";
import "./interfaces/IEndpointIntegrator.sol";
import "./interfaces/IEndpointAdapter.sol";
import "./MessageSequence.sol";
import "./AdapterRegistry.sol";
import "./interfaces/IAdapter.sol";

string constant endpointVersion = "Endpoint-0.0.1";

contract Endpoint is IEndpointAdmin, IEndpointIntegrator, IEndpointAdapter, MessageSequence, AdapterRegistry {
    string public constant ENDPOINT_VERSION = endpointVersion;

    struct IntegratorConfig {
        bool isInitialized;
        address admin;
        address pending_admin;
    }

    // =============== Immutables ============================================================

    /// @dev Wormhole chain ID that the Endpoint is deployed on.
    /// This chain ID is formatted Wormhole Chain IDs -- https://docs.wormhole.com/wormhole/reference/constants.
    uint16 public immutable ourChain;

    // =============== Setup =================================================================

    constructor(uint16 _ourChain) {
        ourChain = _ourChain;
    }

    // =============== Events ================================================================

    /// @notice Emitted when an integrator registers with the endpoint.
    /// @dev Topic0
    ///      0x582a4322c684b4cdebf273e2be5090d5f21476be5566a98d6a224a450447c5b4.
    /// @param integrator The address of the integrator contract.
    /// @param admin The address of the admin contract.
    event IntegratorRegistered(address integrator, address admin);

    /// @notice Emitted when the admin is changed for an integrator.
    /// @dev Topic0
    ///      0x9f6130d220a6021d90d78c7ed17b7cfb79f530974405b174fef75f671205513c.
    /// @param integrator The address of the integrator contract.
    /// @param oldAdmin The address of the old admin contract.
    /// @param newAdmin The address of the new admin contract.
    event AdminUpdated(address integrator, address oldAdmin, address newAdmin);

    /// @notice Emitted when an admin change request is received for an integrator.
    /// @dev Topic0
    ///      0xcdeb0d05a920666dfd2822eb51628fff963ba0b1672f984a8b60017ed83939e4.
    /// @param integrator The address of the integrator contract.
    /// @param oldAdmin The address of the old admin contract.
    /// @param newAdmin The address of the new admin contract.
    event AdminUpdateRequested(address integrator, address oldAdmin, address newAdmin);

    /// @notice Emitted when a message has been sent.
    /// @dev Topic0
    ///      0x1c170583317700fb71bc583fa6fdd8ff893f6c3a15a79104f1681d6d9eb708ee.
    /// @param messageDigest The keccak256 of the provided fields.  It is, also, indexed.
    /// @param sender The address of the sender.
    /// @param sequence The sequence of the message.
    /// @param recipient The address of the recipient.
    /// @param recipientChain The chainId of the recipient.
    /// @param payloadDigest The digest of the payload (from the integrator).
    event MessageSent(
        bytes32 indexed messageDigest,
        UniversalAddress sender,
        uint64 sequence,
        UniversalAddress recipient,
        uint16 recipientChain,
        bytes32 payloadDigest
    );

    /// @notice Emitted when a message has been attested to.
    /// @dev Topic0
    ///      0xb2328f51e669b73cf1831e232716eec9959360a52818a63bb1d82d900de667d8.
    /// @param messageHash The keccak256 of the message.  It is, also, indexed.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param dstChain The Wormhole chain ID of the destination.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @param attestedBitmap Bitmap of adapters that have attested the message.
    /// @param attestingAdapter The address of the adapter that attested the message.
    event MessageAttestedTo(
        bytes32 indexed messageHash,
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash,
        uint128 attestedBitmap,
        UniversalAddress attestingAdapter
    );

    /// @notice Emitted when a message has been received.
    /// @dev Topic0
    ///      0xae4f20b00e13c9f1eec6c3c72ba3146c9538ca60f28c3eb57538b14965905e7d.
    /// @param messageHash The keccak256 of the message.  It is, also, indexed.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param dstChain The Wormhole chain ID of the destination.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @param enabledBitmap Bitmap of adapters enabled for the source chain.
    /// @param attestedBitmap Bitmap of adapters that have attested the message.
    event MessageReceived(
        bytes32 indexed messageHash,
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash,
        uint128 enabledBitmap,
        uint128 attestedBitmap
    );

    // =============== Errors ================================================================

    /// @notice Error when the destination chain ID doesn't match this chain.
    /// @dev Selector: 0xb86ac1ef.
    error InvalidDestinationChain();

    /// @notice Error when the adapter is being used as if it is enabled but it is disabled.
    /// @dev Selector: 0x5bacac21.
    error AdapterNotEnabled();

    /// @notice Error when the admin is the zero address.
    /// @dev Selector: 0x554ff5d7.
    error InvalidAdminZeroAddress();

    /// @notice Error when there was an attempt to change the admin while a transfer is in progress.
    /// @dev Selector: 0xc78a581c.
    error AdminTransferInProgress();

    /// @notice Error when there was an attempt to claim the admin while no transfer was in progress.
    /// @dev Selector: 0xe8aba8ca.
    error NoAdminTransferInProgress();

    /// @notice Error when the integrator tries to re-register.
    /// @dev Selector: 0x626bb491.
    error IntegratorAlreadyRegistered();

    /// @notice Error when the integrator did not register an admin.
    /// @dev Selector: 0x815255c7.
    error IntegratorNotRegistered();

    /// @notice Error when the caller is not the registered admin.
    /// @dev Selector: 0xc183bcef.
    error CallerNotAuthorized();

    /// @notice Error when message attestation not found in store.
    /// @dev Selector: 0x1547aa01.
    error UnknownMessageAttestation();

    /// @notice Error when message is attempted to be attested multiple times.
    /// @dev Selector: 0x833e2681.
    error DuplicateMessageAttestation();

    /// @notice Error when message is already marked as executed.
    /// @dev Selector: 0x0dc10197.
    error AlreadyExecuted();

    // =============== Storage ===============================================================

    /// @dev Holds the integrator address to IntegratorConfig mapping.
    ///      mapping(address => IntegratorConfig).
    bytes32 private constant INTEGRATOR_CONFIGS_SLOT = bytes32(uint256(keccak256("endpoint.integratorConfigs")) - 1);

    /// @dev Integrator address => IntegratorConfig mapping.
    function _getIntegratorConfigsStorage() internal pure returns (mapping(address => IntegratorConfig) storage $) {
        uint256 slot = uint256(INTEGRATOR_CONFIGS_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    struct AttestationInfo {
        bool executed; // replay protection
        uint128 attestedAdapters; // bitmap corresponding to perIntegratorAdapters
    }

    /// @dev Holds the integrator address to message digest to attestation info mapping.
    ///      mapping(address => mapping(bytes32 => AttestationInfo).
    bytes32 private constant ATTESTATION_INFO_SLOT = bytes32(uint256(keccak256("endpoint.attestationInfo")) - 1);

    /// @dev Integrator address => message digest -> attestation info mapping.
    function _getAttestationInfoStorage()
        internal
        pure
        returns (mapping(address => mapping(bytes32 => AttestationInfo)) storage $)
    {
        uint256 slot = uint256(ATTESTATION_INFO_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    // =============== Getters ===============================================================

    /// @notice Returns the admin for a given integrator.
    /// @param integrator The address of the integrator contract.
    /// @return address The address of the administrator contract.
    function getAdmin(address integrator) public view returns (address) {
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();
        return integratorConfigs[integrator].admin;
    }

    /// @notice Returns the pending_admin for a given integrator.
    /// @param integrator The address of the integrator contract.
    /// @return address The address of the pending administrator contract.
    function getPendingAdmin(address integrator) public view returns (address) {
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();
        return integratorConfigs[integrator].pending_admin;
    }

    // =============== External ==============================================================

    /// @notice Computes the message digest.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param dstChain The Wormhole chain ID of the destination.
    /// @param dstAddr The destination universal address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @return bytes32 The keccak256 of the provided fields.
    function computeMessageDigest(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash));
    }

    // =============== Admin functions =======================================================

    /// @inheritdoc IEndpointIntegrator
    function register(address initialAdmin) external {
        if (initialAdmin == address(0)) {
            revert InvalidAdminZeroAddress();
        }

        address integrator = msg.sender;

        // Get the storage for this integrator contract
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();

        if (integratorConfigs[integrator].isInitialized) {
            revert IntegratorAlreadyRegistered();
        }

        // Update the storage.
        integratorConfigs[integrator] =
            IntegratorConfig({isInitialized: true, admin: initialAdmin, pending_admin: address(0)});
        emit IntegratorRegistered(integrator, initialAdmin);
    }

    /// @inheritdoc IEndpointAdmin
    function updateAdmin(address integrator, address newAdmin) external onlyAdmin(integrator) {
        if (newAdmin == address(0)) {
            // Use discardAdmin() instead.
            revert InvalidAdminZeroAddress();
        }
        // Get the storage for this integrator contract
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();

        if (integratorConfigs[integrator].pending_admin != address(0)) {
            revert AdminTransferInProgress();
        }

        // Update the storage.
        integratorConfigs[integrator].admin = newAdmin;
        emit AdminUpdated(integrator, msg.sender, newAdmin);
    }

    /// @inheritdoc IEndpointAdmin
    function transferAdmin(address integrator, address newAdmin) external onlyAdmin(integrator) {
        if (newAdmin == address(0)) {
            // Use discardAdmin() instead.
            revert InvalidAdminZeroAddress();
        }
        // Get the storage for this integrator contract
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();

        if (integratorConfigs[integrator].pending_admin != address(0)) {
            revert AdminTransferInProgress();
        }

        // Update the storage with this request.
        integratorConfigs[integrator].pending_admin = newAdmin;
        emit AdminUpdateRequested(integrator, msg.sender, newAdmin);
    }

    /// @inheritdoc IEndpointAdmin
    function claimAdmin(address integrator) external {
        // Get the storage for this integrator contract
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();

        if (integratorConfigs[integrator].pending_admin == address(0)) {
            revert NoAdminTransferInProgress();
        }

        address oldAdmin = integratorConfigs[integrator].admin;
        address pendingAdmin = integratorConfigs[integrator].pending_admin;
        address newAdmin = msg.sender;
        if (newAdmin != oldAdmin && newAdmin != pendingAdmin) {
            revert CallerNotAuthorized();
        }
        // Update the storage with this request.
        integratorConfigs[integrator].admin = newAdmin;
        integratorConfigs[integrator].pending_admin = address(0);
        emit AdminUpdated(integrator, oldAdmin, newAdmin);
    }

    /// @inheritdoc IEndpointAdmin
    function discardAdmin(address integrator) external onlyAdmin(integrator) {
        // Get the storage for this integrator contract
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();

        if (integratorConfigs[integrator].pending_admin != address(0)) {
            revert AdminTransferInProgress();
        }

        // Update the storage.
        integratorConfigs[integrator].admin = address(0);
        emit AdminUpdated(integrator, msg.sender, address(0));
    }

    // =============== Adapter functions =======================================================

    /// @inheritdoc IEndpointAdmin
    function addAdapter(address integrator, address adapter) external onlyAdmin(integrator) returns (uint8 index) {
        // Call the AdapterRegistry version.
        return _addAdapter(integrator, adapter);
    }

    /// @inheritdoc IEndpointAdmin
    function enableSendAdapter(address integrator, uint16 chain, address adapter) external onlyAdmin(integrator) {
        // Call the AdapterRegistry version.
        _enableSendAdapter(integrator, chain, adapter);
    }

    /// @inheritdoc IEndpointAdmin
    function enableRecvAdapter(address integrator, uint16 chain, address adapter) external onlyAdmin(integrator) {
        // Call the AdapterRegistry version.
        _enableRecvAdapter(integrator, chain, adapter);
    }

    /// @inheritdoc IEndpointAdmin
    function disableSendAdapter(address integrator, uint16 chain, address adapter) external onlyAdmin(integrator) {
        // Call the AdapterRegistry version.
        _disableSendAdapter(integrator, chain, adapter);
    }

    /// @inheritdoc IEndpointAdmin
    function disableRecvAdapter(address integrator, uint16 chain, address adapter) external onlyAdmin(integrator) {
        // Call the AdapterRegistry version.
        _disableRecvAdapter(integrator, chain, adapter);
    }

    /// @inheritdoc IEndpointAdmin
    function getNumEnabledRecvAdaptersForChain(address integrator, uint16 chain) external view returns (uint8 count) {
        // Call the AdapterRegistry version.
        return _getNumEnabledRecvAdaptersForChain(integrator, chain);
    }

    // =============== Message functions =======================================================

    /// @inheritdoc IEndpointIntegrator
    function sendMessage(uint16 dstChain, UniversalAddress dstAddr, bytes32 payloadHash, address refundAddress)
        external
        payable
        returns (uint64 sequence)
    {
        // get the enabled send adapters for [msg.sender][dstChain]
        address[] memory sendAdapters = getSendAdaptersByChain(msg.sender, dstChain);
        uint256 len = sendAdapters.length;
        if (len == 0) {
            revert AdapterNotEnabled();
        }
        UniversalAddress sender = UniversalAddressLibrary.fromAddress(msg.sender);
        // get the next sequence number for msg.sender
        sequence = _useMessageSequence(msg.sender);
        for (uint256 i = 0; i < len;) {
            // quote the delivery price
            uint256 deliveryPrice = IAdapter(sendAdapters[i]).quoteDeliveryPrice(dstChain);
            // call sendMessage
            IAdapter(sendAdapters[i]).sendMessage{value: deliveryPrice}(
                sender, sequence, dstChain, dstAddr, payloadHash, refundAddress
            );
            unchecked {
                ++i;
            }
        }

        emit MessageSent(
            computeMessageDigest(ourChain, sender, sequence, dstChain, dstAddr, payloadHash),
            sender,
            sequence,
            dstAddr,
            dstChain,
            payloadHash
        );
    }

    /// @inheritdoc IEndpointAdapter
    function attestMessage(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external {
        address integrator = dstAddr.toAddress();

        // sanity check that destinationChain is this chain
        if (dstChain != ourChain) {
            revert InvalidDestinationChain();
        }

        AdapterInfo storage tsInfo = _getAdapterInfosStorage()[integrator][msg.sender];
        if (!tsInfo.registered) {
            revert AdapterNotEnabled();
        }

        // Make sure it's enabled on the receive.
        if (!_isRecvAdapterEnabledForChainWithCheck(integrator, srcChain, msg.sender)) {
            revert AdapterNotEnabled();
        }

        // compute the message digest
        bytes32 messageDigest = computeMessageDigest(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash);

        AttestationInfo storage attestationInfo = _getAttestationInfoStorage()[integrator][messageDigest];
        uint128 updatedAdapters = attestationInfo.attestedAdapters | uint128(1 << tsInfo.index);
        // Check that this message has not already been attested.
        if (updatedAdapters == attestationInfo.attestedAdapters) {
            revert DuplicateMessageAttestation();
        }

        // It's okay to mark it as attested if it has already been executed.

        // set the bit in perIntegratorAttestations[dstAddr][digest] corresponding to msg.sender
        attestationInfo.attestedAdapters = updatedAdapters;
        emit MessageAttestedTo(
            computeMessageDigest(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash),
            srcChain,
            srcAddr,
            sequence,
            dstChain,
            dstAddr,
            payloadHash,
            attestationInfo.attestedAdapters,
            UniversalAddressLibrary.fromAddress(msg.sender)
        );
    }

    /// @inheritdoc IEndpointIntegrator
    function recvMessage(uint16 srcChain, UniversalAddress srcAddr, uint64 sequence, bytes32 payloadHash)
        external
        payable
        returns (uint128 enabledBitmap, uint128 attestedBitmap)
    {
        enabledBitmap = _getEnabledRecvAdaptersBitmapForChain(msg.sender, srcChain);
        if (enabledBitmap == 0) {
            revert AdapterNotEnabled();
        }
        UniversalAddress dstAddr = UniversalAddressLibrary.fromAddress(msg.sender);

        // compute the message digest
        bytes32 messageDigest = computeMessageDigest(srcChain, srcAddr, sequence, ourChain, dstAddr, payloadHash);

        AttestationInfo storage attestationInfo = _getAttestationInfoStorage()[msg.sender][messageDigest];

        // revert if not in perIntegratorAttestations map
        if ((attestationInfo.attestedAdapters == 0) && (!attestationInfo.executed)) {
            revert UnknownMessageAttestation();
        }

        // revert if already executed
        if (attestationInfo.executed) {
            revert AlreadyExecuted();
        }

        // set the executed flag in perIntegratorAttestations[dstAddr][digest]
        attestationInfo.executed = true;
        attestedBitmap = attestationInfo.attestedAdapters;

        emit MessageReceived(
            messageDigest, srcChain, srcAddr, sequence, ourChain, dstAddr, payloadHash, enabledBitmap, attestedBitmap
        );
    }

    /// @inheritdoc IEndpointIntegrator
    function getMessageStatus(uint16 srcChain, UniversalAddress srcAddr, uint64 sequence, bytes32 payloadHash)
        external
        view
        returns (uint128 enabledBitmap, uint128 attestedBitmap, bool executed)
    {
        return _getMessageStatus(
            srcChain, srcAddr, sequence, UniversalAddressLibrary.fromAddress(msg.sender), msg.sender, payloadHash
        );
    }

    /// @inheritdoc IEndpointIntegrator
    function getMessageStatus(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external view returns (uint128 enabledBitmap, uint128 attestedBitmap, bool executed) {
        return _getMessageStatus(srcChain, srcAddr, sequence, dstAddr, dstAddr.toAddress(), payloadHash);
    }

    /// @inheritdoc IEndpointIntegrator
    function execMessage(uint16 srcChain, UniversalAddress srcAddr, uint64 sequence, bytes32 payloadHash) external {
        // compute the message digest
        bytes32 messageDigest = computeMessageDigest(
            srcChain, srcAddr, sequence, ourChain, UniversalAddressLibrary.fromAddress(msg.sender), payloadHash
        );

        AttestationInfo storage attestationInfo = _getAttestationInfoStorage()[msg.sender][messageDigest];

        if (attestationInfo.executed) {
            revert AlreadyExecuted();
        }
        attestationInfo.executed = true;
    }

    /// @inheritdoc IEndpointIntegrator
    function quoteDeliveryPrice(address integrator, uint16 dstChain) external view returns (uint256) {
        return _quoteDeliveryPrice(integrator, dstChain);
    }

    /// @inheritdoc IEndpointIntegrator
    function quoteDeliveryPrice(uint16 dstChain) external view returns (uint256) {
        return _quoteDeliveryPrice(msg.sender, dstChain);
    }

    // =============== Internal ==============================================================

    modifier onlyAdmin(address integrator) {
        IntegratorConfig storage config = _getIntegratorConfigsStorage()[integrator];
        if (!config.isInitialized) {
            revert IntegratorNotRegistered();
        }

        if (config.admin != msg.sender) {
            revert CallerNotAuthorized();
        }
        _;
    }

    /// @notice Retrieves the quote for message delivery.
    /// @dev This version does not need to be called by the integrator.
    /// @dev This sums up all the individual sendAdapter's quoteDeliveryPrice calls.
    /// @param integrator The address of the integrator.
    /// @param dstChain The Wormhole chain ID of the recipient.
    /// @return totalCost The total cost of delivering a message to the recipient chain in this chain's native token.
    function _quoteDeliveryPrice(address integrator, uint16 dstChain) internal view returns (uint256 totalCost) {
        address[] memory sendAdapters = getSendAdaptersByChain(integrator, dstChain);
        uint256 len = sendAdapters.length;
        totalCost = 0;
        for (uint256 i = 0; i < len;) {
            totalCost += IAdapter(sendAdapters[i]).quoteDeliveryPrice(dstChain);
            unchecked {
                ++i;
            }
        }
        return totalCost;
    }

    /// @notice Retrieves the status of a message.
    /// @dev This is an internal function taking an extra destination address format in order to avoid extra conversions.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param dstUAddr The destination universal address of the message.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator
    /// @return enabledBitmap A bitmap indicating enabled receive adapters for the destination address.
    /// @return attestedBitmap A bitmap indicating attested adapters for the message.
    /// @return executed A boolean indicating if the message has been executed.
    function _getMessageStatus(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        UniversalAddress dstUAddr,
        address dstAddr,
        bytes32 payloadHash
    ) internal view returns (uint128 enabledBitmap, uint128 attestedBitmap, bool executed) {
        enabledBitmap = _getEnabledRecvAdaptersBitmapForChain(dstAddr, srcChain);
        // compute the message digest
        bytes32 messageDigest = computeMessageDigest(srcChain, srcAddr, sequence, ourChain, dstUAddr, payloadHash);

        AttestationInfo storage attestationInfo = _getAttestationInfoStorage()[dstAddr][messageDigest];

        attestedBitmap = attestationInfo.attestedAdapters;

        executed = attestationInfo.executed;
    }
}
