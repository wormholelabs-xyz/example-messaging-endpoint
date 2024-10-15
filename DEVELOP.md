# GMP Router Development

## Dependencies

### Miscellaneous

- [Node.js](https://nodejs.org/en/download/package-manager)

Run `npm ci` at the root of this repo to install Prettier.

### EVM

- [Foundry](https://book.getfoundry.sh/getting-started/installation)

### SVM

- [Rust 1.75.0](https://www.rust-lang.org/tools/install)
- [Solana 1.18.17](https://solana.com/docs/intro/installation)
- [Yarn](https://yarnpkg.com/getting-started/install)
- [Anchor 0.30.1](https://www.anchor-lang.com/docs/installation)

Required versions are defined in [./svm/rust-toolchain.toml](./svm/rust-toolchain.toml) and [./svm/Anchor.toml](./svm/Anchor.toml)

## Recommended VSCode Settings

Recommended VSCode settings and extensions have been included as workspace settings in this repository (`.vscode`).

This includes:

- Foundry's [forge formatting](https://book.getfoundry.sh/config/vscode#3-formatter)
- [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode)
  - This should work after running `npm ci` at the root of this repo.

Additional, related settings may be required based on your use.
