import * as anchor from "@project-serum/anchor";

export interface IVaultBumps {
  idoAccount: number;
  redeemableMint: number;
  poolWatermelon: number;
  poolUsdc: number;
}

export interface IEpochTimes {
  startIdo: anchor.BN;
  endDeposits: anchor.BN;
  endIdo: anchor.BN;
  endEscrow: anchor.BN;
}

export function sleep(ms) {
  console.log("Sleeping for", ms / 1000, "seconds");
  return new Promise((resolve) => setTimeout(resolve, ms));
}
