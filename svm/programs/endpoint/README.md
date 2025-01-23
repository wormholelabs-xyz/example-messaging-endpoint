# Modular Messaging Endpoint

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
        registered_adapters: Vec<Pubkey>
    }

    class IntegratorChainConfig {
        *bump: u8
        *integrator_program_id: Pubkey
        *chain_id: u16
        recv_adapter_bitmap: Bitmap
        send_adapter_bitmap: Bitmap
    }

    class AdapterInfo {
        *bump: u8
        *integrator_program_id: Pubkey
        *adapter_program_id: Pubkey
        id: u8
    }

    class Bitmap {
        map: u128
    }

    class AttestationInfo {
        *bump: u8
        *message_hash: [u8; 32]
        src_chain: u16
        src_addr: [u8; 32]
        sequence: u64
        dst_chain: u16
        dst_addr: [u8; 32]
        payload_hash: [u8; 32]
        executed: bool
        attested_adapters: Bitmap
    }

    class OutboxMessage {
        src_addr: [u8; 32]
        sequence: u64
        dst_chain: u16
        dst_addr: [u8; 32]
        payload_hash: [u8; 32]
        outstanding_adapters: Bitmap
    }

    class OutboxMessageKey {
        *bump: u8
        *integrator_program_id: Pubkey
        sequence: u64
    }

    IntegratorConfig "1" -- "" IntegratorChainConfig : manages
    IntegratorChainConfig "1" -- "2" Bitmap : uses
    IntegratorConfig "1" -- "*" AdapterInfo : tracks
    IntegratorChainConfig "1" -- "*" AdapterInfo : corresponds to
    AttestationInfo "1" -- "1" Bitmap : uses
    OutboxMessage "1" -- "1" Bitmap : uses
    IntegratorConfig "1" -- "1" OutboxMessageKey : has
```

> **Note:** fields marked with an asterisk (\*) in the class diagrams are used as seeds for Program Derived Address (PDA) derivation.

### Program Structure

```mermaid
graph LR
    Endpoint[Endpoint Program]
    I1[Integrator 1]
    I2[Integrator 2]
    I3[Integrator 3]
    T1[Adapters Vec]
    T2[Adapters Vec]
    T3[Adapters Vec]
    C1[Chain 1]
    C2[Chain 2]
    C3[Chain 3]
    SB1[Send Bitmap]
    RB1[Receive Bitmap]
    SB2[Send Bitmap]
    RB2[Receive Bitmap]
    SB3[Send Bitmap]
    RB3[Receive Bitmap]

    Endpoint --> I1
    Endpoint --> I2
    Endpoint --> I3
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

    subgraph "Registered Adapters"
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

This diagram illustrates the overall structure of the Endpoint program:

- The program manages multiple integrators.
- Each integrator has a vector of up to 128 adapters.
- For each integrator, there are multiple chains.
- Each chain has a send bitmap and a receive bitmap.
- The bitmaps correspond to the adapter vector, indicating which adapters are enabled for sending or receiving on that specific chain.

## Key Components

### IntegratorConfig

Manages the configuration for a specific integrator.

- **bump**: Bump seed for PDA derivation
- **integrator_program_id**: The program ID associated with this integrator
- **admin**: The current admin of the IntegratorConfig account (None if admin is discarded)
- **pending_admin**: The pending admin of the IntegratorConfig account (if a transfer is in progress)
- **registered_adapters**: Vector of registered adapter addresses

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id]`
- Unique for each integrator program
- Initialization: Requires integrator_program's PDA seeded by "endpoint_integrator"

**Constraints**:

- Maximum of 128 adapters per integrator

### IntegratorChainConfig

Manages adapters enabled and config for a specific integrator on a particular chain.

- **bump**: Bump seed for PDA derivation
- **integrator_program_id**: The program ID of the Integrator
- **chain_id**: Identifier for the blockchain network
- **send_adapter_bitmap**: Bitmap tracking enabled send adapters
- **recv_adapter_bitmap**: Bitmap tracking enabled receive adapters

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id, chain_id]`
- Unique for each integrator program and chain combination
- Initialization: Requires admin's signature and existing IntegratorConfig account

### AdapterInfo

Represents a registered adapter in the Endpoint.

- **bump**: Bump seed for PDA derivation
- **integrator_program_id**: The program ID of the Integrator
- **adapter_program_id**: Public key of the adapter's address
- **index**: Unique index of the adapter that corresponds to its position in the registered_adapters in IntegratorConfig account

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id, adapter_program_id]`
- Unique for each adapter within an integrator context

### Bitmap

Utility struct for efficient storage and manipulation of boolean flags.

- **map**: Stores the bitmap as a `u128`

### AttestationInfo

Stores information about message attestations.

- **bump**: Bump seed for PDA derivation
- **message_hash**: Hash of the message (used as a seed for PDA derivation)
- **src_chain**: Source chain ID
- **src_addr**: Source address
- **sequence**: Sequence number
- **dst_chain**: Destination chain ID
- **dst_addr**: Destination address
- **payload_hash**: Hash of the payload
- **executed**: Flag indicating if the message has been executed
- **attested_adapters**: Bitmap of adapters that have attested to the message

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, message_hash]`

### OutboxMessage

Represents an outgoing message in the outbox.

- **src_addr**: The sending integrator's address
- **sequence**: The sequence number of the message
- **dst_chain**: The destination chain's Wormhole Chain ID
- **dst_addr**: The destination address
- **payload_hash**: The hash of the message payload
- **outstanding_adapters**: Bitmap of send-enabled adapters that have not picked up the message

### OutboxMessageKey

Tracks the sequence number for an integrator program.

- **bump**: Bump seed for PDA derivation
- **integrator_program_id**: The program ID of the integrator (used as a seed for PDA derivation)
- **sequence**: The current sequence number for this integrator

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id]`

## Instructions

1. `register`: Registers an integrator and initializes their configuration
2. `add_adapter`: Registers a new adapter for an integrator
3. `enable_recv_adapter`: Sets an adapter as a receive adapter for a specific chain
4. `enable_send_adapter`: Sets an adapter as a send adapter for a specific chain
5. `disable_recv_adapter`: Disables a receive adapter for a specific chain
6. `disable_send_adapter`: Disables a send adapter for a specific chain
7. `update_admin`: A one-step transfer of admin rights for the IntegratorConfig to a new admin
8. `transfer_admin`: Initiates the transfer of admin rights for the IntegratorConfig to a new admin
9. `claim_admin`: Completes the transfer of admin rights, allowing the new admin to claim authority
10. `discard_admin`: Sets IntegratorConfig as immutable to emulate discarding admin on EVM. Action is irreversible
11. `pick_up_message`: Allows an adapter to pick up a message from the outbox, updating the outstanding adapters bitmap
12. `recv_message`: Receives a message that has been attested to, marking it as executed and returning enabled receive adapters and attestations
13. `send_message`: Creates a new outbox message for the specified destination chain and address, initializing it with provided information
14. `attest_message`: Marks an attestation by an outstanding_adapter to be true for a message.
15. `exec_message`: Bypasses attestation check and marks message as executed for replay protection.

## Error Handling

The program uses a custom `EndpointError` enum to handle various error cases, including:

- `CallerNotAuthorized`: The caller is not authorized to perform the action
- `BitmapIndexOutOfBounds`: Bitmap index is out of bounds
- `MaxAdaptersReached`: Maximum number of adapters reached
- `AdapterAlreadyEnabled`: Adapter was already enabled
- `AdapterAlreadyDisabled`: Adapter was already disabled
- `AdminTransferInProgress`: An admin transfer is in progress
- `NoAdminTransferInProgress`: No admin transfer is currently in progress
- `InvalidChainId`: The provided chain ID is invalid
- `AdapterNotEnabled`: No adapters are enabled for the operation
- `DuplicateMessageAttestation`: An attempt was made to attest to a message more than once
- `MessageAlreadyPickedUp`: The message has already been picked up
- `AlreadyExecuted`: The message has already been executed
- `UnknownMessageAttestation`: The message attestation is unknown or invalid

## Testing

| Instruction                                                                                              | Requirements                                                                                                                                                                                                                                                                                                                                                                                                   | Implemented Tests                                                                                                                                                                                                                                                                                                                                                                                                                             |
| -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| register(initialAdmin)                                                                                   | - Check caller not already registered<br>- Initialize registration and set initial admin                                                                                                                                                                                                                                                                                                                       | [x] Successful initialization of IntegratorConfig<br>[x] Reinitialization (fails with AccountAlreadyInUse error)                                                                                                                                                                                                                                                                                                                              |
| updateAdmin(integratorAddr, newAdmin)                                                                    | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Immediately set new admin                                                                                                                                                                                                                                                                     | [x] Successful admin update<br>[x] Update with non-authority signer (fails with CallerNotAuthorized)<br>[x] Update to the same admin address (succeeds)<br>[x] Update when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Update when IntegratorConfig is immutable (fails with CallerNotAuthorized)                                                                                                                  |
| transferAdmin(integratorAddr, newAdmin)                                                                  | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Set pending admin to the new admin                                                                                                                                                                                                                                                            | [x] Successful initiation of admin transfer<br>[x] Transfer when transfer already in progress (fails with AdminTransferInProgress)<br>[x] Transfer by non-authority signer (fails with CallerNotAuthorized)<br>[x] Transfer when IntegratorConfig is immutable (fails with CallerNotAuthorized)                                                                                                                                               |
| claimAdmin(integratorAddr)                                                                               | - Check caller is current or pending admin<br>- Check admin transfer is pending<br>- Complete/cancel transfer                                                                                                                                                                                                                                                                                                  | [x] Successful claiming of admin rights by new admin<br>[x] Successful claiming of admin rights by current admin (cancels transfer)<br>[x] Claim when there is no pending admin (fails with CallerNotAuthorized)<br>[x] Claim by unauthorized user (fails with CallerNotAuthorized)                                                                                                                                                           |
| discardAdmin(integratorAddr)                                                                             | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Clear current admin (make config immutable)                                                                                                                                                                                                                                                   | [x] Successful discarding of admin<br>[x] Discard when already discarded (fails with CallerNotAuthorized)<br>[x] Discard when transfer in progress (fails with AdminTransferInProgress)                                                                                                                                                                                                                                                       |
| addAdapter(integratorAddr, adapterAddr)                                                                  | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check adapter not already in array<br>- Check array won't surpass 128 entries<br>- Append adapter to array                                                                                                                                                                                    | [x] Successful addition of an adapter<br>[x] Addition of multiple adapters<br>[x] Addition with non-authority signer (fails with CallerNotAuthorized)<br>[x] Addition when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Addition when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Register max adapters (fails when exceeding)<br>[x] Reinitialization of existing adapter (fails)        |
| enableSendAdapter(integratorAddr, chain, adapterAddr)                                                    | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check adapter in array<br>- Check adapter currently disabled for sending<br>- Enable adapter for sending                                                                                                                                                                                      | [x] Successful enabling of send adapter<br>[x] Enabling with invalid admin (fails with CallerNotAuthorized)<br>[x] Enabling with invalid adapter ID (fails with AccountNotInitialized)<br>[x] Enabling when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Enabling when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Enabling already enabled adapter (fails with AdapterAlreadyEnabled)    |
| disableSendAdapter(integratorAddr, chain, adapterAddr)                                                   | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check adapter in array<br>- Check adapter currently enabled for sending<br>- Disable adapter for sending                                                                                                                                                                                      | [x] Successful disabling of send adapter<br>[x] Disabling with invalid admin (fails with CallerNotAuthorized)<br>[x] Disabling when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Disabling when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Disabling already disabled adapter (fails with AdapterAlreadyDisabled)                                                                        |
| enableRecvAdapter(integratorAddr, chain, adapterAddr)                                                    | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check adapter in array<br>- Check adapter currently disabled for receiving<br>- Enable adapter for receiving                                                                                                                                                                                  | [x] Successful enabling of receive adapter<br>[x] Enabling with invalid admin (fails with CallerNotAuthorized)<br>[x] Enabling with invalid adapter ID (fails with AccountNotInitialized)<br>[x] Enabling when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Enabling when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Enabling already enabled adapter (fails with AdapterAlreadyEnabled) |
| disableRecvAdapter(integratorAddr, chain, adapterAddr)                                                   | - Check caller is current admin<br>- Check no pending transfer<br>- Check IntegratorConfig is not immutable<br>- Check adapter in array<br>- Check adapter currently enabled for receiving<br>- Disable adapter for receiving                                                                                                                                                                                  | [x] Successful disabling of receive adapter<br>[x] Disabling with invalid admin (fails with CallerNotAuthorized)<br>[x] Disabling when admin transfer in progress (fails with AdminTransferInProgress)<br>[x] Disabling when IntegratorConfig is immutable (fails with CallerNotAuthorized)<br>[x] Disabling already disabled adapter (fails with AdapterAlreadyDisabled)                                                                     |
| pickUpMessage(outboxMsg)                                                                                 | - Check Adapter is an enabled send Adapter for the Integrator (srcAddr) and destination chain (dstChain)<br>- Check Adapter has NOT already picked up the message<br>- Mark Adapter as having picked up the message<br>- Closes outbox message account upon last enabled sending Adapter's pickup                                                                                                              | [x] Successful message pickup<br>[ ] Pickup with disabled Adapter (fails with MessageAlready)<br>[x] Pickup of already picked up message (fails)<br>[x] Pickup by last enabled Adapter (closes account)<br>[x] Pickup attempt after account closed(fails)                                                                                                                                                                                     |
| sendMessage(dstChain, dstAddr, payloadHash) → sequence                                                   | - MUST have at least one enabled **send** Adapter for `dstChain`<br>- Increments the Integrator's sequence and performs the steps to send the message or prepare it for sending, as applicable<br>- If Adapters must pull outgoing messages in the given implementation (via `pickUpMessage`), the Endpoint MUST set the current enabled Send Adapters as the Outstanding Adapters for that message            | [x] Successful message sending<br>[x] Correct outbox message creation (src_addr, sequence, dst_chain, dst_addr, payload_hash, outstanding_adapters)<br>[x] Sending with no enabled adapters (fails with AccountNotInitialized)<br>[x] Sending to unregistered chain (fails with AccountNotInitialized)<br>[x] Correct updating of OutboxMessageKey sequence                                                                                   |
| attestMessage(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash)                               | - MUST check that the Adapter is an enabled **receive** Adapter for the Integrator (`dstAddr`) and **source** chain (`srcChain`).<br>- MUST check that attestation info dst_chain is valid <br>- MUST check that the Adapter has NOT already attested.<br>- MUST allow an Adapter to attest after message execution.<br>- Calculates the message hash and marks the Adapter as having attested to the message. | [x] Successful message attestation<br>[x] Correct attestation info creation (src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash)<br>[ ] Attestation with disabled adapter (fails with AdapterNotEnabled)<br>[x] Destination chain invalid (fails with InvalidDestinationChain) <br>[x] Duplicate attestation (fails with DuplicateMessageAttestation)<br>[x] Attestation after message execution                                |
| execMessage(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash)                                 | - MUST revert if already executed.<br>- MUST NOT require any Adapters to have attested.<br>- Marks the message as executed.                                                                                                                                                                                                                                                                                    | [x] Successful message execution<br>[x] Correct attestation info creation (src_chain, src_addr, sequence, dst_chain, dst_addr, payload_hash)<br>[x] Execution of already executed message (fails with AlreadyExecuted)<br>[x] Execution before any attestations                                                                                                                                                                               |
| recvMessage(srcChain, srcAddr, sequence, dstChain, dstAddr, payloadHash) → enabledBitmap, attestedBitmap | - MUST check that at least one Adapter has attested.<br>- MUST revert if already executed.<br>- Marks the message as executed and returns the enabled receive Adapters for that chain along with the attestations.<br>- NOTE: for efficiency, this combines `getMessageStatus` and `execMessage` into one call and is expected to be the primary way that Integrators receive messages.                        | [x] Successful message receive<br>[x] Receiving already executed message (fails with AlreadyExecuted)<br>[x] Receiving message without prior attestation (fails with AccountNotInitialized - error code 3012)<br>[ ] Correct return of enabled and attested bitmaps                                                                                                                                                                           |

## Events emitted

> Note that these events are emitted through a CPI call

| Event Name                  | Description                                                     | Fields                                                                                                                                                                                                                      |
| --------------------------- | --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| IntegratorRegistered        | Emitted when a new integrator is registered                     | - integrator: Pubkey<br>- admin: Pubkey                                                                                                                                                                                     |
| AdminUpdated                | Emitted when an integrator's admin is updated                   | - integrator: Pubkey<br>- old_admin: Pubkey<br>- new_admin: Pubkey                                                                                                                                                          |
| AdminUpdateRequested        | Emitted when an admin update is requested                       | - integrator: Pubkey<br>- old_admin: Pubkey<br>- new_admin: Pubkey                                                                                                                                                          |
| MessageSent                 | Emitted when a message is sent                                  | - sender: [u8; 32]<br>- sequence: u64<br>- recipient: [u8; 32]<br>- recipient_chain: u16<br>- payload_digest: [u8; 32]                                                                                                      |
| MessagePickedUp             | Emitted when a message is picked up by an adapter               | - src_addr: [u8; 32]<br>- sequence: u64<br>- dst_chain: u16<br>- dst_addr: [u8; 32]<br>- payload_hash: [u8; 32]<br>- adapter: Pubkey<br>- remaining_adapters: u128                                                          |
| MessageAttestedTo           | Emitted when a message is attested to by an adapter             | - message_hash: [u8; 32]<br>- src_chain: u16<br>- src_addr: [u8; 32]<br>- sequence: u64<br>- dst_chain: u16<br>- dst_addr: [u8; 32]<br>- payload_hash: [u8; 32]<br>- attested_bitmap: u128<br>- attesting_adapter: [u8; 32] |
| MessageReceived             | Emitted when a message is received                              | - message_hash: [u8; 32]<br>- src_chain: u16<br>- src_addr: [u8; 32]<br>- sequence: u64<br>- dst_chain: u16<br>- dst_addr: [u8; 32]<br>- payload_hash: [u8; 32]<br>- enabled_bitmap: u128<br>- attested_bitmap: u128        |
| MessageExecuted             | Emitted when a message is executed                              | - message_hash: [u8; 32]<br>- src_chain: u16<br>- src_addr: [u8; 32]<br>- sequence: u64<br>- dst_chain: u16<br>- dst_addr: [u8; 32]<br>- payload_hash: [u8; 32]                                                             |
| AdapterAdded                | Emitted when a new adapter is added to an integrator            | - integrator: Pubkey<br>- adapter: Pubkey<br>- adapters_num: u8                                                                                                                                                             |
| SendAdapterEnabledForChain  | Emitted when a send adapter is enabled for a specific chain     | - integrator: Pubkey<br>- chain: u16<br>- adapter: Pubkey                                                                                                                                                                   |
| RecvAdapterEnabledForChain  | Emitted when a receive adapter is enabled for a specific chain  | - integrator: Pubkey<br>- chain: u16<br>- adapter: Pubkey                                                                                                                                                                   |
| SendAdapterDisabledForChain | Emitted when a send adapter is disabled for a specific chain    | - integrator: Pubkey<br>- chain: u16<br>- adapter: Pubkey                                                                                                                                                                   |
| RecvAdapterDisabledForChain | Emitted when a receive adapter is disabled for a specific chain | - integrator: Pubkey<br>- chain: u16<br>- adapter: Pubkey                                                                                                                                                                   |
| AdminDiscarded              | Emitted when an admin is discarded for an integrator            | - integrator: Pubkey                                                                                                                                                                                                        |
