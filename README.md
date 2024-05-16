# Pryzm-Network-Test

## Problem

Using PRYZM indigo-1, implement and deploy a CW20 token (Called Wtoken) contract and a reward distribution contract: 

The reward contract is supposed to receive some USDsim token every 24 hours; users should be able to send their CW20 token to the reward contract and based on the amount they send to the reward contract and the time they sent their token should be incentivized from the USDsim that has been sent to the contract. 

We expect the reward contract to be: 

- Gas efficient
- Readable
- Have tests

## Example:

User A sends 10 Wtoken to the reward contract and user B sends 90 Wtoken to the reward contract, user C sends 100 Wtoken a day later. The contract receives 1000USDsim, therefore, 

user A should receive 100 USDsim 

user B should receive 900 USDsim 

user C should receive 0 USDsim

one day later the contract receives another 1000 USDsim, the reward distribution should be: 

user A should receive 50 USDsim 

user B should receive 450 USDsim 

user C should receive 500 USDsim

## Pryzm Testnet User Guide
[Pryzm Testnet User Guide](https://docs.pryzm.zone/overview/guide/testnet-guide/)

## Setting up development environment

### Install Nix

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

### Enter development environment

```bash
nix develop --impure
```

It will enter into a virtual environment and install most of the development requirements.

Install other requirements:

```bash
dev-install-cosmwasm-check
```

```bash
dev-install-wasmd
```

```bash
dev-install-injective
```

After those commands, `cosmwasm-check`, `wasmd`, `injectived` and `dev-setup-injective` would be available.

For exiting from virtual environment just run `exit`.

## Running Project

For each contract:

### Test

```bash
cargo test
```

### Build

For debug purpose:

```bash
cargo wasm
```

After that `<contract>.wasm` would be avaiable on `target/wasm32-unknown-unknown/release/` directory

For production purpose:

```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.15.0
```

It will generate optimized build for production purpose.

After that `<contract>.wasm` would be available in `artifacts` directory.

## Project structure

We followed the best practices project structure by cosmwasm book and cw20base project.

here is the summery:

```
wtoken
├── Cargo.toml
└── src
    ├── bin
    │   └── schema.rs   -- to see json schema of contracts (instanciate, query, execute)
    ├── contract.rs     -- instanciate, query, execute entry points
    ├── error.rs        -- define all errors
    ├── lib.rs          -- exporting modules
    └── msg.rs          -- message types that use in contract.rs
```

```
reward-contract
├── Cargo.toml
└── src
    ├── bin
    │   └── schema.rs
    ├── contract.rs
    ├── error.rs
    ├── lib.rs
    ├── msg.rs
    └── state.rs        -- persistane state
```
