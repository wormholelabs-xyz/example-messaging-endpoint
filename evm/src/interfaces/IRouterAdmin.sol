// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

interface IRouterAdmin {
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

    /// @notice Starts the two step process of transferring admin privileges from the current admin to another contract.
    /// @dev The msg.sender must be the current admin contract.
    /// @param integrator The address of the integrator contract.
    function claimAdmin(address integrator) external;

    /// @notice Clears the current admin. THIS IS NOT REVERSIBLE.
    ///         This ensures that the Integrator configuration becomes immutable.
    /// @dev The msg.sender must be the current admin contract.
    /// @param integrator The address of the integrator contract.
    function discardAdmin(address integrator) external;

    /// @notice Adds the given transceiver to the given chain for the integrator's list of transceivers.
    ///         This does NOT enable the transceiver for sending or receiving.
    /// @param integrator The address of the integrator contract.
    /// @param transceiver The address of the Transceiver contract.
    /// @param chainId The chain ID of the Transceiver contract.
    function addTransceiver(address integrator, uint16 chainId, address transceiver) external returns (uint8 index);

    /// @notice This enables the sending of messages from the given transceiver on the given chain.
    /// @param integrator The address of the integrator contract.
    /// @param transceiver The address of the Transceiver contract.
    /// @param chain The chain ID of the Transceiver contract.
    function enableSendTransceiver(address integrator, uint16 chain, address transceiver) external;

    /// @notice This enables the receiving of messages by the given transceiver on the given chain.
    /// @param integrator The address of the integrator contract.
    /// @param transceiver The address of the Transceiver contract.
    /// @param chain The chain ID of the Transceiver contract.
    function enableRecvTransceiver(address integrator, uint16 chain, address transceiver) external;

    /// @notice This disables the sending of messages from the given transceiver on the given chain.
    /// @param integrator The address of the integrator contract.
    /// @param transceiver The address of the Transceiver contract.
    /// @param chain The chain ID of the Transceiver contract.
    function disableSendTransceiver(address integrator, uint16 chain, address transceiver) external;

    /// @notice This disables the receiving of messages by the given transceiver on the given chain.
    /// @param integrator The address of the integrator contract.
    /// @param transceiver The address of the Transceiver contract.
    /// @param chain The chain ID of the Transceiver contract.
    function disableRecvTransceiver(address integrator, uint16 chain, address transceiver) external;
}
