# GMP Router

## Project Structure

```mermaid
classDiagram
    class IntegratorConfig {
        bump: u8
        authority: Pubkey
        program_id: Pubkey
        next_transceiver_id: u8
    }

    class IntegratorChainTransceivers {
        bump: u8
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

   Config "1" -- "" IntegratorConfig : manages
   IntegratorConfig "1" -- "" IntegratorChainTransceivers : manages
   IntegratorChainTransceivers "1" -- "2" Bitmap : uses
   IntegratorConfig "1" -- "" RegisteredTransceiver : manag
```

### Key Components

1. **IntegratorConfig**: Stores configuration specific to an Integrator.

   - **bump**: Bump seed for PDA derivation.
   - **authority**: The authority of the Integrator config.
   - **program_id**: The program ID of the Integrator.
   - **next_transceiver_id**: Counter to track the next transceiver ID.

2. **IntegratorChainTransceivers**: Manages transceivers for a specific integrator on a particular chain.

   - **bump**: Bump seed for PDA derivation.
   - **chain_id**: Identifier for the blockchain network.
   - **integrator_program_id**: The program ID of the Integrator.
   - **in_transceiver_bitmap**: Bitmap tracking enabled incoming transceivers.
   - **out_transceiver_bitmap**: Bitmap tracking enabled outgoing transceivers.

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

### Instructions

1. **init_integrator_config**: Initializes the integrator configuration.
2. **initialize_integrator_chain_transceivers**: Sets up the chain transceivers for an integrator on a specific chain.
3. **register_transceiver**: Registers a new transceiver for an integrator.
4. **set_in_transceivers**: Sets the incoming transceivers for a specific chain.
5. **set_out_transceivers**: Sets the outgoing transceivers for a specific chain.

### Error Handling

The program uses a custom `RouterError` enum to handle various error cases, including:
- Invalid integrator authority
- Bitmap index out of bounds
- Maximum number of transceivers reached
- Invalid transceiver ID

### Tests

1. **InitIntegratorConfig**
   - [x] Test successful initialization
   - [x] Test double initialization (should fail)
   - [x] Test initialization for different programs
   - [ ] Test initialization with non-program-owner authority (not implemented yet)

2. **InitializeIntegratorChainTransceivers**
   - [x] Test successful initialization
   - [x] Test initialization for already initialized chain (should fail)
   - [x] Test initialization for different chains
   - [x] Test initialization with invalid authority

3. **RegisterTransceiver**
   - [x] Test successful registration
   - [x] Test registration causing bitmap overflow
   - [x] Test registration with non-authority signer
   - [ ] Test registration of duplicate transceiver (not implemented yet)
   - [ ] Test registration with invalid transceiver address
   > **Note on Reinitialization:**
   > There is no need to test for reinitialization of the `IntegratorConfig` because the `next_transceiver_id` in `integrator_config` is auto-incremented. This ensures that each transceiver is uniquely identified and prevents accidental overwriting or duplication during initialization.

4. **SetTransceivers**
   - [x] Test successful setting of incoming transceivers
   - [x] Test successful setting of outgoing transceivers
   - [x] Test setting transceivers with invalid authority
   - [x] Test setting transceivers with invalid bitmap
   - [x] Test multiple updates of transceiver settings
