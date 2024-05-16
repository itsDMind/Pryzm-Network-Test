# Pryzm-Test

Pryzm Network Test

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
