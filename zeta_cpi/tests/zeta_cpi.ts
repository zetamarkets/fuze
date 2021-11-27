import * as anchor from "@project-serum/anchor";
import { utils, Exchange } from "@zetamarkets/sdk";

const ZETA_GROUP_SEED = "zeta-group";
const MARGIN_SEED = "margin";

describe("zeta_cpi", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const userKey = provider.wallet.publicKey;

  it("Is initialized!", async () => {
    // Add your test here.
    const program = anchor.workspace.ZetaCpi;
    const zetaProgram = new anchor.web3.PublicKey(
      "GoB7HN9PAumGbFBZUWokX7GiNe8Etcsc22JWmarRhPBq"
    );
    const underlyingMint = new anchor.web3.PublicKey(
      "So11111111111111111111111111111111111111112"
    );
    const [zetaGroup, _zetaGroupNonce] = await utils.getZetaGroup(
      zetaProgram,
      underlyingMint
    );
    const [marginAccount, _marginNonce] = await utils.getMarginAccount(
      zetaProgram,
      zetaGroup,
      userKey
    );
    console.log(`User: ${userKey}`);
    console.log(`Zeta group account: ${zetaGroup}`);
    console.log(`Margin account: ${zetaGroup}`);
    const tx = await program.rpc.initializeMarginAccount({
      accounts: {
        zetaProgram: zetaProgram,
        zetaGroup: zetaGroup,
        marginAccount: marginAccount,
        authority: userKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
    console.log("Your transaction signature", tx);
  });
});
