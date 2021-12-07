require("dotenv").config({ path: __dirname + `/../.env` });
import * as anchor from "@project-serum/anchor";
import { ZetaCpi } from "../target/types/zeta_cpi";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  utils,
  Exchange,
  Wallet,
  types,
  Network,
  Client,
} from "@zetamarkets/sdk";
import * as https from "https";
import { TextEncoder } from "util";
import * as assert from "assert";

// Airdrop amounts
const SOL_AMOUNT = 1; // 1 SOL
const USDC_AMOUNT = 10_000; // 10k USDC

const SERVER_URL = "server.zeta.markets";

// Constants for addresses
const zetaProgram = new anchor.web3.PublicKey(process.env!.zeta_program);
const underlyingMint = new anchor.web3.PublicKey(
  process.env.underlying_mint || "So11111111111111111111111111111111111111112"
);
const pythOracle = new anchor.web3.PublicKey(
  process.env.pyth_oracle || "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix"
);
const dexProgram = new anchor.web3.PublicKey(
  process.env.dex_program || "DEX6XtaRGm4cNU2XE18ykY4RMAY3xdygdkas7CdhMLaF"
);

// Helper function to make the post request to server to mint devnet dummy USDC collateral
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

  const program = anchor.workspace.ZetaCpi as anchor.Program<ZetaCpi>;

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
  let [marketNodeAddress, _marketNodeNonce] = [undefined, undefined];
  let market = undefined;
  let client = undefined;
  let side = undefined;

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
    [vaultAddress, _vaultNonce] = await utils.getVault(
      zetaProgram,
      zetaGroupAddress
    );
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

    // Load the client
    client = await Client.load(
      provider.connection,
      provider.wallet,
      utils.defaultCommitment(),
      undefined,
      false
    );

    // Arbitrarily choosing the nearest expiry, lowest strike call
    const expiryIndex = Exchange.zetaGroup.frontExpiryIndex;
    const marketIndex = 0;
    market = Exchange.markets.getMarketsByExpiryIndex(expiryIndex)[marketIndex];

    // Select the trade side for when we test place and cancel order
    side = types.Side.BID;

    [marketNodeAddress, _marketNodeNonce] = await utils.getMarketNode(
      zetaProgram,
      zetaGroupAddress,
      marketIndex
    );

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

  it("Init margin account via CPI", async () => {
    // FYI can only init this once
    const tx = await program.rpc.initializeMarginAccount({
      accounts: {
        zetaProgram: zetaProgram,
        initializeMarginCpiAccounts: {
          zetaGroup: zetaGroupAddress,
          marginAccount: marginAddress,
          authority: userKeypair.publicKey,
          zetaProgram: zetaProgram,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
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
      new anchor.BN(utils.convertDecimalToNativeInteger(USDC_AMOUNT)),
      {
        accounts: {
          zetaProgram: zetaProgram,
          depositCpiAccounts: {
            state: stateAddress,
            zetaGroup: zetaGroupAddress,
            marginAccount: marginAddress,
            vault: vaultAddress,
            userTokenAccount: usdcAccountAddress,
            authority: userKeypair.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
        },
      }
    );
    console.log("Your transaction signature", tx);
  });

  it("Withdraw USDC out of margin account via CPI", async () => {
    // Withdraw 10% of deposited funds
    const tx = await program.rpc.withdraw(
      new anchor.BN(utils.convertDecimalToNativeInteger(0.1 * USDC_AMOUNT)),
      {
        accounts: {
          zetaProgram: zetaProgram,
          withdrawCpiAccounts: {
            state: stateAddress,
            zetaGroup: zetaGroupAddress,
            marginAccount: marginAddress,
            vault: vaultAddress,
            userTokenAccount: usdcAccountAddress,
            authority: userKeypair.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            greeks: greeksAddress,
            oracle: pythOracle,
          },
        },
      }
    );
    console.log("Your transaction signature", tx);
  });

  it("Initialize open orders account via CPI", async () => {
    const tx = await program.rpc.initializeOpenOrders({
      accounts: {
        zetaProgram: zetaProgram,
        initializeOpenOrdersCpiAccounts: {
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
      },
    });
    console.log("Your transaction signature", tx);
  });

  it("Place order via CPI", async () => {
    const marketAccounts = {
      market: market.address,
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
      new anchor.BN(utils.convertDecimalToNativeInteger(1)),
      1,
      types.toProgramSide(side),
      {
        accounts: {
          zetaProgram: zetaProgram,
          placeOrderCpiAccounts: {
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
            oracle: pythOracle,
            marketNode: marketNodeAddress,
          },
        },
      }
    );
    console.log("Your transaction signature", tx);
  });

  it("Cancel order via CPI", async () => {
    await client.updateState();
    const orders = client.orders.filter(
      (x) => x.marketIndex === market.marketIndex
    );
    if (orders.length === 0) {
      throw new Error("No relevant client order to cancel");
    }

    const cancelAccounts = {
      zeta_group: zetaGroupAddress,
      state: stateAddress,
      margin_account: marginAddress,
      dex_program: dexProgram,
      serum_authority: serumAuthorityAddress,
      open_orders: openOrdersAccount,
      market: market.address,
      bids: market.serumMarket.decoded.bids,
      asks: market.serumMarket.decoded.asks,
      event_queue: market.serumMarket.decoded.eventQueue,
    };

    const tx = await program.rpc.placeOrder(
      types.toProgramSide(side),
      orders[0].orderId,
      {
        accounts: {
          zetaProgram: zetaProgram,
          placeOrderCpiAccounts: {
            authority: userKeypair.publicKey,
            cancelAccounts: cancelAccounts,
          },
        },
      }
    );
    console.log("Your transaction signature", tx);
  });

  it("Read Zeta data", async () => {
    const tx = await program.rpc.readProgramData({
      accounts: {
        state: stateAddress,
        zetaGroup: zetaGroupAddress,
        marginAccount: marginAddress,
        greeks: greeksAddress,
        oracle: pythOracle,
      },
    });
    console.log("Your transaction signature", tx);
  });

  // Closes the account subscriptions so the test won't hang.
  it("BOILERPLATE: Close websockets.", async () => {
    await Exchange.close();
    await client.close();
  });
});
