// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "./interfaces/IRouterAdmin.sol";
import "./interfaces/IRouterIntegrator.sol";
import "./interfaces/IRouterTransceiver.sol";
import "./MessageSequence.sol";
import "./TransceiverRegistry.sol";
import "./interfaces/ITransceiver.sol";

string constant routerVersion = "Router-0.0.1";

contract Router is IRouterAdmin, IRouterIntegrator, IRouterTransceiver, MessageSequence, TransceiverRegistry {
    string public constant ROUTER_VERSION = routerVersion;

    struct IntegratorConfig {
        bool isInitialized;
        address admin;
        address pending_admin;
    }

    // =============== Immutables ============================================================

    /// @dev Wormhole chain ID that the Router is deployed on.
    /// This chain ID is formatted Wormhole Chain IDs -- https://docs.wormhole.com/wormhole/reference/constants
    uint16 public immutable ourChainId;

    // =============== Setup =================================================================

    constructor(uint16 _ourChainId) {
        ourChainId = _ourChainId;
    }

    // =============== Events ================================================================

    /// @notice Emitted when an integrator registers with the router.
    /// @dev Topic0
    ///      0x582a4322c684b4cdebf273e2be5090d5f21476be5566a98d6a224a450447c5b4
    /// @param integrator The address of the integrator contract.
    /// @param admin The address of the admin contract.
    event IntegratorRegistered(address integrator, address admin);

    /// @notice Emitted when the admin is changed for an integrator.
    /// @dev Topic0
    ///      0x9f6130d220a6021d90d78c7ed17b7cfb79f530974405b174fef75f671205513c
    /// @param integrator The address of the integrator contract.
    /// @param oldAdmin The address of the old admin contract.
    /// @param newAdmin The address of the new admin contract.
    event AdminUpdated(address integrator, address oldAdmin, address newAdmin);

    /// @notice Emitted when an admin change request is received for an integrator.
    /// @dev Topic0
    ///      0xcdeb0d05a920666dfd2822eb51628fff963ba0b1672f984a8b60017ed83939e4
    /// @param integrator The address of the integrator contract.
    /// @param oldAdmin The address of the old admin contract.
    /// @param newAdmin The address of the new admin contract.
    event AdminUpdateRequested(address integrator, address oldAdmin, address newAdmin);

    /// @notice Emitted when a message has been sent.
    /// @param messageDigest The keccak256 of the provided fields.  It is, also, indexed.
    /// @param sender The address of the sender.
    /// @param sequence The sequence of the message.
    /// @param recipient The address of the recipient.
    /// @param recipientChain The chainId of the recipient.
    /// @param payloadDigest The digest of the payload (from the integrator).
    /// @dev Topic0 0x1c170583317700fb71bc583fa6fdd8ff893f6c3a15a79104f1681d6d9eb708ee
    event MessageSent(
        bytes32 indexed messageDigest,
        UniversalAddress sender,
        uint64 sequence,
        UniversalAddress recipient,
        uint16 recipientChain,
        bytes32 payloadDigest
    );

    /// @notice Emitted when a message has been attested to.
    /// @param messageHash The keccak256 of the message.  It is, also, indexed.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param dstChain The Wormhole chain ID of the destination.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @param attestedBitmap Bitmap of transceivers that have attested the message.
    /// @param attestingTransceiver The address of the transceiver that attested the message.
    /// @dev Topic0 0xb2328f51e669b73cf1831e232716eec9959360a52818a63bb1d82d900de667d8
    event MessageAttestedTo(
        bytes32 indexed messageHash,
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash,
        uint128 attestedBitmap,
        UniversalAddress attestingTransceiver
    );

    /// @notice Emitted when a message has been received.
    /// @param messageHash The keccak256 of the message.  It is, also, indexed.
    /// @param srcChain The Wormhole chain ID of the sender.
    /// @param srcAddr The universal address of the peer on the sending chain.
    /// @param sequence The sequence number of the message (per integrator).
    /// @param dstChain The Wormhole chain ID of the destination.
    /// @param dstAddr The destination address of the message.
    /// @param payloadHash The keccak256 of payload from the integrator.
    /// @param enabledBitmap Bitmap of transceivers enabled for the source chain.
    /// @param attestedBitmap Bitmap of transceivers that have attested the message.
    /// @dev Topic0 0xae4f20b00e13c9f1eec6c3c72ba3146c9538ca60f28c3eb57538b14965905e7d
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

    /// @notice Error when the transceiver is being used as if it is enabled but it is disabled.
    /// @dev Selector: 0x424afc23.
    error TransceiverNotEnabled();

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
    ///      mapping(address => IntegratorConfig)
    bytes32 private constant INTEGRATOR_CONFIGS_SLOT = bytes32(uint256(keccak256("router.integratorConfigs")) - 1);

    /// @dev Integrator address => IntegratorConfig mapping.
    function _getIntegratorConfigsStorage() internal pure returns (mapping(address => IntegratorConfig) storage $) {
        uint256 slot = uint256(INTEGRATOR_CONFIGS_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    struct AttestationInfo {
        bool executed; // replay protection
        uint128 attestedTransceivers; // bitmap corresponding to perIntegratorTransceivers
    }

    /// @dev Holds the integrator address to message digest to attestation info mapping.
    ///      mapping(address => mapping(bytes32 => AttestationInfo)
    bytes32 private constant ATTESTATION_INFO_SLOT = bytes32(uint256(keccak256("router.attestationInfo")) - 1);

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
    /// @return address The address of the administrator contract
    function getAdmin(address integrator) public view returns (address) {
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();
        return integratorConfigs[integrator].admin;
    }

    /// @notice Returns the pending_admin for a given integrator.
    /// @param integrator The address of the integrator contract.
    /// @return address The address of the pending administrator contract
    function getPendingAdmin(address integrator) public view returns (address) {
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();
        return integratorConfigs[integrator].pending_admin;
    }

    // =============== External ==============================================================

	/// @notice This function is used to compute the message digest.
    /// @param srcChain The Wormhole chain ID of the sender
    /// @param srcAddr The universal address of the peer on the sending chain
    /// @param sequence The sequence number of the message (per integrator)
    /// @param dstChain The Wormhole chain ID of the destination
    /// @param dstAddr The destination universal address of the message
    /// @param payloadHash The keccak256 of payload from the integrator
    /// @return bytes32 The keccak256 of the provided fields
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

    /// @inheritdoc IRouterIntegrator
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

    /// @inheritdoc IRouterAdmin
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

    /// @inheritdoc IRouterAdmin
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

    /// @inheritdoc IRouterAdmin
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

    /// @inheritdoc IRouterAdmin
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

    // =============== Transceiver functions =======================================================

    /// @inheritdoc IRouterAdmin
    function addTransceiver(address integrator, address transceiver)
        external
        onlyAdmin(integrator)
        returns (uint8 index)
    {
        // Call the TransceiverRegistry version.
        return _addTransceiver(integrator, transceiver);
    }

    /// @inheritdoc IRouterAdmin
    function enableSendTransceiver(address integrator, uint16 chain, address transceiver)
        external
        onlyAdmin(integrator)
    {
        // Call the TransceiverRegistry version.
        _enableSendTransceiver(integrator, chain, transceiver);
    }

    /// @inheritdoc IRouterAdmin
    function enableRecvTransceiver(address integrator, uint16 chain, address transceiver)
        external
        onlyAdmin(integrator)
    {
        // Call the TransceiverRegistry version.
        _enableRecvTransceiver(integrator, chain, transceiver);
    }

    /// @inheritdoc IRouterAdmin
    function disableSendTransceiver(address integrator, uint16 chain, address transceiver)
        external
        onlyAdmin(integrator)
    {
        // Call the TransceiverRegistry version.
        _disableSendTransceiver(integrator, chain, transceiver);
    }

    /// @inheritdoc IRouterAdmin
    function disableRecvTransceiver(address integrator, uint16 chain, address transceiver)
        external
        onlyAdmin(integrator)
    {
        // Call the TransceiverRegistry version.
        _disableRecvTransceiver(integrator, chain, transceiver);
    }

    // =============== Message functions =======================================================

    /// @inheritdoc IRouterIntegrator
    function sendMessage(uint16 dstChain, UniversalAddress dstAddr, bytes32 payloadHash, address refundAddress)
        external
        payable
        returns (uint64 sequence)
    {
        // get the enabled send transceivers for [msg.sender][dstChain]
        address[] memory sendTransceivers = getSendTransceiversByChain(msg.sender, dstChain);
        uint256 len = sendTransceivers.length;
        if (len == 0) {
            revert TransceiverNotEnabled();
        }
        UniversalAddress sender = UniversalAddressLibrary.fromAddress(msg.sender);
        // get the next sequence number for msg.sender
        sequence = _useMessageSequence(msg.sender);
        for (uint256 i = 0; i < len;) {
            // quote the delivery price
            uint256 deliveryPrice = ITransceiver(sendTransceivers[i]).quoteDeliveryPrice(dstChain);
            // call sendMessage
            ITransceiver(sendTransceivers[i]).sendMessage{value: deliveryPrice}(
                sender, sequence, dstChain, dstAddr, payloadHash, refundAddress
            );
            unchecked {
                ++i;
            }
        }

        emit MessageSent(
            computeMessageDigest(ourChainId, sender, sequence, dstChain, dstAddr, payloadHash),
            sender,
            sequence,
            dstAddr,
            dstChain,
            payloadHash
        );
    }

    /// @inheritdoc IRouterTransceiver
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
        if (dstChain != ourChainId) {
            revert InvalidDestinationChain();
        }

        TransceiverInfo storage tsInfo = _getTransceiverInfosStorage()[integrator][msg.sender];
        if (!tsInfo.registered) {
            revert TransceiverNotEnabled();
        }

        // Make sure it's enabled on the receive.
        if (!_isRecvTransceiverEnabledForChainWithCheck(integrator, srcChain, msg.sender)) {
            revert TransceiverNotEnabled();
        }

        // compute the message digest
        bytes32 messageDigest = computeMessageDigest(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash);

        AttestationInfo storage attestationInfo = _getAttestationInfoStorage()[integrator][messageDigest];
        uint128 updatedTransceivers = attestationInfo.attestedTransceivers | uint128(1 << tsInfo.index);
        // Check that this message has not already been attested.
        if (updatedTransceivers == attestationInfo.attestedTransceivers) {
            revert DuplicateMessageAttestation();
        }

        // It's okay to mark it as attested if it has already been executed.

        // set the bit in perIntegratorAttestations[dstAddr][digest] corresponding to msg.sender
        attestationInfo.attestedTransceivers = updatedTransceivers;
        emit MessageAttestedTo(
            computeMessageDigest(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash),
            srcChain,
            srcAddr,
            sequence,
            dstChain,
            dstAddr,
            payloadHash,
            attestationInfo.attestedTransceivers,
            UniversalAddressLibrary.fromAddress(msg.sender)
        );
    }

    /// @inheritdoc IRouterIntegrator
    function recvMessage(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external payable returns (uint128 enabledBitmap, uint128 attestedBitmap) {
        // sanity check that dstChain is this chain
        if (dstChain != ourChainId) {
            revert InvalidDestinationChain();
        }

        enabledBitmap = _getEnabledRecvTransceiversBitmapForChain(msg.sender, srcChain);
        if (enabledBitmap == 0) {
            revert TransceiverNotEnabled();
        }

        // compute the message digest
        bytes32 messageDigest = computeMessageDigest(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash);

        AttestationInfo storage attestationInfo = _getAttestationInfoStorage()[msg.sender][messageDigest];

        // revert if not in perIntegratorAttestations map
        if ((attestationInfo.attestedTransceivers == 0) && (!attestationInfo.executed)) {
            revert UnknownMessageAttestation();
        }

        // revert if already executed
        if (attestationInfo.executed) {
            revert AlreadyExecuted();
        }

        // set the executed flag in perIntegratorAttestations[dstAddr][digest]
        attestationInfo.executed = true;
        attestedBitmap = attestationInfo.attestedTransceivers;

        emit MessageReceived(
            messageDigest, srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash, enabledBitmap, attestedBitmap
        );
    }

    /// @inheritdoc IRouterIntegrator
    function getMessageStatus(uint16 srcChain, UniversalAddress srcAddr, uint64 sequence, bytes32 payloadHash)
        external
        view
        returns (uint128 enabledBitmap, uint128 attestedBitmap, bool executed)
    {
        return _getMessageStatus(
            srcChain,
            srcAddr,
            sequence,
            ourChainId,
            UniversalAddressLibrary.fromAddress(msg.sender),
            msg.sender,
            payloadHash
        );
    }

    /// @inheritdoc IRouterIntegrator
    function getMessageStatus(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external view returns (uint128 enabledBitmap, uint128 attestedBitmap, bool executed) {
        return _getMessageStatus(srcChain, srcAddr, sequence, dstChain, dstAddr, dstAddr.toAddress(), payloadHash);
    }

    /// @inheritdoc IRouterIntegrator
    function execMessage(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstAddr,
        bytes32 payloadHash
    ) external {
        if (dstChain != ourChainId) {
            revert InvalidDestinationChain();
        }
        // compute the message digest
        bytes32 messageDigest = computeMessageDigest(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash);

        AttestationInfo storage attestationInfo = _getAttestationInfoStorage()[msg.sender][messageDigest];

        if (attestationInfo.executed) {
            revert AlreadyExecuted();
        }
        attestationInfo.executed = true;
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

    /// @notice This is a common function that retrieves the status of a message.
    /// @param srcChain The Wormhole chain ID of the sender
    /// @param srcAddr The universal address of the peer on the sending chain
    /// @param sequence The sequence number of the message (per integrator)
    /// @param dstChain The Wormhole chain ID of the destination
    /// @param dstUAddr The destination universal address of the message
    /// @param dstAddr The destination address of the message
    /// @param payloadHash The keccak256 of payload from the integrator
    /// @return enabledBitmap A bitmap indicating enabled receive transceivers for the destination address.
    /// @return attestedBitmap A bitmap indicating attested transceivers for the message.
    /// @return executed A boolean indicating if the message has been executed.
    function _getMessageStatus(
        uint16 srcChain,
        UniversalAddress srcAddr,
        uint64 sequence,
        uint16 dstChain,
        UniversalAddress dstUAddr,
        address dstAddr,
        bytes32 payloadHash
    ) internal view returns (uint128 enabledBitmap, uint128 attestedBitmap, bool executed) {
        // sanity check that dstChain is this chain
        if (dstChain != ourChainId) {
            revert InvalidDestinationChain();
        }
        enabledBitmap = _getEnabledRecvTransceiversBitmapForChain(dstAddr, srcChain);
        // compute the message digest
        bytes32 messageDigest = computeMessageDigest(srcChain, srcAddr, sequence, dstChain, dstUAddr, payloadHash);

        AttestationInfo storage attestationInfo = _getAttestationInfoStorage()[dstAddr][messageDigest];

        attestedBitmap = attestationInfo.attestedTransceivers;

        executed = attestationInfo.executed;
    }
}
