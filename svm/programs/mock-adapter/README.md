# Mock Adapter Program

This program serves as a mock adapter to demonstrate how to interact with the endpoint program. It's designed to simulate the process of picking up messages, attesting messages, and executing messages, which require Cross-Program Invocation (CPI) calls with a Program Derived Address (PDA) signer.

## Overview

The mock adapter program is crucial for testing the endpoint program functionality. It serves as a gateway to test several features of the endpoint program because:

1. It simulates the message pickup process, which is essential for sending cross-chain messages.
2. It demonstrates how to attest to messages.
3. It allows us to test the endpoint program's handling of Cross-Program Invocations (CPIs) from an adapter.

This program contains three main instructions:

1. `invoke_pick_up_message`: This instruction demonstrates how to pick up a message from the endpoint's outbox.
2. `invoke_attest_message`: This instruction shows how to attest to a message in the endpoint program.

By calling these instructions, we can test critical aspects of the endpoint program, ensuring its complete functionality in a controlled testing environment.

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

4. Navigate to the mock-adapter program's directory:

   ```
   cd programs/mock-adapter
   ```

5. Run the tests using Cargo:
   ```
   cargo test-sbf
   ```

This process will first build all the programs in the workspace, including the endpoint program that the mock-adapter depends on, and then run the tests for the mock-adapter program in a Solana BPF environment.

## Code Structure

The main components of the program are:

- `invoke_pick_up_message`: The instruction that demonstrates the CPI call to the endpoint program's pick_up_message function.
- `invoke_attest_message`: The instruction that demonstrates the CPI call to the endpoint program's attest_message function.
- `InvokePickUpMessage`: The struct that defines the accounts required for the invoke_pick_up_message instruction.
- `InvokeAttest`: The struct that defines the accounts required for both invoke_attest_message and invoke_exec_message instructions.

Each instruction uses a PDA signer to authorize the CPI call to the endpoint program, demonstrating how a real adapter would interact with the endpoint.
