### Install Dependencies

### Rust

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup component add rustfmt
```

### Solana

```sh
sh -c "$(curl -sSfL https://release.solana.com/v1.8.5/install)"
```

### Mocha

```sh
npm install -g mocha
```

### Yarn
```sh
# Used for javascript package management.
npm install -g yarn
```

### Anchor

```sh
# Install using pre-build binary on x86_64 Linux.
# Only x86_64 Linux is supported currently, you must build from source for other OS'.
npm i -g @project-serum/anchor-cli

# Build from source for other operating systems.
cargo install --git https://github.com/project-serum/anchor --tag v0.24.2 anchor-cli --locked

# If cargo install fails, you may need additional dependencies.
sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev
```

### Variables

You will need to modify the environment variable at `./.env` to the latest program id.

| Key         |                    Value                     |
| ----------- | :------------------------------------------: |
| NETWORK_URL |        https://api.devnet.solana.com         |
| PROGRAM_ID  | BG3oRikW8d16YjUEmX3ZxHm9SiJzrGtMhsSR8aCw1Cd7 |
| SERVER_URL  |         https://server.zeta.markets          |

Devnet PROGRAM_ID is subject to change based on redeployments.

### Run integration test

```sh
# Configure solana network to devnet.
solana config set --url devnet

# You will need to airdrop yourself some devnet solana to deploy the zeta cpi program.
solana airdrop 2

# Install packages.
yarn install

# Build CPI example.
anchor build

# Run integration test.
anchor test
```
