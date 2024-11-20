// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

/// @title AdapterRegistry
/// @notice This contract is responsible for handling the registration of Adapters.
abstract contract AdapterRegistry {
    /// @dev Information about registered adapters.
    struct AdapterInfo {
        // whether this adapter is registered
        bool registered;
        uint8 index; // the index into the integrator's adapters array
    }

    /// @dev Bitmap encoding the enabled adapters.
    struct _EnabledAdapterBitmap {
        uint128 bitmap; // MAX_ADAPTERS = 128
    }

    uint8 constant MAX_ADAPTERS = 128;

    // =============== Events ===============================================

    /// @notice Emitted when an adapter is added.
    /// @dev Topic0
    ///      0xac0e1ca21680593c8e6fcd302536c12420131dbf0e8ee4b29e529e8a81469f21.
    /// @param integrator The address of the integrator.
    /// @param adapter The address of the adapter.
    /// @param adaptersNum The current number of adapters.
    event AdapterAdded(address integrator, address adapter, uint8 adaptersNum);

    /// @notice Emitted when a send side adapter is enabled for a chain.
    /// @dev Topic0
    ///      0x857691e2cf3b85361da0572fc891250016a03bf7921f5dbb9c34ca3d80591336.
    /// @param integrator The address of the integrator.
    /// @param chain The Wormhole chain ID on which this adapter is enabled.
    /// @param adapter The address of the adapter.
    event SendAdapterEnabledForChain(address integrator, uint16 chain, address adapter);

    /// @notice Emitted when a receive side adapter is enabled for a chain.
    /// @dev Topic0
    ///      0x3649cec96e246496c67087fabed01d3a6c510fec6bfcd3103dd2aedd1b637acc.
    /// @param integrator The address of the integrator.
    /// @param chain The Wormhole chain ID on which this adapter is enabled.
    /// @param adapter The address of the adapter.
    event RecvAdapterEnabledForChain(address integrator, uint16 chain, address adapter);

    /// @notice Emitted when a send side adapter is removed from the endpoint.
    /// @dev Topic0
    ///      0xf5794741500b506041917ff318a1635659dbe538238f4e60979e3f1d29ac021a.
    /// @param integrator The address of the integrator.
    /// @param chain The Wormhole chain ID on which this adapter is disabled.
    /// @param adapter The address of the adapter.
    event SendAdapterDisabledForChain(address integrator, uint16 chain, address adapter);

    /// @notice Emitted when a receive side adapter is removed from the endpoint.
    /// @dev Topic0
    ///      0xcf88609c95c0469dbeb39cfeaa5dcb9b389a6ec8acca2d695dd3feb2f69cffde.
    /// @param integrator The address of the integrator.
    /// @param chain The Wormhole chain ID on which this adapter is disabled.
    /// @param adapter The address of the adapter.
    event RecvAdapterDisabledForChain(address integrator, uint16 chain, address adapter);

    // =============== Errors ===============================================

    /// @notice Error when the caller is not the adapter.
    /// @dev Selector: 0xd8aa0b1c.
    /// @param caller The address of the caller.
    error CallerNotAdapter(address caller);

    /// @notice Error when the adapter is the zero address.
    /// @dev Selector: 0x4e2165a2.
    error InvalidAdapterZeroAddress();

    /// @notice Error when the adapter is disabled.
    /// @dev Selector: 0x3b4742ca.
    error AdapterAlreadyDisabled(address adapter);

    /// @notice Error when the number of registered adapters
    ///         exceeeds (MAX_ADAPTERS = 128).
    /// @dev Selector: 0x5bde12c0.
    error TooManyAdapters();

    /// @notice Error when attempting to use an unregistered adapter
    ///         that is not registered.
    /// @dev Selector: 0xc325a1ea.
    /// @param adapter The address of the adapter.
    error NonRegisteredAdapter(address adapter);

    /// @notice Error when attempting to use an incorrect chain.
    /// @dev Selector: 0x587c94c3.
    /// @param chain The id of the incorrect chain.
    error InvalidChain(uint16 chain);

    /// @notice Error when attempting to register an adapter that is already register.
    /// @dev Selector: 0x2296d41e.
    /// @param adapter The address of the adapter.
    error AdapterAlreadyRegistered(address adapter);

    /// @notice Error when attempting to enable an adapter that is already enabled.
    /// @dev Selector: 0xb7b944b2.
    /// @param adapter The address of the adapter.
    error AdapterAlreadyEnabled(address adapter);

    // =============== Storage ===============================================

    /// @dev Holds the integrator address to adapter address to AdapterInfo mapping.
    ///      mapping(address => mapping(address => AdapterInfo)).
    bytes32 private constant ADAPTER_INFOS_SLOT = bytes32(uint256(keccak256("registry.adapterInfos")) - 1);

    /// @dev Holds send side Integrator address => Adapter addresses mapping.
    ///      mapping(address => address[]) across all chains.
    bytes32 private constant REGISTERED_ADAPTERS_SLOT = bytes32(uint256(keccak256("registry.registeredAdapters")) - 1);

    // =============== Send side =============================================

    /// @dev Holds integrator address => Chain ID => Enabled send side adapter address[] mapping.
    ///      mapping(address => mapping(uint16 => address[])).
    bytes32 private constant ENABLED_SEND_ADAPTER_ARRAY_SLOT =
        bytes32(uint256(keccak256("registry.sendAdapterArray")) - 1);

    // =============== Recv side =============================================

    /// @dev Holds integrator address => Chain ID => Enabled adapter receive side bitmap mapping.
    ///      mapping(address => mapping(uint16 => uint128)).
    bytes32 private constant ENABLED_RECV_ADAPTER_BITMAP_SLOT =
        bytes32(uint256(keccak256("registry.recvAdapterBitmap")) - 1);

    // =============== Mappings ===============================================

    /// @dev Integrator address => adapter address => AdapterInfo mapping.
    function _getAdapterInfosStorage()
        internal
        pure
        returns (mapping(address => mapping(address => AdapterInfo)) storage $)
    {
        uint256 slot = uint256(ADAPTER_INFOS_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    /// @dev Integrator address => Chain ID => Enabled adapter bitmap mapping.
    function _getPerChainSendAdapterArrayStorage()
        private
        pure
        returns (mapping(address => mapping(uint16 => address[])) storage $)
    {
        uint256 slot = uint256(ENABLED_SEND_ADAPTER_ARRAY_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    /// @dev Integrator address => Chain ID => Enabled adapter bitmap mapping.
    function _getPerChainRecvAdapterBitmapStorage()
        private
        pure
        returns (mapping(address => mapping(uint16 => _EnabledAdapterBitmap)) storage $)
    {
        uint256 slot = uint256(ENABLED_RECV_ADAPTER_BITMAP_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    /// @dev Integrator address => Adapter address[] mapping.
    ///      Contains all registered adapters for this integrator.
    function _getRegisteredAdaptersStorage() internal pure returns (mapping(address => address[]) storage $) {
        uint256 slot = uint256(REGISTERED_ADAPTERS_SLOT);
        assembly ("memory-safe") {
            $.slot := slot
        }
    }

    // =============== Modifiers ======================================================

    /// @notice This modifier will revert if the adapter is an invalid address, not registered, or the chain is invalid.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID.
    /// @param adapter The adapter address.
    modifier onlyRegisteredAdapter(address integrator, uint16 chain, address adapter) {
        if (adapter == address(0)) {
            revert InvalidAdapterZeroAddress();
        }

        if (chain == 0) {
            revert InvalidChain(chain);
        }

        if (!_getAdapterInfosStorage()[integrator][adapter].registered) {
            revert NonRegisteredAdapter(adapter);
        }
        _;
    }

    // =============== Storage Getters/Setters ========================================

    /// @dev Adds an adapter.
    /// @param integrator The integrator address.
    /// @param adapter The adapter address.
    /// @return index The index of this newly enabled adapter.
    function _addAdapter(address integrator, address adapter) internal returns (uint8 index) {
        if (adapter == address(0)) {
            revert InvalidAdapterZeroAddress();
        }
        mapping(address => mapping(address => AdapterInfo)) storage adapterInfos = _getAdapterInfosStorage();
        mapping(address => address[]) storage registeredAdapters = _getRegisteredAdaptersStorage();

        if (adapterInfos[integrator][adapter].registered) {
            revert AdapterAlreadyRegistered(adapter);
        }
        uint8 registeredAdaptersLength = uint8(registeredAdapters[integrator].length);
        if (registeredAdaptersLength >= MAX_ADAPTERS) {
            revert TooManyAdapters();
        }
        // Create the AdapterInfo
        adapterInfos[integrator][adapter] = AdapterInfo({registered: true, index: registeredAdaptersLength});
        // Add this adapter to the integrator => address[] mapping
        registeredAdapters[integrator].push(adapter);
        // Emit an event
        emit AdapterAdded(integrator, adapter, registeredAdaptersLength + 1);

        return adapterInfos[integrator][adapter].index;
    }

    /// @dev It is assumed that the integrator address is already validated (and not 0).
    ///      This just enables the send side adapter.  It does not register it.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID.
    /// @param adapter The adapter address.
    function _enableSendAdapter(address integrator, uint16 chain, address adapter)
        internal
        onlyRegisteredAdapter(integrator, chain, adapter)
    {
        if (_isSendAdapterEnabledForChain(integrator, chain, adapter)) {
            revert AdapterAlreadyEnabled(adapter);
        }
        mapping(address => mapping(uint16 => address[])) storage sendAdapterArray =
            _getPerChainSendAdapterArrayStorage();
        sendAdapterArray[integrator][chain].push(adapter);
        emit SendAdapterEnabledForChain(integrator, chain, adapter);
    }

    /// @dev It is assumed that the integrator address is already validated (and not 0).
    ///      This just enables the receive side adapter.  It does not register it.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID.
    /// @param adapter The adapter address.
    function _enableRecvAdapter(address integrator, uint16 chain, address adapter)
        internal
        onlyRegisteredAdapter(integrator, chain, adapter)
    {
        if (_isRecvAdapterEnabledForChain(integrator, chain, adapter)) {
            revert AdapterAlreadyEnabled(adapter);
        }
        uint8 index = _getAdapterInfosStorage()[integrator][adapter].index;
        mapping(address => mapping(uint16 => _EnabledAdapterBitmap)) storage _bitmaps =
            _getPerChainRecvAdapterBitmapStorage();
        _bitmaps[integrator][chain].bitmap |= uint128(1 << index);
        emit RecvAdapterEnabledForChain(integrator, chain, adapter);
    }

    /// @notice Disables a send side adapter by chain.
    /// @param integrator The integrator address.
    /// @param chain The chain ID.
    /// @param adapter The adapter address.
    function _disableSendAdapter(address integrator, uint16 chain, address adapter)
        internal
        onlyRegisteredAdapter(integrator, chain, adapter)
    {
        mapping(address => mapping(uint16 => address[])) storage enabledSendAdapters =
            _getPerChainSendAdapterArrayStorage();
        address[] storage adapters = enabledSendAdapters[integrator][chain];

        // Get the index of the disabled adapter in the enabled adapters array
        // and replace it with the last element in the array.
        uint256 len = adapters.length;
        bool found = false;
        for (uint256 i = 0; i < len;) {
            if (adapters[i] == adapter) {
                // Swap the last element with the element to be removed
                adapters[i] = adapters[len - 1];
                // Remove the last element
                adapters.pop();
                found = true;
                break;
            }
            unchecked {
                ++i;
            }
        }
        if (!found) {
            revert AdapterAlreadyDisabled(adapter);
        }

        emit SendAdapterDisabledForChain(integrator, chain, adapter);
    }

    /// @notice Disables a receive side adapter by chain.
    /// @dev Will revert under the following conditions:
    ///         - The adapter is the zero address.
    ///         - The adapter is not registered.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID.
    /// @param adapter The adapter address.
    function _disableRecvAdapter(address integrator, uint16 chain, address adapter)
        internal
        onlyRegisteredAdapter(integrator, chain, adapter)
    {
        mapping(address => mapping(address => AdapterInfo)) storage adapterInfos = _getAdapterInfosStorage();
        mapping(address => mapping(uint16 => _EnabledAdapterBitmap)) storage _enabledAdapterBitmap =
            _getPerChainRecvAdapterBitmapStorage();

        uint128 updatedEnabledAdapterBitmap =
            _enabledAdapterBitmap[integrator][chain].bitmap & uint128(~(1 << adapterInfos[integrator][adapter].index));
        // ensure that this actually changed the bitmap
        if (updatedEnabledAdapterBitmap >= _enabledAdapterBitmap[integrator][chain].bitmap) {
            revert AdapterAlreadyDisabled(adapter);
        }
        _enabledAdapterBitmap[integrator][chain].bitmap = updatedEnabledAdapterBitmap;

        emit RecvAdapterDisabledForChain(integrator, chain, adapter);
    }

    function _isSendAdapterEnabledForChainWithCheck(address integrator, uint16 chain, address adapter)
        internal
        view
        onlyRegisteredAdapter(integrator, chain, adapter)
        returns (bool)
    {
        return _isSendAdapterEnabledForChain(integrator, chain, adapter);
    }

    /// @notice Returns whether or not the send side adapter is enabled for the given integrator and chain.
    /// @dev This function is private and should only be called by a function that checks the validity of chain and adapter.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID.
    /// @param adapter The adapter address.
    /// @return true if the adapter is enabled, false otherwise.
    function _isSendAdapterEnabledForChain(address integrator, uint16 chain, address adapter)
        private
        view
        returns (bool)
    {
        address[] storage adapters = _getPerChainSendAdapterArrayStorage()[integrator][chain];
        uint256 length = adapters.length;
        for (uint256 i = 0; i < length;) {
            if (adapters[i] == adapter) {
                return true;
            }
            unchecked {
                ++i;
            }
        }
        return false;
    }

    function _isRecvAdapterEnabledForChainWithCheck(address integrator, uint16 chain, address adapter)
        internal
        view
        onlyRegisteredAdapter(integrator, chain, adapter)
        returns (bool)
    {
        return _isRecvAdapterEnabledForChain(integrator, chain, adapter);
    }

    /// @notice Returns whether or not the receive side adapter is enabled for the given integrator and chain.
    /// @dev This function is private and should only be called by a function that checks the validity of chain and adapter.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID.
    /// @param adapter The adapter address.
    /// @return true if the adapter is enabled, false otherwise.
    function _isRecvAdapterEnabledForChain(address integrator, uint16 chain, address adapter)
        private
        view
        returns (bool)
    {
        uint128 bitmap = _getEnabledRecvAdaptersBitmapForChain(integrator, chain);
        uint8 index = _getAdapterInfosStorage()[integrator][adapter].index;
        return (bitmap & uint128(1 << index)) > 0;
    }

    /// @notice Returns the array of send side adapters enabled for this integrator and chain.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID.
    /// @return array The array of the send side adapters enabled for this integrator and chain.
    function _getEnabledSendAdaptersArrayForChain(address integrator, uint16 chain)
        internal
        view
        virtual
        returns (address[] storage array)
    {
        if (chain == 0) {
            revert InvalidChain(chain);
        }
        array = _getPerChainSendAdapterArrayStorage()[integrator][chain];
    }

    /// @notice Returns the bitmap of the receive side adapters enabled for this integrator and chain.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID.
    /// @return bitmap The bitmap of the send side adapters enabled for this integrator and chain.
    function _getEnabledRecvAdaptersBitmapForChain(address integrator, uint16 chain)
        internal
        view
        virtual
        returns (uint128 bitmap)
    {
        if (chain == 0) {
            revert InvalidChain(chain);
        }
        bitmap = _getPerChainRecvAdapterBitmapStorage()[integrator][chain].bitmap;
    }

    // =============== EXTERNAL FUNCTIONS ========================================

    /// @notice Returns all the registered adapter addresses for the given integrator.
    /// @param integrator The integrator address.
    /// @return result The registered adapters for the given integrator.
    function getAdapters(address integrator) external view returns (address[] memory result) {
        result = _getRegisteredAdaptersStorage()[integrator];
    }

    /// @notice Returns the queried adapter addresses.
    /// @param integrator The integrator address.
    /// @param index The index into the integrator's adapters array.
    /// @return result The registered adapter address.
    function getAdapterByIndex(address integrator, uint8 index) external view returns (address result) {
        result = _getRegisteredAdaptersStorage()[integrator][index];
    }

    // =============== PUBLIC GETTERS ========================================

    /// @notice Returns the maximum number of adapters allowed.
    /// @return uint8 The maximum number of adapters allowed.
    function maxAdapters() public pure returns (uint8) {
        return MAX_ADAPTERS;
    }

    /// @notice Returns the queried adapter's index.
    /// @param integrator The integrator address.
    /// @param adapter The address of this adapter.
    /// @return result The registered adapter index.
    function getAdapterIndex(address integrator, address adapter) external view returns (uint8 result) {
        AdapterInfo storage info = _getAdapterInfosStorage()[integrator][adapter];
        if (!info.registered) {
            revert NonRegisteredAdapter(adapter);
        }
        return info.index;
    }

    /// @notice Returns the enabled send side adapter addresses for the given integrator.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID for the desired adapters.
    /// @return result The enabled send side adapters for the given integrator and chain.
    function getSendAdaptersByChain(address integrator, uint16 chain) public view returns (address[] memory result) {
        if (chain == 0) {
            revert InvalidChain(chain);
        }
        result = _getEnabledSendAdaptersArrayForChain(integrator, chain);
    }

    /// @notice Returns the enabled receive side adapter addresses for the given integrator.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID for the desired adapters.
    /// @return result The enabled receive side adapters for the given integrator.
    function getRecvAdaptersByChain(address integrator, uint16 chain) public view returns (address[] memory result) {
        address[] memory allAdapters = _getRegisteredAdaptersStorage()[integrator];
        // Count number of bits set in the bitmap so we can calculate the size of the result array.
        uint128 bitmap = _getEnabledRecvAdaptersBitmapForChain(integrator, chain);
        uint8 count = 0;
        while (bitmap != 0) {
            count += uint8(bitmap & 1);
            bitmap >>= 1;
        }
        result = new address[](count);
        uint256 len = 0;
        uint256 arrayLength = allAdapters.length;
        for (uint256 i = 0; i < arrayLength;) {
            if (_isRecvAdapterEnabledForChain(integrator, chain, allAdapters[i])) {
                result[len] = allAdapters[i];
                ++len;
            }
            unchecked {
                ++i;
            }
        }
    }

    /// @notice Returns the number of enabled receive adapters for the given integrator and chain.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID for the desired adapters.
    /// @return result The number of enabled receive adapters for that chain.
    function _getNumEnabledRecvAdaptersForChain(address integrator, uint16 chain) public view returns (uint8 result) {
        uint128 bitmap = _getEnabledRecvAdaptersBitmapForChain(integrator, chain);
        while (bitmap != 0) {
            result += uint8(bitmap & 1);
            bitmap >>= 1;
        }
    }
}
