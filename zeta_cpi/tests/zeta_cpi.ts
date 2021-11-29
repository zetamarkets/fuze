import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ZetaCpi } from "../target/types/zeta_cpi";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { utils, Exchange, Wallet } from "@zetamarkets/sdk";
import * as https from "https";
import { TextEncoder } from "util";

// Airdrop amounts
const SOL_AMOUNT = 100_000_000;
const USDC_AMOUNT = 10_000;

const SERVER_URL = "server.zeta.markets";

let airdropUsdc = (userPubkey: anchor.web3.PublicKey, amount: number) => {
  const data = new TextEncoder().encode(
    JSON.stringify({
      key: userPubkey.toString(),
      amount: amount,
    })
  );
  const options = {
    hostname: `${SERVER_URL}`,
    port: 443,
    path: "/faucet/USDC",
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Content-Length": data.length,
    },
  };

  return new Promise((resolve, reject) => {
    const req = https.request(options, (res) => {
      console.log(`statusCode: ${res.statusCode}`);

      res.on("data", (d) => {
        process.stdout.write(d);
      });
    });

    req.on("error", (error) => {
      console.error(error);
    });

    req.write(data);
    req.end();
  });
};

describe("zeta_cpi", () => {
  // Configure the client.
  const userKeypair = anchor.web3.Keypair.generate();
  const url = "https://api.devnet.solana.com"; //process.env.ANCHOR_PROVIDER_URL;
  if (url === undefined) {
    throw new Error("ANCHOR_PROVIDER_URL is not defined");
  }
  const connection = new anchor.web3.Connection(url, utils.defaultCommitment());
  const provider = new anchor.Provider(
    connection,
    new Wallet(userKeypair),
    utils.defaultCommitment()
  );
  anchor.setProvider(provider);

  const program = anchor.workspace.ZetaCpi as Program<ZetaCpi>;
  const zetaProgram = new anchor.web3.PublicKey(
    "GoB7HN9PAumGbFBZUWokX7GiNe8Etcsc22JWmarRhPBq"
  );
  const underlyingMint = new anchor.web3.PublicKey(
    "So11111111111111111111111111111111111111112"
  );

  let [zetaGroup, _zetaGroupNonce] = [undefined, undefined];
  let [marginAccount, _marginNonce] = [undefined, undefined];
  let [stateAddress, _stateNonce] = [undefined, undefined];
  let [vaultAddress, _vaultNonce] = [undefined, undefined];
  let usdcMintAddress = undefined;
  let usdcAccountAddress = undefined;

  it("Setup by sourcing addresses and airdropping SOL", async () => {
    [zetaGroup, _zetaGroupNonce] = await utils.getZetaGroup(
      zetaProgram,
      underlyingMint
    );
    [marginAccount, _marginNonce] = await utils.getMarginAccount(
      zetaProgram,
      zetaGroup,
      userKeypair.publicKey
    );
    [stateAddress, _stateNonce] = await utils.getState(zetaProgram);
    [vaultAddress, _vaultNonce] = await utils.getVault(zetaProgram);
    usdcMintAddress = await utils.getTokenMint(
      provider.connection,
      vaultAddress
    );
    usdcAccountAddress = await utils.getAssociatedTokenAddress(
      usdcMintAddress,
      userKeypair.publicKey
    );

    console.log(`User: ${userKeypair.publicKey}`);
    console.log(`Zeta group account: ${zetaGroup}`);
    console.log(`Margin account: ${marginAccount}`);

    // Airdrop SOL
    await provider.connection.requestAirdrop(userKeypair.publicKey, SOL_AMOUNT);
  });

  it("Create margin account via CPI", async () => {
    // FYI can only create this once
    const tx = await program.rpc.createMarginAccount({
      accounts: {
        zetaProgram: zetaProgram,
        zetaGroup: zetaGroup,
        marginAccount: marginAccount,
        authority: userKeypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
    console.log("Your transaction signature", tx);
  });

  it("Init margin account via CPI", async () => {
    // FYI can only init this once
    const tx = await program.rpc.initializeMarginAccount({
      accounts: {
        zetaProgram: zetaProgram,
        zetaGroup: zetaGroup,
        marginAccount: marginAccount,
        authority: userKeypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
    console.log("Your transaction signature", tx);
  });

  it("Deposit USDC into margin account via CPI", async () => {
    const usdcAccount = await provider.connection.getAccountInfo(
      usdcAccountAddress
    );
    // Mint USDC if they don't have an acct
    if (usdcAccount == null) {
      console.info("USDC account doesn't exist, airdropping USDC");

      const body = {
        key: userKeypair.publicKey.toString(),
        amount: USDC_AMOUNT,
      };
      await airdropUsdc(userKeypair.publicKey, USDC_AMOUNT);
    } else {
      console.info("USDC exists, proceeding");
    }

    const tx = await program.rpc.deposit(new anchor.BN(USDC_AMOUNT), {
      accounts: {
        zetaProgram: zetaProgram,
        state: stateAddress,
        zetaGroup: zetaGroup,
        marginAccount: marginAccount,
        vault: vaultAddress,
        userTokenAccount: usdcAccountAddress,
        authority: userKeypair.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });
    console.log("Your transaction signature", tx);
  });
});
