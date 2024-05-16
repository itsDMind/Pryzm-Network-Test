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
install-cosmwasm-check
```

```bash
install-wasmd
```

After those commands, `cosmwasm-check` and `wasmd` would be available.

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

```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.15.0
```

### Deploy

TODO

### Run

TODO
