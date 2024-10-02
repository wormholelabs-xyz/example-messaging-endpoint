# GMP Router

## Project Structure

```mermaid
classDiagram
    class Config {
        bump: u8
        owner: Pubkey
        paused: bool
        next_integrator_id: u64
    }

    class Integrator {
        bump: u8
        id: u64
        authority: Pubkey
    }

    class IntegratorChainTransceivers {
        bump: u8
        integrator_id: u64
        chain_id: u16
        next_in_transceiver_id: u8
        next_out_transceiver_id: u8
        in_transceiver_bitmap: Bitmap
        out_transceiver_bitmap: Bitmap
    }

    class RegisteredTransceiver {
        bump: u8
        integrator_id: u64
        id: u8
        chain_id: u16
        address: Pubkey
    }

    class Bitmap {
        bitmap: u128
    }

    Config "1" -- "*" Integrator : tracks
    Integrator "1" -- "*" IntegratorChainTransceivers : has
    IntegratorChainTransceivers "1" -- "2" Bitmap : uses
    Bitmap "1" -- "*" RegisteredTransceiver : tracks
```

### Key Components

1. **Config**: Stores global configuration for the GMP Router.

   - Tracks the program owner, pause state, and integrator ID counter.
   - Singleton account created during program initialization.

2. **Integrator**: Represents an entity that can interact with the GMP Router.

   - Each integrator has a unique ID and an associated authority.
   - Allows for multiple integrators to use the router independently.

3. **IntegratorChainTransceivers**: Manages transceivers for a specific integrator on a particular chain.

   - Uses bitmaps for efficient storage and lookup of transceiver statuses.
   - Maintains separate counters for incoming and outgoing transceivers.

4. **RegisteredTransceiver**: Represents a registered transceiver in the GMP Router.

   - Associated with a specific integrator and chain.
   - Has a unique ID within its integrator and chain context.

5. **Bitmap**: Utility struct for efficient storage and manipulation of boolean flags.
   - Used to track the status of transceivers (active/inactive).

### Relationships

- The Config account tracks multiple Integrators.
- Each Integrator can have multiple IntegratorChainTransceivers (one per chain).
- IntegratorChainTransceivers use two Bitmaps to efficiently track incoming and outgoing transceiver statuses.
- Each Bitmap tracks multiple RegisteredTransceivers.
- RegisteredTransceivers are associated with a specific Integrator and chain.

This structure allows for efficient management of multiple integrators, chains, and transceivers within the GMP Router system. It provides a scalable and flexible architecture for handling cross-chain message passing.

For detailed documentation on each component and its methods, please refer to the source files and generated API documentation.
