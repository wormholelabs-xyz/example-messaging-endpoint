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
        admin: Pubkey
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
        *transceiver_address: Pubkey
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

### IntegratorChainConfig

Manages transceivers enabled and config for a specific integrator on a particular chain.

- **bump**: Bump seed for PDA derivation
- **chain_id**: Identifier for the blockchain network
- **integrator_program_id**: The program ID of the Integrator
- **recv_transceiver_bitmap**: Bitmap tracking enabled receive transceivers
- **send_transceiver_bitmap**: Bitmap tracking enabled send transceivers

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id, chain_id]`
- Unique for each integrator program and chain combination
- Initialization: Requires admin's signature and existing IntegratorConfig account

### IntegratorChainConfig

Manages transceivers enabled and config for a specific integrator on a particular chain.

- **bump**: Bump seed for PDA derivation
- **chain_id**: Identifier for the blockchain network
- **integrator_program_id**: The program ID of the Integrator
- **recv_transceiver_bitmap**: Bitmap tracking enabled receive transceivers
- **send_transceiver_bitmap**: Bitmap tracking enabled send transceivers

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id, chain_id]`
- Unique for each integrator program and chain combination
- Initialization: Requires admin's signature and existing IntegratorConfig account

### TransceiverInfo

Represents a registered transceiver in the GMP Router.

- **bump**: Bump seed for PDA derivation
- **id**: Unique ID of the transceiver within the integrator's context
- **integrator_program_id**: The program ID of the Integrator
- **address**: Public key of the transceiver's address

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id, transceiver_address]`
- Unique for each transceiver within an integrator context

**Constraints**:

- Maximum of 128 transceivers per integrator
- Will return an error (MaxTransceiversReached) if this limit is exceeded

### Bitmap

Utility struct for efficient storage and manipulation of boolean flags.

- **map**: Stores the bitmap as a `u128`

## Instructions

1. `register`: Registers an integrator and initializes their configuration
2. `register_transceiver`: Registers a new transceiver for an integrator
3. `set_recv_transceiver`: Sets a transceiver as a receive transceiver for a specific chain
4. `set_send_transceiver`: Sets a transceiver as a send transceiver for a specific chain
5. `disable_recv_transceiver`: Disables a receive transceiver for a specific chain
6. `disable_send_transceiver`: Disables a send transceiver for a specific chain
7. `update_admin`: Transfers admin of the IntegratorConfig to a new admin

## Error Handling

The program uses a custom `RouterError` enum to handle various error cases, including:

- `InvalidIntegratorAuthority`: Invalid integrator authority
- `BitmapIndexOutOfBounds`: Bitmap index is out of bounds
- `MaxTransceiversReached`: Maximum number of transceivers reached
- `TransceiverAlreadyEnabled`: Transceiver was already enabled
- `TransceiverAlreadyDisabled`: Transceiver was already disabled

## Testing

### Register

- [x] Successful initialization of IntegratorConfig
- [x] Reinitialization (fails with AccountAlreadyInUse error)

### RegisterTransceiver

- [x] Successful registration
- [x] Registration of multiple transceivers
- [x] Registration of more than 128 transceivers (fails with MaxTransceiversReached)
- [x] Registration of duplicate transceiver (fails with AccountAlreadyInUse error)
- [x] Registration with non-authority signer (fails with InvalidIntegratorAuthority error)

### SetTransceivers

- [x] Successful setting of incoming transceivers
- [x] Successful setting of outgoing transceivers
- [x] Setting transceivers with invalid authority (fails with InvalidIntegratorAuthority error)
- [x] Setting transceivers with invalid transceiver ID (fails with AccountNotInitialized error)
- [x] Multiple updates of transceiver settings
- [x] Attempt to enable already enabled transceiver (fails with TransceiverAlreadyEnabledError)

### DisableTransceivers

- [x] Successful disabling of incoming transceivers
- [x] Successful disabling of outgoing transceivers
- [x] Disabling transceivers with invalid authority (fails with InvalidIntegratorAuthority error)
- [x] Disabling transceivers with invalid transceiver ID (fails with AccountNotInitialized error)
- [x] Attempt to disable already disabled transceiver (fails with TransceiverAlreadyDisabled error)

### UpdateAdmin

> **Note:** The `update_admin` logic needs to be redone. Ignore this for now

- [x] Successful admin transfer
- [x] Transfer with invalid current admin
- [x] Transfer to the same admin
