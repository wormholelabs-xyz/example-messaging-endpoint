# Anchor

This folder was generated using [Anchor](https://www.anchor-lang.com/) with `anchor init --no-git`.

## Testing

```bash
anchor test
```

## Building

```bash
anchor build
```

For verifiable builds use

```bash
anchor build --verifiable
```

## Deploying

### Solana Devnet

```bash
anchor deploy --provider.cluster devnet --provider.wallet ~/.config/solana/your-key.json
```

### Mainnet

```bash
anchor deploy --provider.cluster mainnet --provider.wallet ~/.config/solana/your-key.json
```

### Upgrading

```
anchor upgrade --provider.cluster <network> --provider.wallet ~/.config/solana/your-key.json --program-id <PROGRAM_ID> target/deploy/svm.so
```

If you get an error like this

```
Error: Deploying program failed: RPC response error -32002: Transaction simulation failed: Error processing Instruction 0: account data too small for instruction [3 log messages]
```

Don't fret! Just extend the program size.

```
solana program -u <network> -k ~/.config/solana/your-key.json extend <PROGRAM_ID> <ADDITIONAL_BYTES>
```

You can view the current program size with `solana program -u <network> show <PROGRAM_ID>`.
