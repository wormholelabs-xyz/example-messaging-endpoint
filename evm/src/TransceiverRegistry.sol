// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

/// @title TransceiverRegistry
/// @notice This contract is responsible for handling the registration of Transceivers.
/// @dev This contract checks that a few critical invariants hold when transceivers are added or removed,
///      including:
///         1. If a transceiver is not registered, it should be not enabled.
///         2. The value set in the bitmap of transceivers
///            should directly correspond to the whether the transceiver is enabled
abstract contract TransceiverRegistry {
    /// @dev Information about registered transceivers.
    struct TransceiverInfo {
        // whether this transceiver is registered
        bool registered;
        uint8 index; // the index into the integrator's transceivers array
    }

    /// @dev Bitmap encoding the enabled transceivers.
    /// invariant: forall (i: uint8), enabledTransceiverBitmap & i == 1 <=> transceiverInfos[i].enabled
    struct _EnabledTransceiverBitmap {
        uint128 bitmap; // MAX_TRANSCEIVERS = 128
    }

    /// @dev Total number of registered transceivers. This number can only increase.
    /// invariant: numRegisteredTransceivers <= MAX_TRANSCEIVERS
    /// invariant: forall (i: uint8),
    ///   i < numRegisteredTransceivers <=> exists (a: address), transceiverInfos[a].index == i
    struct _NumTransceivers {
        uint8 registered;
    }

    uint8 constant MAX_TRANSCEIVERS = 128;

    // =============== Events ===============================================

    /// @notice Emitted when a transceiver is added.
    /// @dev Topic0
    ///      0x11b137fe0ddc829607ddb73c998c8792af425d23c3d44235e97ba9d0ded66e58
    /// @param integrator The address of the integrator.
    /// @param chain The Wormhole chain ID on which this transceiver is added.
    /// @param transceiver The address of the transceiver.
    /// @param transceiversNum The current number of transceivers.
    event TransceiverAdded(address integrator, uint16 chain, address transceiver, uint8 transceiversNum);

    /// @notice Emitted when a send side transceiver is enabled for a chain.
    /// @dev Topic0
    ///      0x1e8617217e121e5aee2e06d784ac4dab35309adecb2a18f98eaf8c430e19a5c3
    /// @param integrator The address of the integrator.
    /// @param chain The Wormhole chain ID on which this transceiver is enabled.
    /// @param transceiver The address of the transceiver.
    event SendTransceiverEnabledForChain(address integrator, uint16 chain, address transceiver);

    /// @notice Emitted when a receive side transceiver is enabled for a chain.
    /// @dev Topic0
    ///      0x3e9ae7f2b6957091d9e99a42a88cf2e8da98b142a61811ac0e9e41f2f9778fbc
    /// @param integrator The address of the integrator.
    /// @param chain The Wormhole chain ID on which this transceiver is enabled.
    /// @param transceiver The address of the transceiver.
    event RecvTransceiverEnabledForChain(address integrator, uint16 chain, address transceiver);

    /// @notice Emitted when a send side transceiver is removed from the router.
    /// @dev Topic0
    ///      0xb8844d856d7b255f06b1c28ae0324984a00923b2e98616302766622c20e37fac
    /// @param integrator The address of the integrator.
    /// @param chain The Wormhole chain ID on which this transceiver is disabled.
    /// @param transceiver The address of the transceiver.
    event SendTransceiverDisabledForChain(address integrator, uint16 chain, address transceiver);

    /// @notice Emitted when a receive side transceiver is removed from the router.
    /// @dev Topic0
    ///      0x205d0d0e655937210435fc177252accf5845b3c05787d7374023e44970730d33
    /// @param integrator The address of the integrator.
    /// @param chain The Wormhole chain ID on which this transceiver is disabled.
    /// @param transceiver The address of the transceiver.
    event RecvTransceiverDisabledForChain(address integrator, uint16 chain, address transceiver);

    // =============== Errors ===============================================

    /// @notice Error when the caller is not the transceiver.
    /// @dev Selector: 0xa0ae911d.
    /// @param caller The address of the caller.
    error CallerNotTransceiver(address caller);

    /// @notice Error when the transceiver is the zero address.
    /// @dev Selector: 0x2f44bd77.
    error InvalidTransceiverZeroAddress();

    /// @notice Error when the transceiver is disabled.
    /// @dev Selector: 0xa64030ff.
    error TransceiverAlreadyDisabled(address transceiver);

    /// @notice Error when the number of registered transceivers
    ///         exceeeds (MAX_TRANSCEIVERS = 128).
    /// @dev Selector: 0x891684c3.
    error TooManyTransceivers();

    /// @notice Error when attempting to use an unregistered transceiver
    ///         that is not registered.
    /// @dev Selector: 0x891684c3.
    /// @param transceiver The address of the transceiver.
    error NonRegisteredTransceiver(address transceiver);

    /// @notice Error when attempting to use an incorrect chain
    /// @dev Selector: 0x587c94c3.
    /// @param chain The id of the incorrect chain
    error InvalidChain(uint16 chain);

    /// @notice Error when attempting to register a transceiver that is already register.
    /// @dev Selector: 0xeaac8f97.
    /// @param transceiver The address of the transceiver.
    error TransceiverAlreadyRegistered(address transceiver);

    /// @notice Error when attempting to enable a transceiver that is already enabled.
    /// @dev Selector: 0x8d68f84d.
    /// @param transceiver The address of the transceiver.
    error TransceiverAlreadyEnabled(address transceiver);

    // =============== Storage ===============================================

    /// @dev Holds the integrator address to transceiver address to TransceiverInfo mapping.
    ///      mapping(address => mapping(address => TransceiverInfo))
    bytes32 private constant TRANSCEIVER_INFOS_SLOT = bytes32(uint256(keccak256("registry.transceiverInfos")) - 1);

    /// @dev Holds send side Integrator address => Transceiver addresses mapping.
    ///      mapping(address => address[]) across all chains
    bytes32 private constant REGISTERED_TRANSCEIVERS_SLOT =
        bytes32(uint256(keccak256("registry.registeredTransceivers")) - 1);

    /// @dev Holds send side Integrator address => NumTransceivers mapping.
    ///      mapping(address => _NumTransceivers)
    bytes32 private constant NUM_REGISTERED_TRANSCEIVERS_SLOT =
        bytes32(uint256(keccak256("registry.numRegisteredTransceivers")) - 1);

    // =============== Send side =============================================

    /// @dev Holds integrator address => Chain ID => Enabled send side transceiver address[] mapping.
    ///      mapping(address => mapping(uint16 => address[]))
    bytes32 private constant ENABLED_SEND_TRANSCEIVER_ARRAY_SLOT =
        bytes32(uint256(keccak256("registry.sendTransceiverArray")) - 1);

    /// @dev Holds send side Integrator address => Transceiver addresses mapping.
    ///      mapping(address => address[]) across all chains
    bytes32 private constant REGISTERED_SEND_TRANSCEIVERS_SLOT =
        bytes32(uint256(keccak256("registry.registeredSendTransceivers")) - 1);

    // =============== Recv side =============================================

    /// @dev Holds integrator address => Chain ID => Enabled transceiver receive side bitmap mapping.
    ///      mapping(address => mapping(uint16 => uint128))
    bytes32 private constant ENABLED_RECV_TRANSCEIVER_BITMAP_SLOT =
        bytes32(uint256(keccak256("registry.recvTransceiverBitmap")) - 1);

    /// @dev Holds receive side Integrator address => Transceiver addresses mapping.
    ///      mapping(address => address[]) across all chains
    bytes32 private constant REGISTERED_RECV_TRANSCEIVERS_SLOT =
        bytes32(uint256(keccak256("registry.registeredRecvTransceivers")) - 1);

    // =============== Mappings ===============================================

    /// @dev Integrator address => transceiver address => TransceiverInfo mapping.
    function _getTransceiverInfosStorage()
        internal
        pure
        returns (mapping(address => mapping(address => TransceiverInfo)) storage $)
    {
        uint256 slot = uint256(TRANSCEIVER_INFOS_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    /// @dev Integrator address => Chain ID => Enabled transceiver bitmap mapping.
    function _getPerChainSendTransceiverArrayStorage()
        private
        pure
        returns (mapping(address => mapping(uint16 => address[])) storage $)
    {
        uint256 slot = uint256(ENABLED_SEND_TRANSCEIVER_ARRAY_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    /// @dev Integrator address => Chain ID => Enabled transceiver bitmap mapping.
    function _getPerChainRecvTransceiverBitmapStorage()
        private
        pure
        returns (mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage $)
    {
        uint256 slot = uint256(ENABLED_RECV_TRANSCEIVER_BITMAP_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    /// @dev Integrator address => Transceiver address[] mapping.
    ///      Contains all registered transceivers for this integrator.
    function _getRegisteredTransceiversStorage() internal pure returns (mapping(address => address[]) storage $) {
        uint256 slot = uint256(REGISTERED_TRANSCEIVERS_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    /// @dev Integrator address => NumTransceivers mapping.
    ///      Contains number of registered transceivers for this integrator.
    ///      The transceivers may or may not be enabled.
    function _getNumTransceiversStorage() internal pure returns (mapping(address => _NumTransceivers) storage $) {
        uint256 slot = uint256(NUM_REGISTERED_TRANSCEIVERS_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    // =============== Modifiers ======================================================

    /// @notice This modifier will revert if the transceiver is a zero address or the chain is invalid
    /// @param transceiver The transceiver address
    /// @param chain The Wormhole chain ID
    modifier onlyValidTransceiverAndChain(address transceiver, uint16 chain) {
        if (transceiver == address(0)) {
            revert InvalidTransceiverZeroAddress();
        }
        if (chain == 0) {
            revert InvalidChain(chain);
        }
        _;
    }

    /// @notice This modifier will revert if the transceiver is an invalid address or not registered.
    ///         Or the chain is invalid
    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID
    /// @param transceiver The transceiver address
    modifier onlyRegisteredTransceiver(address integrator, uint16 chain, address transceiver) {
        if (transceiver == address(0)) {
            revert InvalidTransceiverZeroAddress();
        }

        if (chain == 0) {
            revert InvalidChain(chain);
        }

        if (!_getTransceiverInfosStorage()[integrator][transceiver].registered) {
            revert NonRegisteredTransceiver(transceiver);
        }
        _;
    }

    // =============== Storage Getters/Setters ========================================

    /// @dev This function adds a transceiver.
    /// @param integrator The integrator address
    /// @param chain The The Wormhole chain ID
    /// @param transceiver The transceiver address
    /// @return index The index of this newly enabled transceiver
    function _addTransceiver(address integrator, uint16 chain, address transceiver)
        internal
        onlyValidTransceiverAndChain(transceiver, chain)
        returns (uint8 index)
    {
        mapping(address => mapping(address => TransceiverInfo)) storage transceiverInfos = _getTransceiverInfosStorage();
        mapping(address => _NumTransceivers) storage _numTransceivers = _getNumTransceiversStorage();

        if (transceiverInfos[integrator][transceiver].registered) {
            revert TransceiverAlreadyRegistered(transceiver);
        }
        if (_numTransceivers[integrator].registered >= MAX_TRANSCEIVERS) {
            revert TooManyTransceivers();
        }
        // Create the TransceiverInfo
        transceiverInfos[integrator][transceiver] =
            TransceiverInfo({registered: true, index: _numTransceivers[integrator].registered});
        // Add this transceiver to the integrator => address[] mapping
        _getRegisteredTransceiversStorage()[integrator].push(transceiver);
        // Increment count of transceivers
        _numTransceivers[integrator].registered++;
        // Emit an event
        emit TransceiverAdded(integrator, chain, transceiver, _numTransceivers[integrator].registered);

        return transceiverInfos[integrator][transceiver].index;
    }

    /// @dev It is assumed that the integrator address is already validated (and not 0)
    ///      This just enables the send side transceiver.  It does not register it.
    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID
    /// @param transceiver The transceiver address
    function _enableSendTransceiver(address integrator, uint16 chain, address transceiver)
        internal
        onlyRegisteredTransceiver(integrator, chain, transceiver)
    {
        if (_isSendTransceiverEnabledForChain(integrator, chain, transceiver)) {
            revert TransceiverAlreadyEnabled(transceiver);
        }
        mapping(address => mapping(uint16 => address[])) storage sendTransceiverArray =
            _getPerChainSendTransceiverArrayStorage();
        sendTransceiverArray[integrator][chain].push(transceiver);
        emit SendTransceiverEnabledForChain(integrator, chain, transceiver);
    }

    /// @dev It is assumed that the integrator address is already validated (and not 0)
    ///      This just enables the receive side transceiver.  It does not register it.
    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID
    /// @param transceiver The transceiver address
    function _enableRecvTransceiver(address integrator, uint16 chain, address transceiver)
        internal
        onlyRegisteredTransceiver(integrator, chain, transceiver)
    {
        if (_isRecvTransceiverEnabledForChain(integrator, chain, transceiver)) {
            revert TransceiverAlreadyEnabled(transceiver);
        }
        uint8 index = _getTransceiverInfosStorage()[integrator][transceiver].index;
        mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _bitmaps =
            _getPerChainRecvTransceiverBitmapStorage();
        _bitmaps[integrator][chain].bitmap |= uint128(1 << index);
        emit RecvTransceiverEnabledForChain(integrator, chain, transceiver);
    }

    /// @notice This function disables a send side transceiver by chain.
    /// @param integrator The integrator address
    /// @param chain The chain ID
    /// @param transceiver The transceiver address
    function _disableSendTransceiver(address integrator, uint16 chain, address transceiver)
        internal
        onlyRegisteredTransceiver(integrator, chain, transceiver)
    {
        // mapping(address => mapping(address => TransceiverInfo)) storage transceiverInfos = _getTransceiverInfosStorage();
        mapping(address => mapping(uint16 => address[])) storage enabledSendTransceivers =
            _getPerChainSendTransceiverArrayStorage();
        address[] storage transceivers = enabledSendTransceivers[integrator][chain];

        // Get the index of the disabled transceiver in the enabled transceivers array
        // and replace it with the last element in the array
        uint256 len = transceivers.length;
        bool found = false;
        for (uint256 i = 0; i < len;) {
            if (transceivers[i] == transceiver) {
                // Swap the last element with the element to be removed
                transceivers[i] = transceivers[len - 1];
                // Remove the last element
                transceivers.pop();
                found = true;
                break;
            }
            unchecked {
                ++i;
            }
        }
        if (!found) {
            revert TransceiverAlreadyDisabled(transceiver);
        }

        emit SendTransceiverDisabledForChain(integrator, chain, transceiver);
    }

    /// @dev This function disables a receive side transceiver by chain.
    /// @notice This function will revert under the following conditions:
    ///         - The transceiver is the zero address
    ///         - The transceiver is not registered
    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID
    /// @param transceiver The transceiver address
    function _disableRecvTransceiver(address integrator, uint16 chain, address transceiver)
        internal
        onlyRegisteredTransceiver(integrator, chain, transceiver)
    {
        mapping(address => mapping(address => TransceiverInfo)) storage transceiverInfos = _getTransceiverInfosStorage();
        mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _enabledTransceiverBitmap =
            _getPerChainRecvTransceiverBitmapStorage();

        uint128 updatedEnabledTransceiverBitmap = _enabledTransceiverBitmap[integrator][chain].bitmap
            & uint128(~(1 << transceiverInfos[integrator][transceiver].index));
        // ensure that this actually changed the bitmap
        if (updatedEnabledTransceiverBitmap >= _enabledTransceiverBitmap[integrator][chain].bitmap) {
            revert TransceiverAlreadyDisabled(transceiver);
        }
        _enabledTransceiverBitmap[integrator][chain].bitmap = updatedEnabledTransceiverBitmap;

        emit RecvTransceiverDisabledForChain(integrator, chain, transceiver);
    }

    /// @dev Returns if the send side transceiver is enabled for the given integrator and chain.
    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID
    /// @param transceiver The transceiver address
    /// @return true if the transceiver is enabled, false otherwise.
    function _isSendTransceiverEnabledForChain(address integrator, uint16 chain, address transceiver)
        internal
        view
        onlyRegisteredTransceiver(integrator, chain, transceiver)
        returns (bool)
    {
        address[] storage transceivers = _getPerChainSendTransceiverArrayStorage()[integrator][chain];
        uint256 length = transceivers.length;
        for (uint256 i = 0; i < length;) {
            if (transceivers[i] == transceiver) {
                return true;
            }
            unchecked {
                ++i;
            }
        }
        return false;
    }

    /// @dev Returns if the receive side transceiver is enabled for the given integrator and chain.
    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID
    /// @param transceiver The transceiver address
    /// @return true if the transceiver is enabled, false otherwise.
    function _isRecvTransceiverEnabledForChain(address integrator, uint16 chain, address transceiver)
        internal
        view
        onlyRegisteredTransceiver(integrator, chain, transceiver)
        returns (bool)
    {
        uint128 bitmap = _getEnabledRecvTransceiversBitmapForChain(integrator, chain);
        uint8 index = _getTransceiverInfosStorage()[integrator][transceiver].index;
        return (bitmap & uint128(1 << index)) != 0;
    }

    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID
    /// @return array The array of the send side transceivers enabled for this integrator and chain
    function _getEnabledSendTransceiversArrayForChain(address integrator, uint16 chain)
        internal
        view
        virtual
        returns (address[] storage array)
    {
        if (chain == 0) {
            revert InvalidChain(chain);
        }
        array = _getPerChainSendTransceiverArrayStorage()[integrator][chain];
    }

    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID
    /// @return bitmap The bitmap of the send side transceivers enabled for this integrator and chain
    function _getEnabledRecvTransceiversBitmapForChain(address integrator, uint16 chain)
        internal
        view
        virtual
        returns (uint128 bitmap)
    {
        if (chain == 0) {
            revert InvalidChain(chain);
        }
        bitmap = _getPerChainRecvTransceiverBitmapStorage()[integrator][chain].bitmap;
    }

    // =============== EXTERNAL FUNCTIONS ========================================

    /// @notice Returns the registered send side transceiver addresses for the given integrator.
    /// @param integrator The integrator address
    /// @return result The registered transceivers for the given integrator.
    function getTransceivers(address integrator) external view returns (address[] memory result) {
        result = _getRegisteredTransceiversStorage()[integrator];
    }

    /// @notice Returns the enabled send side transceiver addresses for the given integrator.
    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID for the desired transceivers
    /// @return result The enabled send side transceivers for the given integrator and chain.
    function getSendTransceiversByChain(address integrator, uint16 chain)
        public
        view
        returns (address[] memory result)
    {
        if (chain == 0) {
            revert InvalidChain(chain);
        }
        address[] memory allTransceivers = _getRegisteredTransceiversStorage()[integrator];
        address[] memory tempResult = new address[](allTransceivers.length);
        uint8 len = 0;
        uint256 allTransceiversLength = allTransceivers.length;
        for (uint256 i = 0; i < allTransceiversLength;) {
            if (_isSendTransceiverEnabledForChain(integrator, chain, allTransceivers[i])) {
                tempResult[len] = allTransceivers[i];
                ++len;
            }
            unchecked {
                ++i;
            }
        }
        result = new address[](len);
        for (uint8 i = 0; i < len; i++) {
            result[i] = tempResult[i];
        }
    }

    /// @notice Returns the enabled receive side transceiver addresses for the given integrator.
    /// @param integrator The integrator address
    /// @param chain The Wormhole chain ID for the desired transceivers
    /// @return result The enabled receive side transceivers for the given integrator.
    function getRecvTransceiversByChain(address integrator, uint16 chain)
        public
        view
        returns (address[] memory result)
    {
        address[] memory allTransceivers = _getRegisteredTransceiversStorage()[integrator];
        // Count number of bits set in the bitmap so we can calculate the size of the result array.
        uint128 bitmap = _getEnabledRecvTransceiversBitmapForChain(integrator, chain);
        uint8 count = 0;
        while (bitmap != 0) {
            count += uint8(bitmap & 1);
            bitmap >>= 1;
        }
        result = new address[](count);
        uint256 len = 0;
        for (uint256 i = 0; i < allTransceivers.length; i++) {
            if (_isRecvTransceiverEnabledForChain(integrator, chain, allTransceivers[i])) {
                result[len] = allTransceivers[i];
                ++len;
            }
        }
    }
}
