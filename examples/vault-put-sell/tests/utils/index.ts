import * as anchor from "@project-serum/anchor";

export interface IVaultBumps {
  vaultAccount: number;
  redeemableMint: number;
  vaultUsdc: number;
}

export interface IEpochTimes {
  startEpoch: anchor.BN;
  endDeposits: anchor.BN;
  endEpoch: anchor.BN;
  endEscrow: anchor.BN;
}

export function sleep(ms) {
  console.log("Sleeping for", ms / 1000, "seconds");
  return new Promise((resolve) => setTimeout(resolve, ms));
}
