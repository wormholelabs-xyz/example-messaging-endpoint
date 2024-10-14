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
        bump: u8
        owner: Pubkey
        integrator_program_id: Pubkey
        transceivers: Vec<Pubkey>
    }

    class IntegratorChainConfig {
        bump: u8
        chain_id: u16
        integrator_program_id: Pubkey
        recv_transceiver_bitmap: Bitmap
        send_transceiver_bitmap: Bitmap
    }

    class TransceiverInfo {
        bump: u8
        id: u8
        integrator_program_id: Pubkey
        transceiver_address: Pubkey
    }

    class Bitmap {
        map: u128
    }

   IntegratorConfig "1" -- "" IntegratorChainConfig : manages
   IntegratorChainConfig "1" -- "2" Bitmap : uses
   IntegratorConfig "1" -- "*" TransceiverInfo : tracks
   IntegratorChainConfig "1" -- "*" TransceiverInfo : corresponds to
```

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

Stores configuration specific to an Integrator.

- **bump**: Bump seed for PDA derivation
- **owner**: The owner of the IntegratorConfig account
- **integrator_program_id**: The program ID of the Integrator
- **transceivers**: Vector of registered transceiver addresses (max 32)

**PDA Derivation**:

- Seeds: `[SEED_PREFIX, integrator_program_id]`
- Unique for each integrator program
- Initialization:
  - The integrator program must sign the transaction
  - Owner is set during initialization (not required to sign)

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
- Initialization: Requires owner's signature and existing IntegratorConfig account

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

1. `init_integrator_config`: Initialize integrator configuration
2. `register_transceiver`: Register a new transceiver for an integrator
3. `set_recv_transceiver`: Enable a receive transceiver for a specific chain. It initializes IntegratorChainConfig if it doesn't exist.
4. `disable_recv_transceiver`: Disable a receive transceiver for a specific chain
5. `set_send_transceiver`: Enable a send transceiver for a specific chain. It initializes IntegratorChainConfig if it doesn't exist.
6. `disable_send_transceiver`: Disable a send transceiver for a specific chain
7. `update_admin`: Transfer admin of the IntegratorConfig

## Error Handling

The program uses a custom `RouterError` enum to handle various error cases, including:

- Invalid integrator authority
- Bitmap index out of bounds
- Maximum number of transceivers reached

## Testing

### Register

- [x] Successful initialization of IntegratorConfig
- [x] Reinitialization (should fail with AccountAlreadyInUse error)
- [x] Initialization for different integrator programs

### RegisterTransceiver

- [x] Successful registration
- [x] Registration of multiple transceivers
- [x] Registration causing maximum transceivers reached error
- [x] Registration of duplicate transceiver (reinitialization)
- [x] Registration with non-authority signer
- [ ] Registration with invalid transceiver address (TBD: determine validation criteria)

### SetTransceivers

- [x] Successful setting of incoming transceivers
- [x] Successful setting of outgoing transceivers
- [ ] Disabling incoming/outgoing transceivers
- [x] Setting transceivers with invalid authority
- [x] Setting transceivers with invalid transceiver ID
- [x] Multiple updates of transceiver settings

### TransferIntegratorConfigOwnership

- [x] Successful ownership transfer
- [x] Transfer with invalid current owner
- [x] Transfer to the same owner
