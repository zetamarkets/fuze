<div align="center">
  <img height="120px" src="./logo.png" />

  <h1 style="margin-top: 0px">Zeta FuZe 🧬</h1>

  <p>
    <a href="https://discord.gg/dD7YREfBkR"
      ><img
        alt="Discord Chat"
        src="https://img.shields.io/discord/841556000632078378?color=blueviolet"
    /></a>
    <a href="https://opensource.org/licenses/Apache-2.0"
      ><img
        alt="License"
        src="https://img.shields.io/github/license/project-serum/anchor?color=blueviolet"
    /></a>
  </p>
</div>

# Zeta FuZe

FuZe is Zeta's cross-program integration ecosystem.

This repository contains the Zeta Cross Program Invocation (CPI) interface as well as a number of helpful examples and reference implementations on how to compose (fuse) with the Zeta DEX.

## Networks

| Key     |                     Value                      |
| ------- | :--------------------------------------------: |
| Devnet  |     <span style="color:green">Live</span>      |
| Mainnet | <span style="color:green">Live</span> |

## Cross Program Invocations

### Instructions

The instructions currently supported are as follows:

- `initialize_margin_account` - create and initialize a user's margin account
- `deposit` - deposit USDC collateral into the margin account
- `withdraw` - withdraw USDC collateral from the margin account
- `place_order` - place an order of (price, size, side) on the relevant market
- `cancel_order` - cancel a specified order

### Accounts

The accounts and relevant data that is currently supported (non-exhaustive):

- `ZetaGroup` - contains information relating to all derivatives market for an underlying
  - Underlying
  - Serum Market
  - Strike
  - Kind (Call, Put, Future)
  - Expiry
- `Greeks`
  - Mark Price
  - Delta
  - Vega
  - IV
- `MarginAccount`
  - Balance
  - Positions
- `Oracle`
  - Price

## Programs

### zeta-cpi

Basic usage examples outlined in a dummy proxy program that simply calls the main zeta program instructions. Also includes account layouts and outlines how to read all relevant on-chain data from the Zeta program.
This should give all the boilerplate needed to execute core program functionality both on the Rust program and Typescript client (via `tests/zeta_cpi.ts`).

### examples/vault-put-sell (WIP)

_Work in progress_

Reference implementation for a put selling vault that uses the Zeta DEX under the hood.

## Feature Requests

- [x] Zeta program interface
- [x] Core CPI instruction examples
- [x] Examples on how to read and deserialize Zeta account data
- [x] Typescript client examples
- [ ] Proper tests (current tests don't really check and validate state)
- [ ] Put selling vault sample
- [ ] Multi-leg product vault e.g. straddles
