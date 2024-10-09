# GMP Router

## Project Structure

```mermaid
classDiagram
    class IntegratorConfig {
        authority: Pubkey
        program_id: Pubkey
        next_transceiver_id: u8
    }

    class IntegratorChainTransceivers {
        chain_id: u16
        integrator_program_id: Pubkey
        in_transceiver_bitmap: Bitmap
        out_transceiver_bitmap: Bitmap
    }

    class RegisteredTransceiver {
        bump: u8
        id: u8
        integrator_program_id: u16
        address: Pubkey
    }

    class Bitmap {
        map: u128
    }

   IntegratorConfig "1" -- "" IntegratorChainTransceivers : manages
   IntegratorChainTransceivers "1" -- "2" Bitmap : uses
   IntegratorConfig "1" -- "" RegisteredTransceiver : manages
```

### Key Components

1. **IntegratorConfig**: Stores configuration specific to an Integrator.

   - **authority**: The authority of the Integrator config.
   - **program_id**: The program ID of the Integrator.
   - **next_transceiver_id**: Counter to track the next transceiver ID.

2. **IntegratorChainTransceivers**: Manages transceivers for a specific integrator on a particular chain.

   - **chain_id**: Identifier for the blockchain network.
   - **integrator_program_id**: The program ID of the Integrator.
   - **in_transceiver_bitmap**: Bitmap tracking enabled incoming transceivers by their IDs, corresponding to the transceiver IDs managed by `IntegratorConfig`.
   - **out_transceiver_bitmap**: Bitmap tracking enabled outgoing transceivers by their IDs, corresponding to the transceiver IDs managed by `IntegratorConfig`.

3. **RegisteredTransceiver**: Represents a registered transceiver in the GMP Router.

   - **bump**: Bump seed for PDA derivation.
   - **id**: Unique ID of the transceiver.
   - **integrator_program_id**: The program ID of the Integrator.
   - **address**: Address of the transceiver.

4. **Bitmap**: Utility struct for efficient storage and manipulation of boolean flags.

   - **map**: Stores the bitmap as a `u128`.

### PDA Derivation

1. **IntegratorConfig**

   - **Seeds**: `[SEED_PREFIX, integrator_program_id]`
   - **Unique** for each integrator program.

2. **IntegratorChainTransceivers**

   - **Seeds**: `[SEED_PREFIX, integrator_program_id, chain_id]`
   - **Unique** for each integrator program and chain combination.

3. **RegisteredTransceiver**

   - **Seeds**: `[SEED_PREFIX, integrator_program_id, transceiver_id]`
   - **Unique** for each transceiver within an integrator context.

### Relationships

- Each **IntegratorConfig** manages multiple **IntegratorChainTransceivers** (one for each chain).
- Each **IntegratorChainTransceivers** uses two **Bitmap** instances to track incoming and outgoing transceiver statuses.
- Each **IntegratorConfig** manages multiple **RegisteredTransceivers**.

### Instructions

1. **InitIntegratorConfig**: Initializes the integrator configuration.
2. **InitializeIntegratorChainTransceivers**: Sets up the chain transceivers for an integrator on a specific chain.
3. **RegisterTransceiver**: Registers a new transceiver for an integrator.

### Tests

1. **InitIntegratorChainTransceivers**

   - [x] Test init_integrator_chain_transceivers success
   - [x] Test init_integrator_chain_transceivers already initialized
   - [x] Test init_integrator_chain_transceivers for different chains
   - [ ] Test init_integrator_chain_transceivers with invalid chain ID
   - [ ] Test init_integrator_chain_transceivers with invalid owner

2. **RegisterTransceiver**

   - [x] Test register_transceiver success
   - [x] Test register_transceiver bitmap overflow
   - [x] Test register_transceiver with non-authority
   - [ ] Test registration of outgoing transceivers
   - [ ] Test attempt to register a duplicate transceiver
   - [ ] Test registration with invalid chain ID
   - [ ] Test registration with invalid transceiver address

3. **SetTransceivers**
   - [x] Test successful setting of incoming transceivers
   - [x] Test successful setting of outgoing transceivers
   - [ ] Test setting transceivers with an invalid authority (should fail)
   - [ ] Test setting transceivers for a non-existent chain ID (should fail)
   - [ ] Test setting transceivers with an invalid bitmap (e.g., with bits set beyond the maximum allowed transceiver ID)
   - [ ] Test setting transceivers multiple times and verify that the bitmap is correctly updated each time
