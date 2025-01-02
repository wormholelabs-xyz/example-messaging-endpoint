// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

/// @title AdapterRegistry
/// @notice This contract is responsible for handling the registration of Adapters.
interface IAdapterRegistry {
    /// @dev Information about registered adapters.
    struct AdapterInfo {
        // whether this adapter is registered
        bool registered;
        uint8 index; // the index into the integrator's adapters array
    }

    /// @dev Data maintained for each send adapter enabled for an integrator and chain.
    struct PerSendAdapterInfo {
        address addr;
        uint8 index;
    }

    /// @dev Bitmap encoding the enabled adapters.
    struct _EnabledAdapterBitmap {
        uint128 bitmap; // MAX_ADAPTERS = 128
    }

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
    ///         exceeds (MAX_ADAPTERS = 128).
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

    /// @notice Returns the maximum number of adapters allowed.
    /// @return uint8 The maximum number of adapters allowed.
    function maxAdapters() external pure returns (uint8);

    /// @notice Returns the enabled send side adapter addresses for the given integrator.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID for the desired adapters.
    /// @return result The enabled send side adapters for the given integrator and chain.
    function getSendAdaptersByChain(address integrator, uint16 chain)
        external
        view
        returns (PerSendAdapterInfo[] memory result);

    /// @notice Returns the enabled receive side adapter addresses for the given integrator.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID for the desired adapters.
    /// @return result The enabled receive side adapters for the given integrator.
    function getRecvAdaptersByChain(address integrator, uint16 chain) external view returns (address[] memory result);

    /// @notice Returns all the chains for which the integrator has adapters enabled for sending.
    /// @param integrator The integrator address.
    /// @return result The chains that have adapters enabled for sending for the given integrator.
    function getChainsEnabledForSend(address integrator) external view returns (uint16[] memory result);

    /// @notice Returns all the chains for which the integrator has adapters enabled for receiving.
    /// @param integrator The integrator address.
    /// @return result The chains that have adapters enabled for receiving for the given integrator.
    function getChainsEnabledForRecv(address integrator) external view returns (uint16[] memory result);

    /// @notice Returns the number of enabled receive adapters for the given integrator and chain.
    /// @param integrator The integrator address.
    /// @param chain The Wormhole chain ID for the desired adapters.
    /// @return result The number of enabled receive adapters for that chain.
    function _getNumEnabledRecvAdaptersForChain(address integrator, uint16 chain)
        external
        view
        returns (uint8 result);
}
