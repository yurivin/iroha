# Iroha Bridge

A bridge between Iroha and Substrate blockchains.

_Warning: this repository is a part of [Iroha](https://github.com/vmarkushin/iroha/tree/bridge-ms2) and will not work on it's own. To build or run the code you need to clone the Iroha repository:_
```
git clone --recurse-submodules --branch bridge-ms2 https://github.com/vmarkushin/iroha.git && cd iroha
```

## Project structure

- `substrate-iroha-bridge-node/` - bridge node folder
    - `node/` - runnable node
        - `src/`
            - `chain_spec.rs` - node configuration
            - ...
    - `runtime/...` - runtime used by the node
    - `pallets/`
        - `treasury/` - a pallet for managing assets
            - `src/...`
        - `iroha-bridge/` - Iroha-Substrate bridge pallet
            - `src/...`
            - `tests/...` - _tests are under development_
- `bridge-tester` - demo for the functionality

## Local Development

Follow these steps to prepare your local environment for Substrate development :hammer_and_wrench:

### Simple Method

You can install all the required dependencies with a single command (be patient, this can take up
to 30 minutes).

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

### Manual Method

Manual steps for Linux-based systems can be found below; you can
[find more information at substrate.dev](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./iroha_substrate/substrate-iroha-bridge-node/scripts/init.sh
```

### Build

Once you have prepared your local development environment, you can build the node template. Use this
command to build the [Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution)
and [native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```

## Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
./target/release/substrate-iroha-bridge-node purge-chain --dev
```

Start a development chain with:

```bash
./target/release/substrate-iroha-bridge-node --dev
```

Detailed logs may be shown by running the node with the following environment variables set:
`RUST_LOG=debug RUST_BACKTRACE=1 cargo run --package substrate-iroha-bridge-node -- --dev`.

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

#### Build only

To have a clean environment with ability to manually run nodes use the following command:

```bash
./iroha_substrate/substrate-iroha-bridge-node/scripts/docker_run.sh
```

This command will compile the code and let you run process manually.

#### Run precompiled binaries

You can run the precompiled binaries for the bridge including the bridge tester.

```bash
./iroha_substrate/substrate-iroha-bridge-node/scripts/docker_run_precompiled.sh
```

