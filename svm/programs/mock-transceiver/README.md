# Mock Transceiver Program

This program serves as a mock transceiver to demonstrate how to interact with the router program. It's designed to simulate the process of picking up messages, attesting messages, and executing messages, which require Cross-Program Invocation (CPI) calls with a Program Derived Address (PDA) signer.

## Overview

The mock transceiver program is crucial for testing the router program functionality. It serves as a gateway to test several features of the router program because:

1. It simulates the message pickup process, which is essential for sending cross-chain messages.
2. It demonstrates how to attest to messages.
3. It allows us to test the router program's handling of Cross-Program Invocations (CPIs) from a transceiver.

This program contains three main instructions:

1. `invoke_pick_up_message`: This instruction demonstrates how to pick up a message from the router's outbox.
2. `invoke_attest_message`: This instruction shows how to attest to a message in the router program.

By calling these instructions, we can test critical aspects of the router program, ensuring its complete functionality in a controlled testing environment.

## Requirements

Before testing, ensure you have the following installed:

- [Rust 1.75.0](https://www.rust-lang.org/tools/install)
- [Solana 1.18.17](https://solana.com/docs/intro/installation)
- [Anchor 0.30.1](https://www.anchor-lang.com/docs/installation)
- [Yarn](https://yarnpkg.com/getting-started/install)

## How to Test

To test this program, follow these steps:

1. Ensure you have Rust, Anchor, and the Solana tool suite installed on your system.

2. Navigate to the root directory of the SVM project:

   ```
   cd svm
   ```

3. Build the Anchor project:

   ```
   anchor build
   ```

4. Navigate to the mock-transceiver program's directory:

   ```
   cd programs/mock-transceiver
   ```

5. Run the tests using Cargo:
   ```
   cargo test-sbf
   ```

This process will first build all the programs in the workspace, including the router program that the mock-transceiver depends on, and then run the tests for the mock-transceiver program in a Solana BPF environment.

## Code Structure

The main components of the program are:

- `invoke_pick_up_message`: The instruction that demonstrates the CPI call to the router program's pick_up_message function.
- `invoke_attest_message`: The instruction that demonstrates the CPI call to the router program's attest_message function.
- `InvokePickUpMessage`: The struct that defines the accounts required for the invoke_pick_up_message instruction.
- `InvokeAttest`: The struct that defines the accounts required for both invoke_attest_message and invoke_exec_message instructions.

Each instruction uses a PDA signer to authorize the CPI call to the router program, demonstrating how a real transceiver would interact with the router.
