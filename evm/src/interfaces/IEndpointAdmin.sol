// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

interface IEndpointAdmin {
    /// @notice Transfers admin privileges from the current admin to another contract.
    /// @dev The msg.sender must be the current admin contract.
    /// @param integrator The address of the integrator contract.
    /// @param newAdmin The address of the new admin.
    function updateAdmin(address integrator, address newAdmin) external;

    /// @notice Starts the two step process of transferring admin privileges from the current admin to another contract.
    /// @dev The msg.sender must be the current admin contract.
    /// @param integrator The address of the integrator contract.
    /// @param newAdmin The address of the new admin.
    function transferAdmin(address integrator, address newAdmin) external;

    /// @notice Completes the two step process of transferring admin privileges from the current admin to another contract.
    /// @dev The msg.sender must be the current admin contract.
    /// @param integrator The address of the integrator contract.
    function claimAdmin(address integrator) external;

    /// @notice Clears the current admin. THIS IS NOT REVERSIBLE.
    ///         This ensures that the Integrator configuration becomes immutable.
    /// @dev The msg.sender must be the current admin contract.
    /// @param integrator The address of the integrator contract.
    function discardAdmin(address integrator) external;

    /// @notice Adds the given adapter to the given chain for the integrator's list of adapters.
    ///         This does NOT enable the adapter for sending or receiving.
    /// @param integrator The address of the integrator contract.
    /// @param adapter The address of the Adapter contract.
    function addAdapter(address integrator, address adapter) external returns (uint8 index);

    /// @notice This enables the sending of messages from the given adapter on the given chain.
    /// @param integrator The address of the integrator contract.
    /// @param adapter The address of the Adapter contract.
    /// @param chain The chain ID of the Adapter contract.
    function enableSendAdapter(address integrator, uint16 chain, address adapter) external;

    /// @notice This enables the receiving of messages by the given adapter on the given chain.
    /// @param integrator The address of the integrator contract.
    /// @param adapter The address of the Adapter contract.
    /// @param chain The chain ID of the Adapter contract.
    function enableRecvAdapter(address integrator, uint16 chain, address adapter) external;

    /// @notice This disables the sending of messages from the given adapter on the given chain.
    /// @param integrator The address of the integrator contract.
    /// @param adapter The address of the Adapter contract.
    /// @param chain The chain ID of the Adapter contract.
    function disableSendAdapter(address integrator, uint16 chain, address adapter) external;

    /// @notice This disables the receiving of messages by the given adapter on the given chain.
    /// @param integrator The address of the integrator contract.
    /// @param adapter The address of the Adapter contract.
    /// @param chain The chain ID of the Adapter contract.
    function disableRecvAdapter(address integrator, uint16 chain, address adapter) external;

    /// @notice Returns the number enabled receive adapters for the given chain.
    /// @param integrator The address of the integrator contract.
    /// @param chain The chain ID of the Adapter contract.
    function getNumEnabledRecvAdaptersForChain(address integrator, uint16 chain) external view returns (uint8 count);
}
