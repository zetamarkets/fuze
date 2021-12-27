require("dotenv").config({ path: __dirname + `/../.env` });
import * as anchor from "@project-serum/anchor";
import { ZetaFlexCpi } from "../target/types/zeta_flex_cpi";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  utils,
  Exchange,
  Wallet,
  types,
  Network,
  Client,
  constants,
  Market,
} from "@zetamarkets/sdk";
import * as assert from "assert";

describe("zeta-flex-cpi", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.ZetaFlexCpi as anchor.Program<ZetaFlexCpi>;

  it("Initialize auction via CPI", async () => {
    let 
    const tx = await program.rpc.initializeAuction({});
    console.log("Your transaction signature", tx);
  });
});
