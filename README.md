# Multi-Chain Template

## Overview

This repository serves as a template monorepo for developing on multiple blockchains. The goal is to provide the generally recommended starting setups for each runtime along with pre-configured CI.

## Runtime Support

- [x] [EVM](https://ethereum.org/en/developers/docs/evm/)
- [x] [SVM](https://solana.com/developers/evm-to-svm/smart-contracts)
- [ ] [Sui Move](https://sui.io/move)
- [ ] [Aptos Move](https://aptos.dev/en/build/smart-contracts)

## Developer Dependencies

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

## Recommended GitHub Settings

### General

- Default branch: `main`
- *Dis*allow merge commits
- Always suggest updating pull request branches
- Automatically delete head branches

### Rules / Rulesets

- Ruleset Name: Require PRs
- Enforcement status: Active
- Target branches / Add target: Include default branch
- Branch rules
  - Restrict deletions
  - Require linear history
  - Require a pull request before merging
    - Required approvals: 1 (or more/less depending on your number of contributors)
    - Dismiss stale pull request approvals when new commits are pushed
    - Require review from Code Owners
    - Require approval of the most recent reviewable push
    - Require conversation resolution before merging
  - Require status checks to pass
    - Require branches to be up to date before merging
    - Add checks
      - Anchor Test
      - Foundry project
      - prettier
      - spellcheck
  - Block force pushes
- Restrictions [Enterprise Only]
  - Restrict commit metadata
    - Add restriction
      - Applies to: Commit message
      - Requirement: Must match a given regex pattern
      - Matching pattern:
        - `^(ci|evm|svm|readme){1}(\([\w\-\.]+\))?(!)?: ([\w ])+([\s\S]*)` for file/folder prefixes
        - or `^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test){1}(\([\w\-\.]+\))?(!)?: ([\w ])+([\s\S]*)` to [enforce conventional commits](https://docs.github.com/en/enterprise-cloud@latest/organizations/managing-organization-settings/creating-rulesets-for-repositories-in-your-organization#enforce-conventional-commits)
