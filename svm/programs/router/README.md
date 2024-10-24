# GMP Router

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [Key Components](#key-components)
4. [Instructions](#instructions)
5. [Error Handling](#error-handling)
6. [Testing](#testing)

## Architecture

```mermaid
classDiagram
    class IntegratorConfig {
        *bump: u8
        *integrator_program_id: Pubkey
        admin: Option<Pubkey>
        pending_admin: Option<Pubkey>
        registered_transceivers: Vec<Pubkey>
    }

    class IntegratorChainConfig {
        *bump: u8
        *integrator_program_id: Pubkey
        *chain_id: u16
        recv_transceiver_bitmap: Bitmap
        send_transceiver_bitmap: Bitmap
    }

    class TransceiverInfo {
        *bump: u8
        *integrator_program_id: Pubkey
        *transceiver_program_id: Pubkey
        id: u8
    }

    class Bitmap {
        map: u128
    }

   IntegratorConfig "1" -- "" IntegratorChainConfig : manages
   IntegratorChainConfig "1" -- "2" Bitmap : uses
   IntegratorConfig "1" -- "*" TransceiverInfo : tracks
   IntegratorChainConfig "1" -- "*" TransceiverInfo : corresponds to
```

> **Note:** fields marked with an asterisk (\*) in the class diagrams are used as seeds for Program Derived Address (PDA) derivation.

### Program Structure

```mermaid
graph LR
    GMP[GMP Router Program]
    I1[Integrator 1]
    I2[Integrator 2]
    I3[Integrator 3]
    T1[Transceivers Vec]
    T2[Transceivers Vec]
    T3[Transceivers Vec]
    C1[Chain 1]
    C2[Chain 2]
    C3[Chain 3]
    SB1[Send Bitmap]
    RB1[Receive Bitmap]
    SB2[Send Bitmap]
    RB2[Receive Bitmap]
    SB3[Send Bitmap]
    RB3[Receive Bitmap]

    GMP --> I1
    GMP --> I2
    GMP --> I3
    I1 --> T1
    I2 --> T2
    I3 --> T3
    I1 --> C1
    I1 --> C2
    I1 --> C3
    C1 --> SB1
    C1 --> RB1
    C2 --> SB2
    C2 --> RB2
    C3 --> SB3
    C3 --> RB3

    subgraph "Registered Transceivers"
    T1 --- TV1[T1]
    T1 --- TV2[T2]
    T1 --- TV3[T3]
    T1 --- TV4[...]
    T1 --- TV128[T128]
    end

    subgraph "Bitmap (128 bits)"
    SB1 --- B1[1]
    SB1 --- B2[0]
    SB1 --- B3[1]
    SB1 --- B4[...]
    SB1 --- B128[0]
    end

    SB1 -.-> T1
    RB1 -.-> T1
    SB2 -.-> T1
    RB2 -.-> T1
    SB3 -.-> T1
    RB3 -.-> T1
```

This diagram illustrates the overall structure of the GMP Router program:

- The program manages multiple integrators.
- Each integrator has a vector of up to 128 transceivers.
- For each integrator, there are multiple chains.
- Each chain has a send bitmap and a receive bitmap.
- The bitmaps correspond to the transceiver vector, indicating which transceivers are enabled for sending or receiving on that specific chain.

## Key Components

### IntegratorConfig

Manages the configuration for a specific integrator.

- **bump**: Bump seed for PDA derivation
- **integrator_program_id**: The program ID associated with this integrator
- **admin**: The current admin of the IntegratorConfig account (None if admin is discarded)
- **pending_admin**: The pending admin of the IntegratorConfig account (if a transfer is in progress)
- **registered_transceivers**: Vector of registered transceiver addresses

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id]`
- Unique for each integrator program
- Initialization: Requires integrator_program's PDA seeded by "router_integrator"

**Constraints**:

- Maximum of 128 transceivers per integrator

### IntegratorChainConfig

Manages transceivers enabled and config for a specific integrator on a particular chain.

- **bump**: Bump seed for PDA derivation
- **integrator_program_id**: The program ID of the Integrator
- **chain_id**: Identifier for the blockchain network
- **send_transceiver_bitmap**: Bitmap tracking enabled send transceivers
- **recv_transceiver_bitmap**: Bitmap tracking enabled receive transceivers

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id, chain_id]`
- Unique for each integrator program and chain combination
- Initialization: Requires admin's signature and existing IntegratorConfig account

### TransceiverInfo

Represents a registered transceiver in the GMP Router.

- **bump**: Bump seed for PDA derivation
- **index**: Unique index of the transceiver that corresponds to it's position in the registered_transceivers in IntegratorConfig account
- **integrator_program_id**: The program ID of the Integrator
- **address**: Public key of the transceiver's address

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id, transceiver_program_id]`
- Unique for each transceiver within an integrator context

### Bitmap

Utility struct for efficient storage and manipulation of boolean flags.

- **map**: Stores the bitmap as a `u128`

## Instructions

1. `register`: Registers an integrator and initializes their configuration
2. `add_transceiver`: Registers a new transceiver for an integrator
3. `enable_recv_transceiver`: Sets a transceiver as a receive transceiver for a specific chain
4. `enable_send_transceiver`: Sets a transceiver as a send transceiver for a specific chain
5. `disable_recv_transceiver`: Disables a receive transceiver for a specific chain
6. `disable_send_transceiver`: Disables a send transceiver for a specific chain
7. `update_admin`: A one-step transfer of admin rights for the IntegratorConfig to a new admin
8. `transfer_admin`: Initiates the transfer of admin rights for the IntegratorConfig to a new admin
9. `claim_admin`: Completes the transfer of admin rights, allowing the new admin to claim authority
10. `discard_admin`: Sets IntegratorConfig as immutable to emulate discarding admin on EVM. Action is irreversible

## Error Handling

The program uses a custom `RouterError` enum to handle various error cases, including:

- `InvalidIntegratorAuthority`: Invalid integrator authority
- `BitmapIndexOutOfBounds`: Bitmap index is out of bounds
- `MaxTransceiversReached`: Maximum number of transceivers reached
- `TransceiverAlreadyEnabled`: Transceiver was already enabled
- `TransceiverAlreadyDisabled`: Transceiver was already disabled
- `AdminTransferInProgress`: An Admin transfer is in progress

## Testing

| Instruction                                                    | Requirements                                                                                                                                                                                                                              | Implemented Tests                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| -------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| register(initialAdmin)                                         | - Check caller not already registered<br>- Initialize registration and set initial admin                                                                                                                                                  | [x] Successful initialization of IntegratorConfig<br>[x] Reinitialization (fails with AccountAlreadyInUse error)                                                                                                                                                                                                                                                                                                                                              |
| updateAdmin(integratorAddr, newAdmin)                          | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Immediately set new admin                                                                                                | [x] Successful admin update<br>[x] Update with non-authority signer (fails with CallerNotAuthorized)<br>[x] Update to the same admin address (succeeds)<br>[x] Update when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Update when IntegratorConfig is immutable (fails with CallerNotAuthorized)                                                                                                                                  |
| transferAdmin(integratorAddr, newAdmin)                        | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Set pending admin to the new admin                                                                                       | [x] Successful initiation of admin transfer<br>[x] Transfer when transfer already in progress (fails with AdminTransferInProgress)<br>[x] Transfer by non-authority signer (fails with CallerNotAuthorized)<br>[x] Transfer when IntegratorConfig is immutable (fails with CallerNotAuthorized)                                                                                                                                                               |
| claimAdmin(integratorAddr)                                     | - Check caller is current or pending admin<br>- Check admin transfer is pending<br>- Complete/cancel transfer                                                                                                                             | [x] Successful claiming of admin rights by new admin<br>[x] Successful claiming of admin rights by current admin (cancels transfer)<br>[x] Claim when there is no pending admin (fails with CallerNotAuthorized)<br>[x] Claim by unauthorized user (fails with CallerNotAuthorized)                                                                                                                                                                           |
| discardAdmin(integratorAddr)                                   | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Clear current admin (make config immutable)                                                                              | [x] Successful discarding of admin<br>[x] Discard when already discarded (fails with CallerNotAuthorized)<br>[x] Discard when transfer in progress (fails with AdminTransferInProgress)                                                                                                                                                                                                                                                                       |
| addTransceiver(integratorAddr, transceiverAddr)                | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check transceiver not already in array<br>- Check array won't surpass 128 entries<br>- Append transceiver to array       | [x] Successful addition of a transceiver<br>[x] Addition of multiple transceivers<br>[x] Addition with non-authority signer (fails with CallerNotAuthorized)<br>[x] Addition when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Addition when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Register max transceivers (fails when exceeding)<br>[x] Reinitialization of existing transceiver (fails)         |
| enableSendTransceiver(integratorAddr, chain, transceiverAddr)  | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check transceiver in array<br>- Check transceiver currently disabled for sending<br>- Enable transceiver for sending     | [x] Successful enabling of send transceiver<br>[x] Enabling with invalid admin (fails with CallerNotAuthorized)<br>[x] Enabling with invalid transceiver ID (fails with AccountNotInitialized)<br>[x] Enabling when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Enabling when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Enabling already enabled transceiver (fails with TransceiverAlreadyEnabled)    |
| disableSendTransceiver(integratorAddr, chain, transceiverAddr) | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check transceiver in array<br>- Check transceiver currently enabled for sending<br>- Disable transceiver for sending     | [x] Successful disabling of send transceiver<br>[x] Disabling with invalid admin (fails with CallerNotAuthorized)<br>[x] Disabling when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Disabling when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Disabling already disabled transceiver (fails with TransceiverAlreadyDisabled)                                                                            |
| enableRecvTransceiver(integratorAddr, chain, transceiverAddr)  | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check transceiver in array<br>- Check transceiver currently disabled for receiving<br>- Enable transceiver for receiving | [x] Successful enabling of receive transceiver<br>[x] Enabling with invalid admin (fails with CallerNotAuthorized)<br>[x] Enabling with invalid transceiver ID (fails with AccountNotInitialized)<br>[x] Enabling when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Enabling when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Enabling already enabled transceiver (fails with TransceiverAlreadyEnabled) |
| disableRecvTransceiver(integratorAddr, chain, transceiverAddr) | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check transceiver in array<br>- Check transceiver currently enabled for receiving<br>- Disable transceiver for receiving | [x] Successful disabling of receive transceiver<br>[x] Disabling with invalid admin (fails with CallerNotAuthorized)<br>[x] Disabling when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Disabling when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Disabling already disabled transceiver (fails with TransceiverAlreadyDisabled)                                                                         |
