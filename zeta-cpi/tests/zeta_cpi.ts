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
  Market,
  assets,
} from "@zetamarkets/sdk";
import * as assert from "assert";
import { mintUsdc } from "./utils";

// Airdrop amounts
const SOL_AMOUNT = 1; // 1 SOL
const USDC_AMOUNT = 10_000; // 10k USDC

// Constants for addresses
const zetaProgram = new anchor.web3.PublicKey(
  "BG3oRikW8d16YjUEmX3ZxHm9SiJzrGtMhsSR8aCw1Cd7"
);

describe("zeta-cpi", () => {
  // Configure the client.
  const userKeypair = anchor.web3.Keypair.generate();
  const url = "https://api.devnet.solana.com";
  if (url === undefined) {
    throw new Error("ANCHOR_PROVIDER_URL is not defined");
  }
  const connection = new anchor.web3.Connection(url, utils.defaultCommitment());
  const asset = assets.Asset.BTC;
  let allAssets = [asset];
  const provider = new anchor.AnchorProvider(
    connection,
    new Wallet(userKeypair),
    utils.defaultCommitment()
  );
  anchor.setProvider(provider);

  const program = anchor.workspace.ZetaCpi as anchor.Program<ZetaCpi>;

  const pythOracle = constants.PYTH_PRICE_FEEDS[Network.DEVNET][asset];

  let usdcMint,
    userUsdc,
    openOrders,
    openOrdersMap,
    market: Market,
    client: Client,
    side: types.Side;

  it("Setup by sourcing addresses and airdropping SOL", async () => {
    // Load the exchange object
    await Exchange.load(
      allAssets,
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

    usdcMint = await utils.getTokenMint(
      provider.connection,
      Exchange.getVaultAddress(asset)
    );
    userUsdc = await utils.getAssociatedTokenAddress(
      usdcMint,
      userKeypair.publicKey
    );

    // Arbitrarily choosing the nearest expiry, lowest strike call (productIndex 0)
    const expiryIndex = Exchange.getZetaGroup(asset).frontExpiryIndex; // [0,2)
    const productIndex = 0; // [0,23)
    const marketIndex =
      expiryIndex * constants.PRODUCTS_PER_EXPIRY + productIndex; // [0,46)
    market = Exchange.getMarketsByExpiryIndex(asset, expiryIndex)[productIndex];

    // Select the trade side for when we test place and cancel order
    side = types.Side.BID;

    console.log(`User: ${userKeypair.publicKey}`);
    console.log(`Zeta group account: ${Exchange.getZetaGroupAddress(asset)}`);
    console.log(`Margin account: ${client.getMarginAccountAddress(asset)}`);

    // Airdrop SOL
    const signature = await provider.connection.requestAirdrop(
      userKeypair.publicKey,
      SOL_AMOUNT * 10 ** 9 // lamports
    );
    await connection.confirmTransaction(signature);
  });

  it("Init margin account via CPI", async () => {
    // FYI can only init this once
    const tx = await program.methods
      .initializeMarginAccount()
      .accounts({
        zetaProgram: Exchange.programId,
        initializeMarginCpiAccounts: {
          zetaGroup: Exchange.getZetaGroupAddress(asset),
          marginAccount: client.getMarginAccountAddress(asset),
          authority: userKeypair.publicKey,
          payer: userKeypair.publicKey,
          zetaProgram: Exchange.programId,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Deposit USDC into margin account via CPI", async () => {
    let userUsdcAccount = await provider.connection.getAccountInfo(userUsdc);
    // Mint USDC if they don't have an acct
    if (userUsdcAccount == null) {
      console.info("USDC account doesn't exist, airdropping USDC");
      await mintUsdc(userKeypair.publicKey, USDC_AMOUNT);
    } else {
      console.info("USDC exists, proceeding");
    }
    userUsdcAccount = await provider.connection.getAccountInfo(userUsdc);
    assert.ok(userUsdcAccount !== undefined);

    // Deposit all newly minted USDC into the margin account
    let [greeks, _greeksNonce] = await utils.getGreeks(
      Exchange.programId,
      Exchange.getZetaGroupAddress(asset)
    );
    const tx = await program.methods
      .deposit(new anchor.BN(utils.convertDecimalToNativeInteger(USDC_AMOUNT)))
      .accounts({
        zetaProgram: Exchange.programId,
        depositCpiAccounts: {
          zetaGroup: Exchange.getZetaGroupAddress(asset),
          marginAccount: client.getMarginAccountAddress(asset),
          vault: Exchange.getVaultAddress(asset),
          userTokenAccount: userUsdc,
          socializedLossAccount:
            Exchange.getSocializedLossAccountAddress(asset),
          authority: userKeypair.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          state: Exchange.stateAddress,
          greeks,
        },
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Withdraw USDC out of margin account via CPI", async () => {
    let [greeks, _greeksNonce] = await utils.getGreeks(
      Exchange.programId,
      Exchange.getZetaGroupAddress(asset)
    );

    // Withdraw 10% of deposited funds
    const tx = await program.methods
      .withdraw(
        new anchor.BN(utils.convertDecimalToNativeInteger(0.1 * USDC_AMOUNT))
      )
      .accounts({
        zetaProgram: Exchange.programId,
        withdrawCpiAccounts: {
          state: Exchange.stateAddress,
          zetaGroup: Exchange.getZetaGroupAddress(asset),
          marginAccount: client.getMarginAccountAddress(asset),
          vault: Exchange.getVaultAddress(asset),
          userTokenAccount: userUsdc,
          authority: userKeypair.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          greeks,
          oracle: pythOracle,
          socializedLossAccount:
            Exchange.getSocializedLossAccountAddress(asset),
        },
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Initialize open orders account via CPI", async () => {
    [openOrders] = await utils.getOpenOrders(
      Exchange.programId,
      market.address,
      userKeypair.publicKey
    );

    [openOrdersMap] = await utils.getOpenOrdersMap(
      Exchange.programId,
      openOrders
    );

    const tx = await program.methods
      .initializeOpenOrders()
      .accounts({
        zetaProgram: Exchange.programId,
        initializeOpenOrdersCpiAccounts: {
          state: Exchange.stateAddress,
          zetaGroup: Exchange.getZetaGroupAddress(asset),
          dexProgram: constants.DEX_PID.devnet,
          systemProgram: anchor.web3.SystemProgram.programId,
          openOrders,
          marginAccount: client.getMarginAccountAddress(asset),
          authority: client.publicKey,
          payer: client.publicKey,
          market: market.address,
          serumAuthority: Exchange.serumAuthority,
          openOrdersMap,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
      })
      .rpc();

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

    let [greeks, _greeksNonce] = await utils.getGreeks(
      Exchange.programId,
      Exchange.getZetaGroupAddress(asset)
    );

    const tx = await program.methods
      .placeOrder(
        new anchor.BN(utils.convertDecimalToNativeInteger(1)),
        new anchor.BN(utils.convertDecimalToNativeLotSize(1)),
        types.toProgramSide(side),
        null
      )
      .accounts({
        zetaProgram: Exchange.programId,
        placeOrderCpiAccounts: {
          state: Exchange.stateAddress,
          zetaGroup: Exchange.getZetaGroupAddress(asset),
          marginAccount: client.getMarginAccountAddress(asset),
          authority: userKeypair.publicKey,
          dexProgram: constants.DEX_PID.devnet,
          tokenProgram: TOKEN_PROGRAM_ID,
          serumAuthority: Exchange.serumAuthority,
          greeks,
          openOrders: openOrders,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          marketAccounts: marketAccounts,
          oracle: pythOracle,
          marketNode: Exchange.getGreeks(asset).nodeKeys[market.marketIndex],
          marketMint:
            side == types.Side.BID
              ? market.serumMarket.quoteMintAddress
              : market.serumMarket.baseMintAddress,
          mintAuthority: Exchange.mintAuthority,
        },
      })
      .rpc();

    console.log("Your transaction signature", tx);
  });

  it("Cancel order via CPI", async () => {
    await client.updateState();
    let orders = client.getOrders(asset);
    if (orders.length === 0) {
      throw new Error("No relevant client order to cancel");
    }

    const cancelAccounts = {
      zetaGroup: Exchange.getZetaGroupAddress(asset),
      state: Exchange.stateAddress,
      marginAccount: client.getMarginAccountAddress(asset),
      dexProgram: constants.DEX_PID.devnet,
      serumAuthority: Exchange.serumAuthority,
      openOrders: openOrders,
      market: market.address,
      bids: market.serumMarket.decoded.bids,
      asks: market.serumMarket.decoded.asks,
      eventQueue: market.serumMarket.decoded.eventQueue,
    };

    const tx = await program.methods
      .cancelOrder(types.toProgramSide(side), orders[0].orderId)
      .accounts({
        zetaProgram: Exchange.programId,
        cancelOrderCpiAccounts: {
          authority: userKeypair.publicKey,
          cancelAccounts: cancelAccounts,
        },
      });

    console.log("Your transaction signature", tx);
  });

  it("Read Zeta data", async () => {
    let [greeks, _greeksNonce] = await utils.getGreeks(
      Exchange.programId,
      Exchange.getZetaGroupAddress(asset)
    );

    const tx = await program.rpc.readProgramData({
      accounts: {
        state: Exchange.stateAddress,
        zetaGroup: Exchange.getZetaGroupAddress(asset),
        marginAccount: client.getMarginAccountAddress(asset),
        greeks,
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
