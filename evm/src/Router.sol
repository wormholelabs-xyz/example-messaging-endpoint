// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "./interfaces/IRouter.sol";
import "./MessageSequence.sol";
import "./TransceiverRegistry.sol";
import "./interfaces/ITransceiver.sol";

contract Router is IRouter, MessageSequence, TransceiverRegistry {
    string public constant ROUTER_VERSION = "0.0.1";

    // =============== Events ================================================================

    /// @notice Emitted when an integrator registers a delegate.
    /// @param integrator The address of the integrator.
    /// @param delegate The address of the delegate.
    event DelegateRegistered(address integrator, address delegate);

    /// @notice Emitted when a message has been attested to.
    /// @param integrator The address of the integrator.
    /// @param transceiver The address of the transceiver.
    /// @param digest The digest of the message.
    event MessageAttestedTo(address integrator, address transceiver, bytes32 digest);

    /// @notice Emitted when a message has been sent.
    /// @param sender The address of the sender.
    /// @param recipient The address of the recipient.
    /// @param recipientChain The chainId of the recipient.
    /// @param digest The digest of the message.
    event MessageSent(address sender, address recipient, uint16 recipientChain, bytes32 digest);

    // =============== Errors ================================================================

    /// @notice Error when the transceiver is disabled.
    error TransceiverNotEnabled();

    /// @notice Error when the admin is the zero address.
    error InvalidAdminZeroAddress();

    /// @notice Error when the integrator did not register an admin.
    error IntegratorNotRegistered();

    /// @notice Error when the caller is not the registered admin.
    error CallerNotAdmin();

    // =============== Storage ===============================================================

    /// @dev Holds the integrator address to IntegratorConfig mapping.
    ///      mapping(address => IntegratorConfig)
    bytes32 private constant INTEGRATOR_CONFIGS_SLOT = bytes32(uint256(keccak256("registry.integratorConfigs")) - 1);

    /// @dev Integrator address => IntegratorConfig mapping.
    function _getIntegratorConfigsStorage() internal pure returns (mapping(address => IntegratorConfig) storage $) {
        uint256 slot = uint256(INTEGRATOR_CONFIGS_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    // =============== External ==============================================================

    /// @notice This is the first thing an integrator should do to register the admin address.
    ///         The admin address is used to manage the transceivers.
    /// @dev The msg.sender needs to be the integrator contract.
    /// @param admin The address of the admin.  Pass in msg.sender, if you want the integrator to be the admin.
    function registerAdmin(address admin) external {
        // Get the storage for this integrator contract
        mapping(address => IntegratorConfig) storage integratorConfigs = _getIntegratorConfigsStorage();
        // Do some checks?  Should address(0) mean something special?
        if (admin == address(0)) {
            revert InvalidAdminZeroAddress();
        }
        // Update the storage.
        integratorConfigs[msg.sender] = IntegratorConfig({isInitialized: true, admin: admin});
    }

    /// @notice The admin contract calls this function.
    /// @param integrator The address of the integrator contract.
    /// @param transceiver The address of the Transceiver contract.
    /// @param chain The chain ID of the Transceiver contract.
    function setSendTransceiver(address integrator, address transceiver, uint16 chain) external {
        _checkIntegratorAdmin(integrator, msg.sender);
        _setSendTransceiver(integrator, transceiver, chain);
    }

    /// @notice The admin contract uses this function.
    /// @param integrator The address of the integrator contract.
    /// @param transceiver The address of the Transceiver contract.
    /// @param chain The chain ID of the Transceiver contract.
    function setRecvTransceiver(address integrator, address transceiver, uint16 chain) external {
        _checkIntegratorAdmin(integrator, msg.sender);
        _setRecvTransceiver(integrator, transceiver, chain);
    }

    /// @inheritdoc IRouter
    function sendMessage(
        uint16 recipientChain,
        UniversalAddress recipientAddress,
        address refundAddress,
        bytes32 payloadHash
    ) external payable returns (uint64) {
        return _sendMessage(recipientChain, recipientAddress, refundAddress, payloadHash, msg.sender);
    }

    /// @dev Receive a message from another chain called by integrator.
    /// @param sourceChain The Wormhole chain ID of the recipient.
    /// @param senderAddress The universal address of the peer on the recipient chain.
    /// @param refundAddress The source chain refund address passed to the Transceiver.
    /// @param messageHash The hash of the message.
    /// @return uint128 The bitmap
    function receiveMessage(
        uint16 sourceChain,
        UniversalAddress senderAddress,
        address refundAddress,
        bytes32 messageHash
    ) external payable returns (uint128) {
        // Find the transceiver for the source chain.
        // address transceiver = this.getRecvTransceiverByChain(msg.sender, sourceChain);
        // Receive the message.
    }

    /// @inheritdoc IRouter
    function attestMessage(
        uint16 sourceChain, // Wormhole Chain ID
        UniversalAddress sourceAddress, // UniversalAddress of the message sender (integrator)
        uint64 sequence, // Next sequence number for that integrator (consuming the sequence number)
        uint16 destinationChainId, // Wormhole Chain ID
        UniversalAddress destinationAddress, // UniversalAddress of the messsage recipient (integrator on destination chain)
        bytes32 payloadHash // keccak256 of arbitrary payload from the integrator
    ) external {
        _attestMessage(sourceChain, sourceAddress, sequence, destinationChainId, destinationAddress, payloadHash);
    }

    // =============== Internal ==============================================================

    /// @notice This function checks that the integrator is registered.
    /// @dev This function will revert under the following conditions:
    ///         - The integrator is not registered
    /// @param integrator The integrator address
    function _checkIntegrator(address integrator) internal view {
        IntegratorConfig storage config = _getIntegratorConfigsStorage()[integrator];
        if (!config.isInitialized) {
            revert IntegratorNotRegistered();
        }
    }

    /// @notice This function checks that the integrator is registered and the admin is valid.
    /// @dev This function will revert under the following conditions:
    ///         - The integrator is not registered
    ///         - The admin is not configured for this integrator
    /// @param integrator The integrator address
    /// @param admin The admin address for this integrator
    function _checkIntegratorAdmin(address integrator, address admin) internal view {
        IntegratorConfig storage config = _getIntegratorConfigsStorage()[integrator];
        if (!config.isInitialized) {
            revert IntegratorNotRegistered();
        }

        if (config.admin != admin) {
            revert CallerNotAdmin();
        }
    }

    function _sendMessage(
        uint16 chainId,
        UniversalAddress recipientAddress,
        address refundAddress,
        bytes32 messageHash,
        address // sender
    ) internal returns (uint64 sequence) {
        _checkIntegrator(msg.sender);
        // get the next sequence number for msg.sender
        sequence = _useMessageSequence(msg.sender);
        // get the enabled send transceivers for [msg.sender][recipientChain]
        address[] memory sendTransceivers = this.getSendTransceiversByChain(msg.sender, chainId);
        if (sendTransceivers.length == 0) {
            revert TransceiverNotEnabled();
        }
        for (uint256 i = 0; i < sendTransceivers.length; i++) {
            // quote the delivery price
            uint256 deliveryPrice = ITransceiver(sendTransceivers[i]).quoteDeliveryPrice(chainId);
            // call sendMessage
            ITransceiver(sendTransceivers[i]).sendMessage{value: deliveryPrice}(
                chainId, messageHash, recipientAddress, UniversalAddressLibrary.fromAddress(refundAddress).toBytes32()
            );
        }
        // for each enabled transceiver
        //   quote the delivery price
        //     see https://github.com/wormhole-foundation/example-native-token-transfers/blob/68a7ca4132c74e838ac23e54752e8c0bc02bb4a2/evm/src/NttManager/ManagerBase.sol#L113
        //   call sendMessage
    }

    function _attestMessage(
        uint16 sourceChain, // Wormhole Chain ID
        UniversalAddress sourceAddress, // UniversalAddress of the message sender (integrator)
        uint64 sequence, // Next sequence number for that integrator (consuming the sequence number)
        uint16 destinationChainId, // Wormhole Chain ID
        UniversalAddress destinationAddress, // UniversalAddress of the messsage recipient (integrator on destination chain)
        bytes32 payloadHash // keccak256 of arbitrary payload from the integrator
    ) internal {
        _checkIntegrator(msg.sender);
        // sanity check that destinationChainId is this chain
        // get enabled recv transceivers for [destinationAddress][sourceChain]
        // address transceiver = this.getRecvTransceiverByChain(sourceChain);
        // check that msg.sender is one of those transceivers
        // compute the message digest
        // set the bit in perIntegratorAttestations[destinationAddress][digest] corresponding to msg.sender
    }
}
