import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ZetaCpi } from "../target/types/zeta_cpi";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  utils,
  Exchange,
  Wallet,
  types,
  Network,
  constants,
} from "@zetamarkets/sdk";
import * as https from "https";
import { TextEncoder } from "util";
import * as assert from "assert";

// Airdrop amounts
const SOL_AMOUNT = 1; // 1 SOL
const USDC_AMOUNT = 10_000;

const SERVER_URL = "server.zeta.markets";

const zetaProgram = new anchor.web3.PublicKey(
  "268ZKaPTbRnnse9ohE5MdgWniLrtnJcjvAAwLmZMDbj3"
);
const underlyingMint = new anchor.web3.PublicKey(
  "So11111111111111111111111111111111111111112"
);
const pythSolOracle = new anchor.web3.PublicKey(
  "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix"
);
const dexProgram = new anchor.web3.PublicKey(
  "DEX6XtaRGm4cNU2XE18ykY4RMAY3xdygdkas7CdhMLaF"
);

let airdropUsdc = async (userPubkey: anchor.web3.PublicKey, amount: number) => {
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
      let body = "";
      res.on("data", (chunk) => (body += chunk.toString()));
      res.on("error", reject);
      res.on("end", () => {
        if (res.statusCode >= 200 && res.statusCode <= 299) {
          resolve({
            statusCode: res.statusCode,
            headers: res.headers,
            body: body,
          });
        } else {
          reject(
            "Request failed. status: " + res.statusCode + ", body: " + body
          );
        }
      });
    });
    req.on("error", reject);
    req.write(data, "binary");
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

  let [zetaGroupAddress, _zetaGroupNonce] = [undefined, undefined];
  let [marginAddress, _marginNonce] = [undefined, undefined];
  let [stateAddress, _stateNonce] = [undefined, undefined];
  let [vaultAddress, _vaultNonce] = [undefined, undefined];
  let usdcMintAddress = undefined;
  let usdcAccountAddress = undefined;
  let [greeksAddress, _greeksNone] = [undefined, undefined];
  let [serumAuthorityAddress, _serumAuthorityNonce] = [undefined, undefined];
  let [openOrdersAccount, openOrdersNonce] = [undefined, undefined];
  let [openOrdersMapAddress, openOrdersMapNonce] = [undefined, undefined];
  let market = undefined;

  it("Setup by sourcing addresses and airdropping SOL", async () => {
    [zetaGroupAddress, _zetaGroupNonce] = await utils.getZetaGroup(
      zetaProgram,
      underlyingMint
    );
    [marginAddress, _marginNonce] = await utils.getMarginAccount(
      zetaProgram,
      zetaGroupAddress,
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
    [greeksAddress, _greeksNone] = await utils.getGreeks(
      zetaProgram,
      zetaGroupAddress
    );
    [serumAuthorityAddress, _serumAuthorityNonce] =
      await utils.getSerumAuthority(zetaProgram);

    // Load the exchange object
    await Exchange.load(
      zetaProgram,
      Network.DEVNET,
      provider.connection,
      utils.defaultCommitment(),
      undefined,
      0
    );

    // Pick 0 index market arbitrarily
    const frontIndex =
      Exchange.zetaGroup.frontExpiryIndex * constants.PRODUCTS_PER_EXPIRY;
    market = Exchange.markets.markets[frontIndex];

    [openOrdersAccount, openOrdersNonce] = await utils.getOpenOrders(
      zetaProgram,
      market.address,
      userKeypair.publicKey
    );

    [openOrdersMapAddress, openOrdersMapNonce] = await utils.getOpenOrdersMap(
      zetaProgram,
      openOrdersAccount
    );

    console.log(`User: ${userKeypair.publicKey}`);
    console.log(`Zeta group account: ${zetaGroupAddress}`);
    console.log(`Margin account: ${marginAddress}`);

    // Airdrop SOL
    const signature = await provider.connection.requestAirdrop(
      userKeypair.publicKey,
      SOL_AMOUNT * 10 ** 9 // lamports
    );
    await connection.confirmTransaction(signature);
  });

  it("Create margin account via CPI", async () => {
    // FYI can only create this once
    const tx = await program.rpc.createMarginAccount({
      accounts: {
        zetaProgram: zetaProgram,
        zetaGroup: zetaGroupAddress,
        marginAccount: marginAddress,
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
        zetaGroup: zetaGroupAddress,
        marginAccount: marginAddress,
        authority: userKeypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
    console.log("Your transaction signature", tx);
  });

  it("Deposit USDC into margin account via CPI", async () => {
    let usdcAccount = await provider.connection.getAccountInfo(
      usdcAccountAddress
    );
    // Mint USDC if they don't have an acct
    if (usdcAccount == null) {
      console.info("USDC account doesn't exist, airdropping USDC");

      const body = {
        key: userKeypair.publicKey.toString(),
        amount: USDC_AMOUNT,
      };
      let req = await airdropUsdc(userKeypair.publicKey, USDC_AMOUNT);
    } else {
      console.info("USDC exists, proceeding");
    }

    usdcAccount = await provider.connection.getAccountInfo(usdcAccountAddress);
    assert.ok(usdcAccount !== undefined);

    // Deposit all newly minted USDC into the margin account
    const tx = await program.rpc.deposit(
      new anchor.BN(utils.getNativeAmount(USDC_AMOUNT)),
      {
        accounts: {
          zetaProgram: zetaProgram,
          state: stateAddress,
          zetaGroup: zetaGroupAddress,
          marginAccount: marginAddress,
          vault: vaultAddress,
          userTokenAccount: usdcAccountAddress,
          authority: userKeypair.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      }
    );
    console.log("Your transaction signature", tx);
  });

  it("Withdraw USDC out of margin account via CPI", async () => {
    // Withdraw 10% of deposited funds
    const tx = await program.rpc.withdraw(
      new anchor.BN(utils.getNativeAmount(0.1 * USDC_AMOUNT)),
      {
        accounts: {
          zetaProgram: zetaProgram,
          state: stateAddress,
          zetaGroup: zetaGroupAddress,
          marginAccount: marginAddress,
          vault: vaultAddress,
          userTokenAccount: usdcAccountAddress,
          authority: userKeypair.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          greeks: greeksAddress,
          oracle: pythSolOracle,
        },
      }
    );
    console.log("Your transaction signature", tx);
  });

  it("Initialize open orders account via CPI", async () => {
    const tx = await program.rpc.initializeOpenOrders({
      accounts: {
        zetaProgram: zetaProgram,
        state: stateAddress,
        zetaGroup: zetaGroupAddress,
        dexProgram: dexProgram,
        systemProgram: anchor.web3.SystemProgram.programId,
        openOrders: openOrdersAccount,
        marginAccount: marginAddress,
        authority: userKeypair.publicKey,
        market: market.address,
        serumAuthority: serumAuthorityAddress,
        openOrdersMap: openOrdersMapAddress,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
    });
    console.log("Your transaction signature", tx);
  });

  it("Place order via CPI", async () => {
    const side = types.Side.BID;

    const marketAccounts = {
      market: market.serumMarket.decoded.ownAddress,
      requestQueue: market.serumMarket.decoded.requestQueue,
      eventQueue: market.serumMarket.decoded.eventQueue,
      bids: market.serumMarket.decoded.bids,
      asks: market.serumMarket.decoded.asks,
      coinVault: market.serumMarket.decoded.baseVault,
      pcVault: market.serumMarket.decoded.quoteVault,
      orderPayerTokenAccount:
        side == types.Side.BID ? market.quoteVault : market.baseVault,
      coinWallet: market.baseVault,
      pcWallet: market.quoteVault,
    };

    const tx = await program.rpc.placeOrder(
      new anchor.BN(utils.getNativeAmount(1)),
      1,
      types.toProgramSide(side),
      {
        accounts: {
          zetaProgram: zetaProgram,
          state: stateAddress,
          zetaGroup: zetaGroupAddress,
          marginAccount: marginAddress,
          authority: userKeypair.publicKey,
          dexProgram: dexProgram,
          tokenProgram: TOKEN_PROGRAM_ID,
          serumAuthority: serumAuthorityAddress,
          greeks: greeksAddress,
          openOrders: openOrdersAccount,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          marketAccounts: marketAccounts,
          oracle: pythSolOracle,
        },
      }
    );
    console.log("Your transaction signature", tx);
  });
});
