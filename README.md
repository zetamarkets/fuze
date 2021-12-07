# zeta-cpi

Cross program invocation examples for the Zeta Markets program, useful for on-chain integration.

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

- `State` - general exchange setup parameters
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

### zeta_cpi

Basic usage examples outlined in a dummy proxy program that simply calls the main zeta program instructions. Also includes account layouts and outlines how to read all relevant on-chain data from the Zeta program.
This should give all the boilerplate needed to execute core program functionality both on the Rust program and Typescript client (via `tests/zeta_cpi.ts`).

### zeta_demo_vault (WIP)

_Work in progress_

A naive vault implementation. Aiming to implement put selling vault boilerplate first.

## Feature Requests

- [x] Zeta program interface
- [x] Core CPI instruction examples
- [x] Examples on how to read and deserialize Zeta account data
- [x] Typescript client examples
- [ ] Proper tests (current tests don't really check and validate state)
- [ ] Put selling vault sample
- [ ] Multi-leg product vault e.g. straddles
