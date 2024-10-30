# Mock Integrator Program

This program serves as a mock integrator to demonstrate how to interact with the router program. It's designed to simulate the process of registering an integrator and performing various operations, which require Cross-Program Invocation (CPI) calls with a Program Derived Address (PDA) signer.

## Overview

The mock integrator program is crucial for testing the entire router program functionality. It serves as a gateway to test all features of the router program because:

1. It simulates the registration process, which is a prerequisite for all other router program operations.
2. It demonstrates how to send messages, receive messages, and execute messages through the router program.
3. It allows us to test the router program's handling of Cross-Program Invocations (CPIs) from an integrator.

This program contains four main instructions:

1. `invoke_register`: Demonstrates how to register an integrator with the router program.
2. `invoke_send_message`: Demonstrates how to send a message through the router program.
3. `invoke_recv_message`: Demonstrates how to receive a message from the router program.
4. `invoke_exec_message`: Demonstrates how to execute a message through the router program.

By calling these instructions, we can test all aspects of the router program, ensuring its complete functionality in a controlled testing environment.

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

4. Navigate to the mock-integrator program's directory:

   ```
   cd programs/mock-integrator
   ```

5. Run the tests using Cargo:
   ```
   cargo test-sbf
   ```

This process will first build all the programs in the workspace, including the router program that the mock-integrator depends on, and then run the tests for the mock-integrator program in a Solana BPF environment.

## Code Structure

The main components of the program are:

- `invoke_register`: The instruction that demonstrates the CPI call to the router program's register function.
- `invoke_send_message`: The instruction that demonstrates how to send a message through the router program.
- `invoke_recv_message`: The instruction that demonstrates how to receive a message from the router program.
- `invoke_exec_message`: The instruction that demonstrates how to execute a message through the router program.
- `InvokeRegisterArgs`: The struct that defines the arguments for the invoke_register instruction.
- `InvokeRegister`: The struct that defines the accounts required for the invoke_register instruction.
- `InvokeSendMessage`: The struct that defines the accounts required for the invoke_send_message instruction.
- `InvokeRecvMessage`: The struct that defines the accounts required for the invoke_recv_message instruction.
- `InvokeExecMessage`: The struct that defines the accounts required for the invoke_exec_message instruction.
