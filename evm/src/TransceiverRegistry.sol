// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

/// @title TransceiverRegistry
/// @notice This contract is responsible for handling the registration of Transceivers.
/// @dev This contract checks that a few critical invariants hold when transceivers are added or removed,
///      including:
///         1. If a transceiver is not registered, it should be enabled.
///         2. The value set in the bitmap of transceivers
///            should directly correspond to the whether the transceiver is enabled
abstract contract TransceiverRegistry {
    constructor() {}

    /// @dev Information about registered transceivers.
    struct TransceiverInfo {
        // whether this transceiver is registered
        bool registered;
        uint8 index; // the index into the integrator's transceivers array
    }

    // TODO: Does this need to be a struct?
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

    struct IntegratorConfig {
        bool isInitialized;
        address admin;
    }

    uint8 constant MAX_TRANSCEIVERS = 128;

    // =============== Events ===============================================

    /// @notice Emitted when a send side transceiver is added.
    /// @param integrator The address of the integrator.
    /// @param transceiver The address of the transceiver.
    /// @param chainId The chain to which the threshold applies.
    /// @param transceiversNum The current number of transceivers.
    event SendTransceiverAdded(address integrator, address transceiver, uint16 chainId, uint64 transceiversNum);

    /// @notice Emitted when a receive side transceiver is added.
    /// @param integrator The address of the integrator.
    /// @param transceiver The address of the transceiver.
    /// @param chainId The chain to which the threshold applies.
    /// @param transceiversNum The current number of transceivers.
    event RecvTransceiverAdded(address integrator, address transceiver, uint16 chainId, uint64 transceiversNum);

    /// @notice Emitted when a send side transceiver is enabled for a chain.
    /// @param integrator The address of the integrator.
    /// @param transceiver The address of the transceiver.
    /// @param chainId The chain to which the threshold applies.
    event SendTransceiverEnabledForChain(address integrator, address transceiver, uint16 chainId);

    /// @notice Emitted when a receive side transceiver is enabled for a chain.
    /// @param integrator The address of the integrator.
    /// @param transceiver The address of the transceiver.
    /// @param chainId The chain to which the threshold applies.
    event RecvTransceiverEnabledForChain(address integrator, address transceiver, uint16 chainId);

    /// @notice Emitted when a send side transceiver is removed from the nttManager.
    /// @param integrator The address of the integrator.
    /// @param transceiver The address of the transceiver.
    /// @param chainId The chain to which the threshold applies.
    event SendTransceiverDisabled(address integrator, address transceiver, uint16 chainId);

    /// @notice Emitted when a receive side transceiver is removed from the nttManager.
    /// @param integrator The address of the integrator.
    /// @param transceiver The address of the transceiver.
    /// @param chainId The chain to which the threshold applies.
    event RecvTransceiverDisabled(address integrator, address transceiver, uint16 chainId);

    // =============== Errors ===============================================

    /// @notice Error when the caller is not the transceiver.
    /// @param caller The address of the caller.
    error CallerNotTransceiver(address caller);

    /// @notice Error when the transceiver is the zero address.
    error InvalidTransceiverZeroAddress();

    /// @notice Error when the transceiver is disabled.
    error DisabledTransceiver(address transceiver);

    /// @notice Error when the number of registered transceivers
    ///         exceeeds (MAX_TRANSCEIVERS = 64).
    error TooManyTransceivers();

    /// @notice Error when attempting to remove a transceiver
    ///         that is not registered.
    /// @param transceiver The address of the transceiver.
    error NonRegisteredTransceiver(address transceiver);

    /// @notice Error when attempting to use an incorrect chain
    /// @param chain The id of the incorrect chain
    error InvalidChain(uint16 chain);

    /// @notice Error when attempting to enable a transceiver that is already enabled.
    /// @param transceiver The address of the transceiver.
    error TransceiverAlreadyEnabled(address transceiver);

    // TODO:  Not sure if I need this, yet. Will add if Router.sol needs it.
    // modifier onlyTransceiver() {
    //     if (!_getTransceiverInfosStorage()[msg.sender].enabled) {
    //         revert CallerNotTransceiver(msg.sender);
    //     }
    //     _;
    // }

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

    /// @dev Holds send side integrator address => Chain ID => Enabled transceiver bitmap mapping.
    ///      mapping(address => mapping(uint16 => uint128))
    bytes32 private constant ENABLED_SEND_TRANSCEIVER_BITMAP_SLOT =
        bytes32(uint256(keccak256("registry.sendTransceiverBitmap")) - 1);

    /// @dev Holds send side Integrator address => Transceiver addresses mapping.
    ///      mapping(address => address[]) across all chains
    bytes32 private constant REGISTERED_SEND_TRANSCEIVERS_SLOT =
        bytes32(uint256(keccak256("registry.registeredSendTransceivers")) - 1);

    // =============== Recv side =============================================

    /// @dev Holds receive side integrator address => Chain ID => Enabled transceiver bitmap mapping.
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
    function _getPerChainSendTransceiverBitmapStorage()
        private
        pure
        returns (mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage $)
    {
        uint256 slot = uint256(ENABLED_SEND_TRANSCEIVER_BITMAP_SLOT);
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

    // =============== Storage Getters/Setters ========================================

    /// @dev Returns if the send side transceiver is enabled for the given integrator and chain.
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    /// @param chainId The chain ID
    /// @return true if the transceiver is enabled, false otherwise.
    function _isSendTransceiverEnabledForChain(address integrator, address transceiver, uint16 chainId)
        internal
        view
        returns (bool)
    {
        uint128 bitmap = _getEnabledSendTransceiversBitmapForChain(integrator, chainId);
        return _isTransceiverEnabledForChain(integrator, transceiver, bitmap);
    }

    /// @dev Returns if the receive side transceiver is enabled for the given integrator and chain.
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    /// @param chainId The chain ID
    /// @return true if the transceiver is enabled, false otherwise.
    function _isRecvTransceiverEnabledForChain(address integrator, address transceiver, uint16 chainId)
        internal
        view
        returns (bool)
    {
        uint128 bitmap = _getEnabledRecvTransceiversBitmapForChain(integrator, chainId);
        return _isTransceiverEnabledForChain(integrator, transceiver, bitmap);
    }

    /// @dev This is a common function between send/receive transceivers.
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    /// @return true if the transceiver is enabled, false otherwise.
    function _isTransceiverEnabledForChain(address integrator, address transceiver, uint128 bitmap)
        internal
        view
        returns (bool)
    {
        if (transceiver == address(0)) {
            revert InvalidTransceiverZeroAddress();
        }
        uint8 index = _getTransceiverInfosStorage()[integrator][transceiver].index;
        return (bitmap & uint128(1 << index)) != 0;
    }

    /// @dev This function will revert if the transceiver is an invalid address or not registered.
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    function _checkTransceiver(address integrator, address transceiver) internal view {
        if (transceiver == address(0)) {
            revert InvalidTransceiverZeroAddress();
        }

        if (!_getTransceiverInfosStorage()[integrator][transceiver].registered) {
            revert NonRegisteredTransceiver(transceiver);
        }
    }

    /// @dev It is assumed that the integrator address is already validated (and not 0)
    ///      This just enables the send side transceiver.  It does not register it.
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    /// @param chainId The chain ID
    function _enableSendTransceiverForChain(address integrator, address transceiver, uint16 chainId) internal {
        _checkTransceiver(integrator, transceiver);

        uint8 index = _getTransceiverInfosStorage()[integrator][transceiver].index;
        mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _bitmaps =
            _getPerChainSendTransceiverBitmapStorage();
        _bitmaps[integrator][chainId].bitmap |= uint128(1 << index);
    }

    /// @dev It is assumed that the integrator address is already validated (and not 0)
    ///      This just enables the receive side transceiver.  It does not register it.
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    /// @param chainId The chain ID
    function _enableRecvTransceiverForChain(address integrator, address transceiver, uint16 chainId) internal {
        _checkTransceiver(integrator, transceiver);

        uint8 index = _getTransceiverInfosStorage()[integrator][transceiver].index;
        mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _bitmaps =
            _getPerChainRecvTransceiverBitmapStorage();
        _bitmaps[integrator][chainId].bitmap |= uint128(1 << index);
    }

    /// @dev This function enables a send side transceiver.  If it is not registered, it will register it.
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    /// @param chainId The chain ID
    /// @return index The index of this newly enabled send side transceiver
    function _setSendTransceiver(address integrator, address transceiver, uint16 chainId)
        internal
        returns (uint8 index)
    {
        // These are everything for an integrator.
        mapping(address => mapping(address => TransceiverInfo)) storage transceiverInfos = _getTransceiverInfosStorage();
        mapping(address => _NumTransceivers) storage _numTransceivers = _getNumTransceiversStorage();
        // This is send side for a specific chain.
        mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _enabledTransceiverBitmap =
            _getPerChainSendTransceiverBitmapStorage();

        if (transceiver == address(0)) {
            revert InvalidTransceiverZeroAddress();
        }

        if (chainId == 0) {
            revert InvalidChain(chainId);
        }

        if (!transceiverInfos[integrator][transceiver].registered) {
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
            emit SendTransceiverAdded(integrator, transceiver, chainId, _numTransceivers[integrator].registered);
        }

        // _numTransceivers[integrator].enabled++;

        // Add this transceiver to the per chain list of transceivers by updating the bitmap
        uint128 updatedEnabledTransceiverBitmap = _enabledTransceiverBitmap[integrator][chainId].bitmap
            | uint128(1 << transceiverInfos[integrator][transceiver].index);
        // ensure that this actually changed the bitmap
        if (updatedEnabledTransceiverBitmap == _enabledTransceiverBitmap[integrator][chainId].bitmap) {
            revert TransceiverAlreadyEnabled(transceiver);
        }
        _enabledTransceiverBitmap[integrator][chainId].bitmap = updatedEnabledTransceiverBitmap;

        _checkSendTransceiversInvariants(integrator);
        emit SendTransceiverEnabledForChain(integrator, transceiver, chainId);

        return transceiverInfos[integrator][transceiver].index;
    }

    /// @dev This function enables a transceiver.  If it is not registered, it will register it.
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    /// @param chainId The chain ID
    /// @return index The index of this newly enabled receive side transceiver
    function _setRecvTransceiver(address integrator, address transceiver, uint16 chainId)
        internal
        returns (uint8 index)
    {
        // These are everything for an integrator.
        mapping(address => mapping(address => TransceiverInfo)) storage transceiverInfos = _getTransceiverInfosStorage();
        mapping(address => _NumTransceivers) storage _numTransceivers = _getNumTransceiversStorage();
        // This is send side for a specific chain.
        mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _enabledTransceiverBitmap =
            _getPerChainRecvTransceiverBitmapStorage();

        if (transceiver == address(0)) {
            revert InvalidTransceiverZeroAddress();
        }

        if (chainId == 0) {
            revert InvalidChain(chainId);
        }

        if (!transceiverInfos[integrator][transceiver].registered) {
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
            emit RecvTransceiverAdded(integrator, transceiver, chainId, _numTransceivers[integrator].registered);
        }

        // _numTransceivers[integrator].enabled++;

        // Add this transceiver to the per chain list of transceivers by updating the bitmap
        uint128 updatedEnabledTransceiverBitmap = _enabledTransceiverBitmap[integrator][chainId].bitmap
            | uint128(1 << transceiverInfos[integrator][transceiver].index);
        // ensure that this actually changed the bitmap
        if (updatedEnabledTransceiverBitmap == _enabledTransceiverBitmap[integrator][chainId].bitmap) {
            revert TransceiverAlreadyEnabled(transceiver);
        }
        _enabledTransceiverBitmap[integrator][chainId].bitmap = updatedEnabledTransceiverBitmap;

        _checkRecvTransceiversInvariants(integrator);
        emit RecvTransceiverEnabledForChain(integrator, transceiver, chainId);

        return transceiverInfos[integrator][transceiver].index;
    }

    /// @dev This function disables a send side transceiver by chain.
    /// @notice This function will revert under the following conditions:
    ///         - The transceiver is the zero address
    ///         - The transceiver is not registered
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    /// @param chainId The chain ID
    function _disableSendTransceiver(address integrator, address transceiver, uint16 chainId) internal {
        mapping(address => mapping(address => TransceiverInfo)) storage transceiverInfos = _getTransceiverInfosStorage();
        mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _enabledTransceiverBitmap =
            _getPerChainSendTransceiverBitmapStorage();

        if (transceiver == address(0)) {
            revert InvalidTransceiverZeroAddress();
        }

        if (chainId == 0) {
            revert InvalidChain(chainId);
        }

        TransceiverInfo storage info = transceiverInfos[integrator][transceiver];

        if (!info.registered) {
            revert NonRegisteredTransceiver(transceiver);
        }

        uint128 updatedEnabledTransceiverBitmap = _enabledTransceiverBitmap[integrator][chainId].bitmap
            & uint128(~(1 << transceiverInfos[integrator][transceiver].index));
        // ensure that this actually changed the bitmap
        if (updatedEnabledTransceiverBitmap >= _enabledTransceiverBitmap[integrator][chainId].bitmap) {
            revert DisabledTransceiver(transceiver);
        }
        _enabledTransceiverBitmap[integrator][chainId].bitmap = updatedEnabledTransceiverBitmap;

        _checkSendTransceiversInvariants(integrator);
        // we call the invariant check on the transceiver here as well, since
        // the above check only iterates through the enabled transceivers.
        _checkSendTransceiverInvariants(integrator, transceiver);
        emit SendTransceiverDisabled(integrator, transceiver, chainId);
    }

    /// @dev This function disables a receive side transceiver by chain.
    /// @notice This function will revert under the following conditions:
    ///         - The transceiver is the zero address
    ///         - The transceiver is not registered
    /// @param integrator The integrator address
    /// @param transceiver The transceiver address
    /// @param chainId The chain ID
    function _disableRecvTransceiver(address integrator, address transceiver, uint16 chainId) internal {
        mapping(address => mapping(address => TransceiverInfo)) storage transceiverInfos = _getTransceiverInfosStorage();
        mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _enabledTransceiverBitmap =
            _getPerChainRecvTransceiverBitmapStorage();

        if (transceiver == address(0)) {
            revert InvalidTransceiverZeroAddress();
        }

        if (chainId == 0) {
            revert InvalidChain(chainId);
        }

        TransceiverInfo storage info = transceiverInfos[integrator][transceiver];

        if (!info.registered) {
            revert NonRegisteredTransceiver(transceiver);
        }

        uint128 updatedEnabledTransceiverBitmap = _enabledTransceiverBitmap[integrator][chainId].bitmap
            & uint128(~(1 << transceiverInfos[integrator][transceiver].index));
        // ensure that this actually changed the bitmap
        if (updatedEnabledTransceiverBitmap >= _enabledTransceiverBitmap[integrator][chainId].bitmap) {
            revert DisabledTransceiver(transceiver);
        }
        _enabledTransceiverBitmap[integrator][chainId].bitmap = updatedEnabledTransceiverBitmap;

        _checkRecvTransceiversInvariants(integrator);
        // we call the invariant check on the transceiver here as well, since
        // the above check only iterates through the enabled transceivers.
        _checkRecvTransceiverInvariants(integrator, transceiver);
        emit RecvTransceiverDisabled(integrator, transceiver, chainId);
    }

    /// @param integrator The integrator address
    /// @param forChainId The chain ID
    /// @return bitmap The bitmap of the send side transceivers enabled for this integrator and chain
    function _getEnabledSendTransceiversBitmapForChain(address integrator, uint16 forChainId)
        internal
        view
        virtual
        returns (uint128 bitmap)
    {
        if (forChainId == 0) {
            revert InvalidChain(forChainId);
        }
        bitmap = _getPerChainSendTransceiverBitmapStorage()[integrator][forChainId].bitmap;
    }

    /// @param integrator The integrator address
    /// @param forChainId The chain ID
    /// @return bitmap The bitmap of the send side transceivers enabled for this integrator and chain
    function _getEnabledRecvTransceiversBitmapForChain(address integrator, uint16 forChainId)
        internal
        view
        virtual
        returns (uint128 bitmap)
    {
        if (forChainId == 0) {
            revert InvalidChain(forChainId);
        }
        bitmap = _getPerChainRecvTransceiverBitmapStorage()[integrator][forChainId].bitmap;
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
    /// @param chainId The chainId for the desired transceivers
    /// @return result The enabled send side transceivers for the given integrator and chain.
    function getSendTransceiversByChain(address integrator, uint16 chainId)
        external
        view
        returns (address[] memory result)
    {
        address[] memory allTransceivers = _getRegisteredTransceiversStorage()[integrator];
        address[] memory tempResult = new address[](allTransceivers.length);
        for (uint256 i = 0; i < allTransceivers.length; i++) {
            if (_isSendTransceiverEnabledForChain(integrator, allTransceivers[i], chainId)) {
                tempResult[i] = allTransceivers[i];
            }
        }
        result = new address[](tempResult.length);
        for (uint256 i = 0; i < tempResult.length; i++) {
            result[i] = tempResult[i];
        }
    }

    /// @notice Returns the enabled send side transceiver addresses for the given integrator.
    /// @param integrator The integrator address
    /// @param chainId The chainId for the desired transceivers
    /// @return result The enabled send side transceivers for the given integrator.
    function getRecvTransceiversByChain(address integrator, uint16 chainId)
        external
        view
        returns (address[] memory result)
    {
        address[] memory allTransceivers = _getRegisteredTransceiversStorage()[integrator];
        address[] memory tempResult = new address[](allTransceivers.length);
        for (uint256 i = 0; i < allTransceivers.length; i++) {
            if (_isRecvTransceiverEnabledForChain(integrator, allTransceivers[i], chainId)) {
                tempResult[i] = allTransceivers[i];
            }
        }
        result = new address[](tempResult.length);
        for (uint256 i = 0; i < tempResult.length; i++) {
            result[i] = tempResult[i];
        }
    }

    // ============== Invariants =============================================

    /// @dev Check that the transceiver is in a valid state.
    /// Checking these invariants is somewhat costly, but we only need to do it
    /// when modifying the transceivers, which happens infrequently.
    function _checkSendTransceiversInvariants(address integrator) internal view {
        // _NumTransceivers storage _numTransceivers = _getNumSendTransceiversStorage()[integrator];
        // address[] storage _enabledTransceivers = _getRegisteredSendTransceiversStorage()[integrator];

        // uint256 numTransceiversEnabled = _numTransceivers.enabled;
        // assert(numTransceiversEnabled == _enabledTransceivers.length);

        // for (uint256 i = 0; i < numTransceiversEnabled; i++) {
        //     _checkSendTransceiverInvariants(integrator, _enabledTransceivers[i]);
        // }

        // // invariant: each transceiver is only enabled once
        // for (uint256 i = 0; i < numTransceiversEnabled; i++) {
        //     for (uint256 j = i + 1; j < numTransceiversEnabled; j++) {
        //         assert(_enabledTransceivers[i] != _enabledTransceivers[j]);
        //     }
        // }

        // // invariant: numRegisteredTransceivers <= MAX_TRANSCEIVERS
        // assert(_numTransceivers.registered <= MAX_TRANSCEIVERS);
    }

    /// @dev Check that the transceiver is in a valid state.
    function _checkSendTransceiverInvariants(address integrator, address transceiver) private view {
        // mapping(address => mapping(address => TransceiverInfo)) storage transceiverInfos = _getTransceiverInfosStorage();
        // mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _enabledTransceiverBitmap =
        //     _getPerChainSendTransceiverBitmapStorage();
        // mapping(address => _NumTransceivers) storage _numTransceivers = _getNumSendTransceiversStorage();
        // mapping(address => address[]) storage _enabledTransceivers = _getRegisteredSendTransceiversStorage();

        // TransceiverInfo memory transceiverInfo = transceiverInfos[integrator][transceiver];

        // // if an transceiver is not registered, it should not be enabled
        // assert(transceiverInfo.registered || (!transceiverInfo.enabled && transceiverInfo.index == 0));

        // bool transceiverInEnabledBitmap = (
        //     _enabledTransceiverBitmap[integrator][transceiverInfo.chainId].bitmap & uint128(1 << transceiverInfo.index)
        // ) != 0;
        // bool transceiverEnabled = transceiverInfo.enabled;

        // bool transceiverInEnabledTransceivers = false;

        // for (uint256 i = 0; i < _numTransceivers[integrator].enabled; i++) {
        //     if (_enabledTransceivers[integrator][i] == transceiver) {
        //         transceiverInEnabledTransceivers = true;
        //         break;
        //     }
        // }

        // // invariant: transceiverInfos[integrator][transceiver].enabled
        // //            <=> enabledTransceiverBitmap & (1 << transceiverInfos[integrator][transceiver].index) != 0
        // assert(transceiverInEnabledBitmap == transceiverEnabled);

        // // invariant: transceiverInfos[integrator][transceiver].enabled <=> transceiver in _enabledTransceivers
        // assert(transceiverInEnabledTransceivers == transceiverEnabled);

        // assert(transceiverInfo.index < _numTransceivers[integrator].registered);
    }

    /// @dev Check that the transceiver is in a valid state.
    /// Checking these invariants is somewhat costly, but we only need to do it
    /// when modifying the transceivers, which happens infrequently.
    function _checkRecvTransceiversInvariants(address integrator) internal view {
        // _NumTransceivers storage _numTransceivers = _getNumRecvTransceiversStorage()[integrator];
        // address[] storage _enabledTransceivers = _getRegisteredRecvTransceiversStorage()[integrator];

        // uint256 numTransceiversEnabled = _numTransceivers.enabled;
        // assert(numTransceiversEnabled == _enabledTransceivers.length);

        // for (uint256 i = 0; i < numTransceiversEnabled; i++) {
        //     _checkRecvTransceiverInvariants(integrator, _enabledTransceivers[i]);
        // }

        // // invariant: each transceiver is only enabled once
        // for (uint256 i = 0; i < numTransceiversEnabled; i++) {
        //     for (uint256 j = i + 1; j < numTransceiversEnabled; j++) {
        //         assert(_enabledTransceivers[i] != _enabledTransceivers[j]);
        //     }
        // }

        // // invariant: numRegisteredTransceivers <= MAX_TRANSCEIVERS
        // assert(_numTransceivers.registered <= MAX_TRANSCEIVERS);
    }

    /// @dev Check that the transceiver is in a valid state.
    function _checkRecvTransceiverInvariants(address integrator, address transceiver) private view {
        // mapping(address => mapping(address => TransceiverInfo)) storage transceiverInfos = _getTransceiverInfosStorage();
        // mapping(address => mapping(uint16 => _EnabledTransceiverBitmap)) storage _enabledTransceiverBitmap =
        //     _getPerChainRecvTransceiverBitmapStorage();
        // mapping(address => _NumTransceivers) storage _numTransceivers = _getNumRecvTransceiversStorage();
        // mapping(address => address[]) storage _enabledTransceivers = _getRegisteredRecvTransceiversStorage();

        // TransceiverInfo memory transceiverInfo = transceiverInfos[integrator][transceiver];

        // // if an transceiver is not registered, it should not be enabled
        // assert(transceiverInfo.registered || (!transceiverInfo.enabled && transceiverInfo.index == 0));

        // bool transceiverInEnabledBitmap = (
        //     _enabledTransceiverBitmap[integrator][transceiverInfo.chainId].bitmap & uint128(1 << transceiverInfo.index)
        // ) != 0;
        // bool transceiverEnabled = transceiverInfo.enabled;

        // bool transceiverInEnabledTransceivers = false;

        // for (uint256 i = 0; i < _numTransceivers[integrator].enabled; i++) {
        //     if (_enabledTransceivers[integrator][i] == transceiver) {
        //         transceiverInEnabledTransceivers = true;
        //         break;
        //     }
        // }

        // // invariant: transceiverInfos[integrator][transceiver].enabled
        // //            <=> enabledTransceiverBitmap & (1 << transceiverInfos[integrator][transceiver].index) != 0
        // assert(transceiverInEnabledBitmap == transceiverEnabled);

        // // invariant: transceiverInfos[integrator][transceiver].enabled <=> transceiver in _enabledTransceivers
        // assert(transceiverInEnabledTransceivers == transceiverEnabled);

        // assert(transceiverInfo.index < _numTransceivers[integrator].registered);
    }
}
