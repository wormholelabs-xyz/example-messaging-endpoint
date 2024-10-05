# GMP Router

## Project Structure

```mermaid
classDiagram
    class Config {
        bump: u8
        next_integrator_id: u64
    }

    class IntegratorChainTransceivers {
        bump: u8
        chain_id: u16
        owner: Pubkey
        next_in_transceiver_id: u8
        next_out_transceiver_id: u8
        in_transceiver_bitmap: Bitmap
        out_transceiver_bitmap: Bitmap
    }

    class RegisteredTransceiver {
        bump: u8
        id: u8
        chain_id: u16
        address: Pubkey
    }

    class Bitmap {
        map: u128
    }

    Config "1" -- "*" IntegratorChainTransceivers : tracks
    IntegratorChainTransceivers "1" -- "2" Bitmap : uses
    IntegratorChainTransceivers "1" -- "*" RegisteredTransceiver : manages
```

### Key Components

1. **Config**: Stores global configuration for the GMP Router.

    - Tracks the integrator ID counter.
    - Singleton account created during program initialization.

2. **IntegratorChainTransceivers**: Manages transceivers for a specific integrator on a particular chain.

    - Uses bitmaps for efficient storage and lookup of transceiver statuses.
    - Maintains separate counters for incoming and outgoing transceivers.
    - Stores the owner of the account.

3. **RegisteredTransceiver**: Represents a registered transceiver in the GMP Router.

    - Associated with a specific integrator and chain.
    - Has a unique ID within its integrator and chain context.

4. **Bitmap**: Utility struct for efficient storage and manipulation of boolean flags.
    - Used to track the status of transceivers (active/inactive).

### Relationships

- The Config account tracks multiple IntegratorChainTransceivers.
- Each IntegratorChainTransceivers account is associated with a specific integrator (identified by their public key) and chain.
- IntegratorChainTransceivers use two Bitmaps to efficiently track incoming and outgoing transceiver statuses.
- Each Bitmap tracks multiple RegisteredTransceivers.
- RegisteredTransceivers are associated with a specific integrator (via public key) and chain.

This structure allows for efficient management of multiple integrators, chains, and transceivers within the GMP Router system. It provides a scalable and flexible architecture for handling cross-chain message passing.

For detailed documentation on each component and its methods, please refer to the source files and generated API documentation.

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

3. **TransferIntegratorChainTransceiversOwnership**
   - [x] Test successful ownership transfer
   - [x] Test attempt to transfer ownership with non-owner account
   - [ ] Test attempt to transfer ownership to the same owner
   - [ ] Test attempt to transfer ownership to a zero address
   - [ ] Test registration of transceivers after ownership transfer
