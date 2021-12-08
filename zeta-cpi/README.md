## Developing

This program requires building the Serum DEX from source, which is done using
git submodules.

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

### Anchor

```sh
cargo install --git https://github.com/project-serum/anchor --tag v0.18.2 anchor-cli --locked

# If cargo install fails, you may need additional dependencies

sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev
```

### Run integration test

```sh
# Configure solana network to devnet.
solana config set --url devnet

# You will need to airdrop yourself some devnet solana to deploy the zeta cpi program.
solana airdrop 5

# Install packages.
yarn install

# Build CPI example.
anchor build

# Run integration test.
anchor test
```
