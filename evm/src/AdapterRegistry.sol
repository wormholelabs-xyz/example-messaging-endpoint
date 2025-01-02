// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "./interfaces/IAdapterRegistry.sol";

/// @title AdapterRegistry
/// @notice This contract is responsible for handling the registration of Adapters.
abstract contract AdapterRegistry is IAdapterRegistry {
    uint8 constant MAX_ADAPTERS = 128;

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

    /// @dev Holds mapping of integrator address => array of chains with adapters enabled for sending.
    bytes32 private constant SEND_ENABLED_CHAINS_SLOT = bytes32(uint256(keccak256("registry.sendEnabledChains")) - 1);

    /// @dev Holds mapping of integrator address => array of chains with adapters enabled for receiving.
    bytes32 private constant RECV_ENABLED_CHAINS_SLOT = bytes32(uint256(keccak256("registry.recvEnabledChains")) - 1);

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
        returns (mapping(address => mapping(uint16 => PerSendAdapterInfo[])) storage $)
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

    /// @dev Integrator address => chainID[] mapping.
    ///      Contains all chains that have adapters enabled for this integrator.
    function _getChainsEnabledStorage(bytes32 tag) internal pure returns (mapping(address => uint16[]) storage $) {
        uint256 slot = uint256(tag);
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
        uint8 index = _getAdapterInfosStorage()[integrator][adapter].index;
        PerSendAdapterInfo[] storage sendAdapterArray = _getPerChainSendAdapterArrayStorage()[integrator][chain];
        if (sendAdapterArray.length == 0) {
            _addEnabledChain(SEND_ENABLED_CHAINS_SLOT, integrator, chain);
        }
        sendAdapterArray.push(PerSendAdapterInfo({addr: adapter, index: index}));
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
        _EnabledAdapterBitmap storage _bitmapEntry = _getPerChainRecvAdapterBitmapStorage()[integrator][chain];
        if (_bitmapEntry.bitmap == 0) {
            _addEnabledChain(RECV_ENABLED_CHAINS_SLOT, integrator, chain);
        }
        _bitmapEntry.bitmap |= uint128(1 << index);
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
        mapping(address => mapping(uint16 => PerSendAdapterInfo[])) storage enabledSendAdapters =
            _getPerChainSendAdapterArrayStorage();
        PerSendAdapterInfo[] storage adapters = enabledSendAdapters[integrator][chain];

        // Get the index of the disabled adapter in the enabled adapters array
        // and replace it with the last element in the array.
        uint256 len = adapters.length;
        bool found = false;
        for (uint256 i = 0; i < len;) {
            if (adapters[i].addr == adapter) {
                // Swap the last element with the element to be removed
                adapters[i] = adapters[len - 1];
                // Remove the last element
                adapters.pop();
                found = true;
                if (adapters.length == 0) {
                    _removeEnabledChain(SEND_ENABLED_CHAINS_SLOT, integrator, chain);
                }
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
        _EnabledAdapterBitmap storage _bitmapEntry = _getPerChainRecvAdapterBitmapStorage()[integrator][chain];

        uint128 updatedEnabledAdapterBitmap =
            _bitmapEntry.bitmap & uint128(~(1 << adapterInfos[integrator][adapter].index));
        // ensure that this actually changed the bitmap
        if (updatedEnabledAdapterBitmap >= _bitmapEntry.bitmap) {
            revert AdapterAlreadyDisabled(adapter);
        }
        _bitmapEntry.bitmap = updatedEnabledAdapterBitmap;
        if (_bitmapEntry.bitmap == 0) {
            _removeEnabledChain(RECV_ENABLED_CHAINS_SLOT, integrator, chain);
        }

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
        PerSendAdapterInfo[] storage adapters = _getPerChainSendAdapterArrayStorage()[integrator][chain];
        uint256 length = adapters.length;
        for (uint256 i = 0; i < length;) {
            if (adapters[i].addr == adapter) {
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
        returns (PerSendAdapterInfo[] storage array)
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
    function getSendAdaptersByChain(address integrator, uint16 chain)
        public
        view
        returns (PerSendAdapterInfo[] memory result)
    {
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
        uint8 count = _getNumEnabledRecvAdaptersForChain(integrator, chain);
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

    /// @notice Returns all the chains for which the integrator has adapters enabled for sending.
    /// @dev The chain IDs are not guaranteed to be in order.
    /// @param integrator The integrator address.
    /// @return result The chains that have adapters enabled for sending for the given integrator.
    function getChainsEnabledForSend(address integrator) public view returns (uint16[] memory result) {
        return _getChainsEnabledStorage(SEND_ENABLED_CHAINS_SLOT)[integrator];
    }

    /// @notice Returns all the chains for which the integrator has adapters enabled for receiving.
    /// @dev The chain IDs are not guaranteed to be in order.
    /// @param integrator The integrator address.
    /// @return result The chains that have adapters enabled for receiving for the given integrator.
    function getChainsEnabledForRecv(address integrator) public view returns (uint16[] memory result) {
        return _getChainsEnabledStorage(RECV_ENABLED_CHAINS_SLOT)[integrator];
    }

    /// @notice Returns the number of enabled receive adapters for the given integrator and chain.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID for the desired adapters.
    /// @return result The number of enabled receive adapters for that chain.
    function _getNumEnabledRecvAdaptersForChain(address integrator, uint16 chain) public view returns (uint8 result) {
        uint128 bitmap = _getEnabledRecvAdaptersBitmapForChain(integrator, chain);
        while (bitmap != 0) {
            bitmap &= bitmap - 1;
            result++;
        }
    }

    // =============== Implementations =======================================

    /// @dev It's not an error if the chain is already in the list.
    function _addEnabledChain(bytes32 tag, address integrator, uint16 chain) internal {
        uint16[] storage chains = _getChainsEnabledStorage(tag)[integrator];
        uint256 len = chains.length;
        for (uint256 idx = 0; (idx < len);) {
            if (chains[idx] == chain) {
                return;
            }
            unchecked {
                ++idx;
            }
        }
        chains.push(chain);
    }

    /// @dev It's not an error if the chain is not in the list.
    function _removeEnabledChain(bytes32 tag, address integrator, uint16 chain) internal {
        uint16[] storage chains = _getChainsEnabledStorage(tag)[integrator];
        uint256 len = chains.length;
        for (uint256 idx = 0; (idx < len);) {
            if (chains[idx] == chain) {
                chains[idx] = chains[len - 1];
                chains.pop();
                return;
            }
            unchecked {
                ++idx;
            }
        }
    }
}
