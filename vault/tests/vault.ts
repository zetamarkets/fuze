require("dotenv").config({ path: __dirname + `/../.env` });
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Vault } from "../target/types/vault";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import assert from "assert";
import { sleep, IVaultBumps, IEpochTimes } from "./utils";
import {
  Exchange,
  Network,
  utils as zetaUtils,
  types,
  constants,
  Client,
} from "@zetamarkets/sdk";

// TODO:

const DECIMALS = 6;
const UNIX_WEEK: number = 604800; // unix time (seconds)

describe("vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Vault as Program<Vault>;
  const zetaProgram = new anchor.web3.PublicKey(process.env!.zeta_program);

  const vaultAuthority = anchor.web3.Keypair.generate();
  const userKeypair = anchor.web3.Keypair.generate();

  const pythOracle = constants.PYTH_PRICE_FEEDS[Network.DEVNET]["SOL/USD"];

  // These are all of the variables we assume exist in the world already and
  // are available to the client.
  let usdcMintAccount: Token;
  let usdcMint: anchor.web3.PublicKey;
  let vaultAuthorityUsdc: anchor.web3.PublicKey;

  let client: Client;

  it("Initializes the state-of-the-world", async () => {
    // Load Zeta SDK exchange object which has all the info one might need
    await Exchange.load(
      zetaProgram,
      Network.DEVNET,
      provider.connection,
      zetaUtils.defaultCommitment(),
      undefined,
      0
    );

    client = await Client.load(
      provider.connection,
      new anchor.Wallet(userKeypair),
      zetaUtils.defaultCommitment(),
      undefined,
      false
    );

    // Airdrop some SOL to the vault authority
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        vaultAuthority.publicKey,
        2_000_000_000 // 2 SOL
      ),
      "confirmed"
    );
    // Airdrop some SOL to the user
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        userKeypair.publicKey,
        0_100_000_000 // 2 SOL
      ),
      "confirmed"
    );

    usdcMintAccount = await Token.createMint(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      vaultAuthority.publicKey,
      null,
      DECIMALS,
      TOKEN_PROGRAM_ID
    );
    usdcMint = usdcMintAccount.publicKey;
    vaultAuthorityUsdc = await usdcMintAccount.createAssociatedTokenAccount(
      vaultAuthority.publicKey
    );
  });

  // These are all variables the client will need to create in order to
  // initialize the vault
  const vaultName = "sol_put_sell";

  let vaultAccount,
    vaultAccountBump,
    redeemableMint,
    redeemableMintAccount,
    redeemableMintBump,
    vaultUsdc,
    vaultUsdcBump,
    userRedeemable,
    secondUserRedeemable,
    bumps: IVaultBumps,
    epochTimes: IEpochTimes;

  it("Initializes the vault", async () => {
    [vaultAccount, vaultAccountBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(vaultName)],
        program.programId
      );

    [redeemableMint, redeemableMintBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(vaultName), Buffer.from("redeemable_mint")],
        program.programId
      );

    [vaultUsdc, vaultUsdcBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(vaultName), Buffer.from("vault_usdc")],
      program.programId
    );

    bumps = {
      vaultAccount: vaultAccountBump,
      redeemableMint: redeemableMintBump,
      vaultUsdc: vaultUsdcBump,
    };

    const nowBn = new anchor.BN(Date.now() / 1000);
    epochTimes = {
      startEpoch: nowBn.add(new anchor.BN(5)),
      endDeposits: nowBn.add(new anchor.BN(10)),
      startAuction: nowBn.add(new anchor.BN(12)),
      endAuction: nowBn.add(new anchor.BN(15)),
      startSettlement: nowBn.add(new anchor.BN(18)),
      endEpoch: nowBn.add(new anchor.BN(20)),
    };

    await program.rpc.initializeVault(vaultName, bumps, epochTimes, {
      accounts: {
        vaultAuthority: vaultAuthority.publicKey,
        vaultAccount,
        usdcMint,
        redeemableMint,
        vaultUsdc,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [vaultAuthority],
    });

    redeemableMintAccount = new Token(
      provider.connection,
      redeemableMint,
      TOKEN_PROGRAM_ID,
      vaultAuthority
    );

    // USDC in vault == 0
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.ok(vaultUsdcAccount.amount.eq(new anchor.BN(0)));
    // Redeemable tokens minted == 0
    let redeemableMintInfo = await redeemableMintAccount.getMintInfo();
    assert.ok(redeemableMintInfo.supply.eq(new anchor.BN(0)));
  });

  // We're going to need to start using the associated program account for creating token accounts
  // if not in testing, then definitely in production.

  let userUsdc;
  // 10 usdc
  const firstDeposit = new anchor.BN(10_000_000);

  it("Exchanges user USDC for redeemable tokens", async () => {
    // Wait until the vault has opened.
    if (Date.now() < epochTimes.startEpoch.toNumber() * 1000) {
      await sleep(epochTimes.startEpoch.toNumber() * 1000 - Date.now() + 2000);
    }

    userUsdc = await usdcMintAccount.createAssociatedTokenAccount(
      userKeypair.publicKey
    );

    // Mint USDC to user USDC wallet
    await usdcMintAccount.mintTo(
      userUsdc,
      vaultAuthority,
      [],
      firstDeposit.toNumber()
    );

    // Check if we inited correctly
    let userUsdcAccount = await usdcMintAccount.getAccountInfo(userUsdc);
    assert.ok(userUsdcAccount.amount.eq(firstDeposit));

    [userRedeemable] = await anchor.web3.PublicKey.findProgramAddress(
      [
        userKeypair.publicKey.toBuffer(),
        Buffer.from(vaultName),
        Buffer.from("user_redeemable"),
      ],
      program.programId
    );

    await program.rpc.exchangeUsdcForRedeemable(firstDeposit, {
      accounts: {
        userAuthority: userKeypair.publicKey,
        userUsdc,
        userRedeemable,
        vaultAccount,
        usdcMint,
        redeemableMint,
        vaultUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      instructions: [
        program.instruction.initUserRedeemable({
          accounts: {
            userAuthority: userKeypair.publicKey,
            userRedeemable,
            vaultAccount,
            redeemableMint,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
        }),
      ],
      signers: [userKeypair],
    });

    // Check that USDC is in vault and user has received their redeem tokens in return
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.ok(vaultUsdcAccount.amount.eq(firstDeposit));
    let userRedeemableAccount = await redeemableMintAccount.getAccountInfo(
      userRedeemable
    );
    assert.ok(userRedeemableAccount.amount.eq(firstDeposit));
  });

  // 420 usdc
  const secondDeposit = new anchor.BN(420_000_000);
  let totalvaultUsdc, secondUserKeypair, secondUserUsdc;

  it("Exchanges a second users USDC for redeemable tokens", async () => {
    secondUserKeypair = anchor.web3.Keypair.generate();

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        secondUserKeypair.publicKey,
        0_100_000_000 // 0.1 SOL
      ),
      "confirmed"
    );
    secondUserUsdc = await usdcMintAccount.createAssociatedTokenAccount(
      secondUserKeypair.publicKey
    );
    await usdcMintAccount.mintTo(
      secondUserUsdc,
      vaultAuthority,
      [],
      secondDeposit.toNumber()
    );

    // Checking the transfer went through
    let secondUserUsdcAccount = await usdcMintAccount.getAccountInfo(
      secondUserUsdc
    );
    assert.ok(secondUserUsdcAccount.amount.eq(secondDeposit));

    [secondUserRedeemable] = await anchor.web3.PublicKey.findProgramAddress(
      [
        secondUserKeypair.publicKey.toBuffer(),
        Buffer.from(vaultName),
        Buffer.from("user_redeemable"),
      ],
      program.programId
    );

    await program.rpc.exchangeUsdcForRedeemable(secondDeposit, {
      accounts: {
        userAuthority: secondUserKeypair.publicKey,
        userUsdc: secondUserUsdc,
        userRedeemable: secondUserRedeemable,
        vaultAccount,
        usdcMint,
        redeemableMint,
        vaultUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      instructions: [
        program.instruction.initUserRedeemable({
          accounts: {
            userAuthority: secondUserKeypair.publicKey,
            userRedeemable: secondUserRedeemable,
            vaultAccount,
            redeemableMint,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
        }),
      ],
      signers: [secondUserKeypair],
    });

    totalvaultUsdc = firstDeposit.add(secondDeposit);
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.ok(vaultUsdcAccount.amount.eq(totalvaultUsdc));
    let secondUserRedeemableAccount =
      await redeemableMintAccount.getAccountInfo(secondUserRedeemable);
    assert.ok(secondUserRedeemableAccount.amount.eq(secondDeposit));
  });

  function getClosestMarket(
    exchange: typeof Exchange, // TODO: change this to Market[] when sdk 0.8.3 released
    delta: number,
    expiry: number = UNIX_WEEK
  ) {
    // Find closest expiry
    let closestExpiry = exchange.markets.expirySeries.sort((a, b) => {
      return Math.abs(expiry - a.expiryTs) - Math.abs(expiry - b.expiryTs);
    })[0];

    // Find closest strike to 5-delta
    let closestPutDeltaIndex = exchange.greeks.productGreeks.reduce(
      (iMin, x, i, arr) =>
        Math.abs(delta - zetaUtils.convertNativeBNToDecimal(x.delta)) <
        Math.abs(delta - zetaUtils.convertNativeBNToDecimal(arr[iMin].delta))
          ? i
          : iMin,
      0
    );
    assert(
      closestPutDeltaIndex >= 0 &&
        closestPutDeltaIndex < constants.PRODUCTS_PER_EXPIRY
    );

    let market = Exchange.markets.getMarketsByExpiryIndex(
      closestExpiry.expiryIndex
    )[constants.PRODUCTS_PER_EXPIRY + closestPutDeltaIndex];

    console.log(
      `Closest market found: Expiry ${market.expirySeries.expiryTs}, Strike ${market.strike}, Kind ${market.kind}`
    );

    return market;
  }

  it("Place order via CPI", async () => {
    // Select instrument for vault to trade
    // For puposes of this put selling vault we choose the market closest to 1w expiry and 5-delta strike
    let market = getClosestMarket(Exchange, 5);
    // Determine sizing of trade
    // size = total_vault_usdc / K
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    let size = vaultUsdcAccount.amount
      .div(new anchor.BN(market.strike))
      .toNumber();
    // Price - arbitrary rn
    let price = 1;
    console.log(`${size}`);

    let [openOrdersAccount] = await zetaUtils.getOpenOrders(
      zetaProgram,
      market.address,
      userKeypair.publicKey
    );

    const marketAccounts = {
      market: market.address,
      requestQueue: market.serumMarket.decoded.requestQueue,
      eventQueue: market.serumMarket.decoded.eventQueue,
      bids: market.serumMarket.decoded.bids,
      asks: market.serumMarket.decoded.asks,
      coinVault: market.serumMarket.decoded.baseVault,
      pcVault: market.serumMarket.decoded.quoteVault,
      orderPayerTokenAccount: types.Side.ASK,
      coinWallet: market.baseVault,
      pcWallet: market.quoteVault,
    };

    const tx = await program.rpc.placeAuctionOrder(
      new anchor.BN(zetaUtils.convertDecimalToNativeInteger(price)),
      size,
      types.toProgramSide(types.Side.ASK),
      {
        accounts: {
          zetaProgram: zetaProgram,
          vaultAuthority: vaultAuthority.publicKey,
          vaultAccount: vaultAccount,
          usdcMint: usdcMint,
          placeOrderCpiAccounts: {
            state: Exchange.stateAddress,
            zetaGroup: Exchange.zetaGroupAddress,
            marginAccount: client.marginAccountAddress,
            authority: userKeypair.publicKey,
            dexProgram: constants.DEX_PID,
            tokenProgram: TOKEN_PROGRAM_ID,
            serumAuthority: Exchange.serumAuthority,
            greeks: Exchange.greeksAddress,
            openOrders: openOrdersAccount,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            marketAccounts: marketAccounts,
            oracle: pythOracle,
            marketNode: Exchange.greeks.nodeKeys[market.marketIndex],
          },
        },
      }
    );
    console.log("Your transaction signature", tx);
  });

  // Withdraw Phase

  const firstWithdrawal = new anchor.BN(2_000_000);

  it("Exchanges user Redeemable tokens for USDC", async () => {
    [escrowUsdc] = await anchor.web3.PublicKey.findProgramAddress(
      [
        userKeypair.publicKey.toBuffer(),
        Buffer.from(vaultName),
        Buffer.from("escrow_usdc"),
      ],
      program.programId
    );

    await program.rpc.exchangeRedeemableForUsdc(firstWithdrawal, {
      accounts: {
        userAuthority: userKeypair.publicKey,
        userUsdc,
        userRedeemable,
        vaultAccount,
        usdcMint,
        redeemableMint,
        vaultUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [userKeypair],
    });

    totalvaultUsdc = totalvaultUsdc.sub(firstWithdrawal);
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.ok(vaultUsdcAccount.amount.eq(totalvaultUsdc));
    let userUsdcAccount = await usdcMintAccount.getAccountInfo(userUsdc);
    assert.ok(userUsdcAccount.amount.eq(firstWithdrawal));
  });

  // it("Exchanges user Redeemable tokens for watermelon", async () => {
  //   // Wait until the vault has ended.
  //   if (Date.now() < epochTimes.endvault.toNumber() * 1000) {
  //     await sleep(epochTimes.endvault.toNumber() * 1000 - Date.now() + 3000);
  //   }

  //   let firstUserRedeemable = firstDeposit.sub(firstWithdrawal);
  //   let userWatermelon =
  //     await watermelonMintAccount.createAssociatedTokenAccount(
  //       userKeypair.publicKey
  //     );

  //   await program.rpc.exchangeRedeemableForWatermelon(firstUserRedeemable, {
  //     accounts: {
  //       payer: provider.wallet.publicKey,
  //       userAuthority: userKeypair.publicKey,
  //       userWatermelon,
  //       userRedeemable,
  //       vaultAccount,
  //       watermelonMint,
  //       redeemableMint,
  //       vaultWatermelon,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //     // signers: [userKeypair],
  //   });

  //   let vaultWatermelonAccount = await watermelonMintAccount.getAccountInfo(
  //     vaultWatermelon
  //   );
  //   let redeemedWatermelon = firstUserRedeemable
  //     .mul(watermelonvaultAmount)
  //     .div(totalvaultUsdc);
  //   let remainingWatermelon = watermelonvaultAmount.sub(redeemedWatermelon);
  //   assert.ok(vaultWatermelonAccount.amount.eq(remainingWatermelon));
  //   let userWatermelonAccount = await watermelonMintAccount.getAccountInfo(
  //     userWatermelon
  //   );
  //   assert.ok(userWatermelonAccount.amount.eq(redeemedWatermelon));
  // });

  // it("Exchanges second user's Redeemable tokens for watermelon", async () => {
  //   let secondUserWatermelon =
  //     await watermelonMintAccount.createAssociatedTokenAccount(
  //       secondUserKeypair.publicKey
  //     );

  //   await program.rpc.exchangeRedeemableForWatermelon(secondDeposit, {
  //     accounts: {
  //       payer: provider.wallet.publicKey,
  //       userAuthority: secondUserKeypair.publicKey,
  //       userWatermelon: secondUserWatermelon,
  //       userRedeemable: secondUserRedeemable,
  //       vaultAccount,
  //       watermelonMint,
  //       redeemableMint,
  //       vaultWatermelon,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //     // signers: []
  //   });

  //   let vaultWatermelonAccount = await watermelonMintAccount.getAccountInfo(
  //     vaultWatermelon
  //   );
  //   assert.ok(vaultWatermelonAccount.amount.eq(new anchor.BN(0)));
  // });

  // it("Withdraws total USDC from vault account", async () => {
  //   await program.rpc.withdrawvaultUsdc({
  //     accounts: {
  //       vaultAuthority: vaultAuthority.publicKey,
  //       vaultAuthorityUsdc,
  //       vaultAccount,
  //       usdcMint,
  //       watermelonMint,
  //       vaultUsdc,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //     signers: [vaultAuthority],
  //   });

  //   let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
  //   assert.ok(vaultUsdcAccount.amount.eq(new anchor.BN(0)));
  //   let vaultAuthorityUsdcAccount = await usdcMintAccount.getAccountInfo(
  //     vaultAuthorityUsdc
  //   );
  //   assert.ok(vaultAuthorityUsdcAccount.amount.eq(totalvaultUsdc));
  // });

  // it("Withdraws USDC from the escrow account after waiting period is over", async () => {
  //   // Wait until the escrow period is over.
  //   if (Date.now() < epochTimes.endEscrow.toNumber() * 1000 + 1000) {
  //     await sleep(epochTimes.endEscrow.toNumber() * 1000 - Date.now() + 4000);
  //   }

  //   await program.rpc.withdrawFromEscrow(firstWithdrawal, {
  //     accounts: {
  //       payer: provider.wallet.publicKey,
  //       userAuthority: userKeypair.publicKey,
  //       userUsdc,
  //       escrowUsdc,
  //       vaultAccount,
  //       usdcMint,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //   });

  //   let userUsdcAccount = await usdcMintAccount.getAccountInfo(userUsdc);
  //   assert.ok(userUsdcAccount.amount.eq(firstWithdrawal));
  // });

  // Closes the account subscriptions so the test won't hang.
  it("BOILERPLATE: Close websockets.", async () => {
    await Exchange.close();
    await client.close();
  });
});
