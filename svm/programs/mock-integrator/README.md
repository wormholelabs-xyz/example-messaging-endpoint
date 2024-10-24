# Mock Integrator Program

This program serves as a mock integrator to demonstrate how to call the register function in the router program. It's designed to simulate the process of registering an integrator, which requires a Cross-Program Invocation (CPI) call with a Program Derived Address (PDA) signer.

## Overview

The mock integrator program is crucial for testing the entire router program functionality. It serves as a gateway to test all other features of the router program because:

1. It simulates the registration process, which is a prerequisite for all other router program operations.
2. By successfully calling the `register` function, it establishes the necessary account such as `IntegratorConfig` and permissions required for subsequent router program interactions.
3. It allows us to test the router program's handling of Cross-Program Invocations (CPIs) from an integrator.

This program contains a single instruction:

1. `invoke_register`: This instruction demonstrates how to properly set up the accounts and sign the transaction using a PDA, which is required for the registration process.

By first calling `invoke_register`, we can then proceed to test all other aspects of the router program, ensuring its complete functionality in a controlled testing environment.

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
- `InvokeRegisterArgs`: The struct that defines the arguments for the invoke_register instruction.
- `InvokeRegister`: The struct that defines the accounts required for the invoke_register instruction.
