# Pryzm-Test

Pryzm Network Test

## Setting up development environment

### Installing Nix

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

### Entering development environment

```bash
nix develop --impure
```

It will enter into a virtual environment and install most of the development requirements.

Installing other requirements:

```bash
install-cosmwasm-check
```

```bash
install-wasmd
```

After that commands `cosmwasm-check` and `wasmd` would be available.

For exiting from virtual environment just run `exit`.

## Running Project

### Test

```bash
cargo test
```

### Build

```bash
cargo wasm
```

### Deploy

TODO

### Run

TODO
