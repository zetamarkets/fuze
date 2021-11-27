import * as anchor from "@project-serum/anchor";
import { utils, Exchange, Network } from "@zetamarkets/sdk";

describe("zeta_cpi", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const userKey = provider.wallet.publicKey;

  const program = anchor.workspace.ZetaCpi;
  const zetaProgram = new anchor.web3.PublicKey(
    "GoB7HN9PAumGbFBZUWokX7GiNe8Etcsc22JWmarRhPBq"
  );
  const underlyingMint = new anchor.web3.PublicKey(
    "So11111111111111111111111111111111111111112"
  );

  let [zetaGroup, _zetaGroupNonce] = [undefined, undefined];
  let [marginAccount, _marginNonce] = [undefined, undefined];

  // it("Create margin account via CPI", async () => {
  //   [zetaGroup, _zetaGroupNonce] = await utils.getZetaGroup(
  //     zetaProgram,
  //     underlyingMint
  //   );
  //   [marginAccount, _marginNonce] = await utils.getMarginAccount(
  //     zetaProgram,
  //     zetaGroup,
  //     userKey
  //   );

  //   console.log(`User: ${userKey}`);
  //   console.log(`Zeta group account: ${zetaGroup}`);
  //   console.log(`Margin account: ${marginAccount}`);

  //// TODO: remove for debugging
  // await Exchange.load(
  //   zetaProgram,
  //   Network.DEVNET,
  //   provider.connection,
  //   utils.defaultCommitment(),
  //   provider.wallet,
  //   0
  // );
  // const txix = await Exchange.program.instruction.createMarginAccount(
  //   _marginNonce,
  //   {
  //     accounts: {
  //       marginAccount: marginAccount,
  //       authority: userKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       zetaProgram: zetaProgram,
  //       zetaGroup: zetaGroup,
  //     },
  //   }
  // );
  // console.log(`Create tx: ${txix}`);
  ////

  // FYI can only create this once
  // const tx = await program.rpc.createMarginAccount({
  //   accounts: {
  //     marginAccount: marginAccount,
  //     authority: userKey,
  //     systemProgram: anchor.web3.SystemProgram.programId,
  //     zetaProgram: zetaProgram,
  //     zetaGroup: zetaGroup,
  //   },
  // });
  // console.log("Your transaction signature", tx);
  // });

  // it("Init margin account via CPI", async () => {
  //   const tx = await program.rpc.initializeMarginAccount({
  //     accounts: {
  //       zetaProgram: zetaProgram,
  //       zetaGroup: zetaGroup,
  //       marginAccount: marginAccount,
  //       authority: userKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     },
  //   });
  //   console.log("Your transaction signature", tx);
  // });
});
