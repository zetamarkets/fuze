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
  constants,
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
const underlyingMint = constants.MINTS["SOL"];
const pythOracle = constants.PYTH_PRICE_FEEDS[Network.DEVNET]["SOL/USD"];
const dexProgram = constants.DEX_PID;

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

describe("zeta-cpi", () => {
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

  let zetaGroup,
    margin,
    state,
    vault,
    usdcMint,
    userUsdc,
    greeks,
    serumAuthority,
    openOrders,
    openOrdersMap,
    marketNode,
    market,
    client,
    side;

  it("Setup by sourcing addresses and airdropping SOL", async () => {
    [zetaGroup] = await utils.getZetaGroup(zetaProgram, underlyingMint);
    [margin] = await utils.getMarginAccount(
      zetaProgram,
      zetaGroup,
      userKeypair.publicKey
    );
    [state] = await utils.getState(zetaProgram);
    [vault] = await utils.getVault(zetaProgram, zetaGroup);
    usdcMint = await utils.getTokenMint(provider.connection, vault);
    userUsdc = await utils.getAssociatedTokenAddress(
      usdcMint,
      userKeypair.publicKey
    );
    [greeks] = await utils.getGreeks(zetaProgram, zetaGroup);
    [serumAuthority] = await utils.getSerumAuthority(zetaProgram);

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
      new Wallet(userKeypair),
      utils.defaultCommitment(),
      undefined,
      false
    );

    // Arbitrarily choosing the nearest expiry, lowest strike call (productIndex 0)
    const expiryIndex = Exchange.zetaGroup.frontExpiryIndex; // [0,2)
    const productIndex = 0; // [0,23)
    const marketIndex =
      expiryIndex * constants.PRODUCTS_PER_EXPIRY + productIndex; // [0,46)
    market =
      Exchange.markets.getMarketsByExpiryIndex(expiryIndex)[productIndex];

    // Select the trade side for when we test place and cancel order
    side = types.Side.BID;

    [marketNode] = await utils.getMarketNode(
      zetaProgram,
      zetaGroup,
      marketIndex
    );

    [openOrders] = await utils.getOpenOrders(
      zetaProgram,
      market.address,
      userKeypair.publicKey
    );

    [openOrdersMap] = await utils.getOpenOrdersMap(zetaProgram, openOrders);

    console.log(`User: ${userKeypair.publicKey}`);
    console.log(`Zeta group account: ${zetaGroup}`);
    console.log(`Margin account: ${margin}`);

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
          zetaGroup: zetaGroup,
          marginAccount: margin,
          authority: userKeypair.publicKey,
          zetaProgram: zetaProgram,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      },
    });
    console.log("Your transaction signature", tx);
  });

  it("Deposit USDC into margin account via CPI", async () => {
    let userUsdcAccount = await provider.connection.getAccountInfo(userUsdc);
    // Mint USDC if they don't have an acct
    if (userUsdcAccount == null) {
      console.info("USDC account doesn't exist, airdropping USDC");

      const body = {
        key: userKeypair.publicKey.toString(),
        amount: USDC_AMOUNT,
      };
      let req = await airdropUsdc(userKeypair.publicKey, USDC_AMOUNT);
    } else {
      console.info("USDC exists, proceeding");
    }

    userUsdcAccount = await provider.connection.getAccountInfo(userUsdc);
    assert.ok(userUsdcAccount !== undefined);

    // Deposit all newly minted USDC into the margin account
    const tx = await program.rpc.deposit(
      new anchor.BN(utils.convertDecimalToNativeInteger(USDC_AMOUNT)),
      {
        accounts: {
          zetaProgram: zetaProgram,
          depositCpiAccounts: {
            state: state,
            zetaGroup: zetaGroup,
            marginAccount: margin,
            vault: vault,
            userTokenAccount: userUsdc,
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
            state: state,
            zetaGroup: zetaGroup,
            marginAccount: margin,
            vault: vault,
            userTokenAccount: userUsdc,
            authority: userKeypair.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            greeks: greeks,
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
          state: state,
          zetaGroup: zetaGroup,
          dexProgram: dexProgram,
          systemProgram: anchor.web3.SystemProgram.programId,
          openOrders: openOrders,
          marginAccount: margin,
          authority: userKeypair.publicKey,
          market: market.address,
          serumAuthority: serumAuthority,
          openOrdersMap: openOrdersMap,
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
            state: state,
            zetaGroup: zetaGroup,
            marginAccount: margin,
            authority: userKeypair.publicKey,
            dexProgram: dexProgram,
            tokenProgram: TOKEN_PROGRAM_ID,
            serumAuthority: serumAuthority,
            greeks: greeks,
            openOrders: openOrders,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            marketAccounts: marketAccounts,
            oracle: pythOracle,
            marketNode: marketNode,
          },
        },
      }
    );
    console.log("Your transaction signature", tx);
  });

  it("Cancel order via CPI", async () => {
    await client.updateState();
    let orders = client.orders;
    if (orders.length === 0) {
      throw new Error("No relevant client order to cancel");
    }

    const cancelAccounts = {
      zetaGroup: zetaGroup,
      state: state,
      marginAccount: margin,
      dexProgram: dexProgram,
      serumAuthority: serumAuthority,
      openOrders: openOrders,
      market: market.address,
      bids: market.serumMarket.decoded.bids,
      asks: market.serumMarket.decoded.asks,
      eventQueue: market.serumMarket.decoded.eventQueue,
    };

    const tx = await program.rpc.cancelOrder(
      types.toProgramSide(side),
      orders[0].orderId,
      {
        accounts: {
          zetaProgram: zetaProgram,
          cancelOrderCpiAccounts: {
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
        state: state,
        zetaGroup: zetaGroup,
        marginAccount: margin,
        greeks: greeks,
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
