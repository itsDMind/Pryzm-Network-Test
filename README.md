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

For production purpose:


### Deploy

TODO

### Run

TODO
