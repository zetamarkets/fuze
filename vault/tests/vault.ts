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
} from "@zetamarkets/sdk";
import { mintUsdc, getClosestMarket } from "./utils";

describe("vault", () => {
  const vaultAuthority = anchor.web3.Keypair.generate();
  const userKeypair = anchor.web3.Keypair.generate();
  console.log(vaultAuthority.publicKey.toString());
  console.log(userKeypair.publicKey.toString());

  // Configure the client to use the local cluster.
  const url = "https://api.devnet.solana.com";
  if (url === undefined) {
    throw new Error("ANCHOR_PROVIDER_URL is not defined");
  }
  const connection = new anchor.web3.Connection(
    url,
    zetaUtils.defaultCommitment()
  );
  const provider = new anchor.Provider(
    connection,
    new anchor.Wallet(userKeypair),
    zetaUtils.defaultCommitment()
  );
  anchor.setProvider(provider);
  const publicConnection = new anchor.web3.Connection(
    "https://api.devnet.solana.com",
    zetaUtils.defaultCommitment()
  );

  const program = anchor.workspace.Vault as Program<Vault>;
  const zetaProgram = new anchor.web3.PublicKey(process.env!.zeta_program);

  const pythOracle = constants.PYTH_PRICE_FEEDS[Network.DEVNET]["SOL/USD"];

  // These are all of the variables we assume exist in the world already and
  // are available to the client.
  let usdcMintAccount: Token;
  let usdcMint: anchor.web3.PublicKey;
  let vaultAuthorityUsdc: anchor.web3.PublicKey;
  let vaultMargin;

  it("Initializes the state of the world", async () => {
    // Load Zeta SDK exchange object which has all the info one might need
    await Exchange.load(
      zetaProgram,
      Network.DEVNET,
      provider.connection,
      zetaUtils.defaultCommitment(),
      undefined,
      0
    );

    // Airdrop some SOL to the vault authority
    await publicConnection.confirmTransaction(
      await publicConnection.requestAirdrop(
        vaultAuthority.publicKey,
        1.0 * anchor.web3.LAMPORTS_PER_SOL // 1 SOL
      ),
      "confirmed"
    );
    console.log("Airdrop completed");

    const transferTransaction = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: vaultAuthority.publicKey,
        lamports: 0.1 * anchor.web3.LAMPORTS_PER_SOL, // 0.1 SOL
        toPubkey: userKeypair.publicKey,
      })
    );
    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      transferTransaction,
      [vaultAuthority]
    );

    usdcMint = await zetaUtils.getTokenMint(
      provider.connection,
      Exchange.vaultAddress
    );
    usdcMintAccount = new Token(
      provider.connection,
      usdcMint,
      TOKEN_PROGRAM_ID,
      userKeypair // TODO: not sure this is the way to go?
    );
    vaultAuthorityUsdc = await usdcMintAccount.createAssociatedTokenAccount(
      vaultAuthority.publicKey
    );
  });

  // These are all variables the client will need to create in order to
  // initialize the vault
  // TODO: remove this - for purposes of creating unique testing vaults
  const vaultName = "test_vault_" + Math.random().toString(16).substring(2, 8); // "sol_put_sell";
  console.log(`Vault name: ${vaultName}`);

  let vault: anchor.web3.PublicKey,
    vaultBump,
    vaultPayer,
    vaultPayerBump,
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
    [vault, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
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

    [vaultPayer, vaultPayerBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(vaultName), Buffer.from("payer")],
        program.programId
      );

    let vaultLamports = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 0.1);

    bumps = {
      vault: vaultBump,
      vaultPayer: vaultPayerBump,
      redeemableMint: redeemableMintBump,
      vaultUsdc: vaultUsdcBump,
    };

    const nowBn = new anchor.BN(Date.now() / 1000);
    epochTimes = {
      startEpoch: nowBn.add(new anchor.BN(4)),
      endDeposits: nowBn.add(new anchor.BN(18)),
      startAuction: nowBn.add(new anchor.BN(20)),
      endAuction: nowBn.add(new anchor.BN(22)),
      startSettlement: nowBn.add(new anchor.BN(24)),
      endEpoch: nowBn.add(new anchor.BN(25)),
    };

    await program.rpc.initializeVault(
      vaultName,
      vaultLamports,
      bumps,
      epochTimes,
      {
        accounts: {
          vaultAuthority: vaultAuthority.publicKey,
          vault,
          vaultPayer,
          usdcMint,
          redeemableMint,
          vaultUsdc,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [vaultAuthority],
      }
    );

    redeemableMintAccount = new Token(
      provider.connection,
      redeemableMint,
      TOKEN_PROGRAM_ID,
      vaultAuthority
    );

    // SOL balance for vault payer PDA is `vaultLamports`
    let vaultPayerAccount = await connection.getAccountInfo(vaultPayer);
    assert.ok(vaultPayerAccount.lamports == vaultLamports.toNumber());
    // USDC in vault == 0
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.ok(vaultUsdcAccount.amount.eq(new anchor.BN(0)));
    // Redeemable tokens minted == 0
    let redeemableMintInfo = await redeemableMintAccount.getMintInfo();
    assert.ok(redeemableMintInfo.supply.eq(new anchor.BN(0)));
  });

  it("Init Zeta margin for vault account via CPI", async () => {
    [vaultMargin] = await zetaUtils.getMarginAccount(
      Exchange.programId,
      Exchange.zetaGroupAddress,
      vaultPayer
    );

    // TODO: temporary workaround, will need to avoid PDA authority until issue #350 is fixed
    const tx = await program.rpc.initializeZetaMarginAccount({
      accounts: {
        zetaProgram: Exchange.programId,
        vaultAuthority: vaultAuthority.publicKey,
        vault,
        usdcMint,
        initializeMarginCpiAccounts: {
          marginAccount: vaultMargin,
          authority: vaultPayer,
          zetaProgram: Exchange.programId,
          systemProgram: anchor.web3.SystemProgram.programId,
          zetaGroup: Exchange.zetaGroupAddress,
        },
      },
      signers: [vaultAuthority],
    });
    console.log("Your transaction signature", tx);
  });

  let userUsdc;
  const firstDeposit = 40;

  it("Exchanges user USDC for redeemable tokens", async () => {
    // Wait until the vault has opened.
    if (Date.now() < epochTimes.startEpoch.toNumber() * 1000) {
      await sleep(epochTimes.startEpoch.toNumber() * 1000 - Date.now() + 2000);
    }

    userUsdc = await usdcMintAccount.createAssociatedTokenAccount(
      userKeypair.publicKey
    );

    // Mint USDC to user USDC wallet
    console.log("Minting USDC to User 1");
    await mintUsdc(userKeypair.publicKey, firstDeposit);

    // Check if we inited correctly
    let userUsdcAccount = await usdcMintAccount.getAccountInfo(userUsdc);

    assert.equal(
      zetaUtils.convertNativeBNToDecimal(userUsdcAccount.amount),
      firstDeposit
    );

    [userRedeemable] = await anchor.web3.PublicKey.findProgramAddress(
      [
        userKeypair.publicKey.toBuffer(),
        Buffer.from(vaultName),
        Buffer.from("user_redeemable"),
      ],
      program.programId
    );

    await program.rpc.exchangeUsdcForRedeemable(
      new anchor.BN(zetaUtils.convertDecimalToNativeInteger(firstDeposit)),
      {
        accounts: {
          userAuthority: userKeypair.publicKey,
          userUsdc,
          userRedeemable,
          vault,
          vaultPayer,
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
              vault,
              vaultPayer,
              redeemableMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            },
          }),
        ],
        signers: [userKeypair],
      }
    );

    // Check that USDC is in vault and user has received their redeem tokens in return
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(vaultUsdcAccount.amount),
      firstDeposit
    );
    let userRedeemableAccount = await redeemableMintAccount.getAccountInfo(
      userRedeemable
    );
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(userRedeemableAccount.amount),
      firstDeposit
    );
  });

  const secondDeposit = 420;
  let totalVaultUsdc, secondUserKeypair, secondUserUsdc;

  it("Exchanges a second users USDC for redeemable tokens", async () => {
    secondUserKeypair = anchor.web3.Keypair.generate();

    const transferTransaction = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: vaultAuthority.publicKey,
        lamports: 0.1 * anchor.web3.LAMPORTS_PER_SOL, // 0.1 SOL
        toPubkey: secondUserKeypair.publicKey,
      })
    );
    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      transferTransaction,
      [vaultAuthority]
    );
    secondUserUsdc = await usdcMintAccount.createAssociatedTokenAccount(
      secondUserKeypair.publicKey
    );
    console.log("Minting USDC to User 2");
    await mintUsdc(secondUserKeypair.publicKey, secondDeposit);

    // Checking the transfer went through
    let secondUserUsdcAccount = await usdcMintAccount.getAccountInfo(
      secondUserUsdc
    );
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(secondUserUsdcAccount.amount),
      secondDeposit
    );

    [secondUserRedeemable] = await anchor.web3.PublicKey.findProgramAddress(
      [
        secondUserKeypair.publicKey.toBuffer(),
        Buffer.from(vaultName),
        Buffer.from("user_redeemable"),
      ],
      program.programId
    );

    await program.rpc.exchangeUsdcForRedeemable(
      new anchor.BN(zetaUtils.convertDecimalToNativeInteger(secondDeposit)),
      {
        accounts: {
          userAuthority: secondUserKeypair.publicKey,
          userUsdc: secondUserUsdc,
          userRedeemable: secondUserRedeemable,
          vault,
          vaultPayer,
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
              vault,
              vaultPayer,
              redeemableMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            },
          }),
        ],
        signers: [secondUserKeypair],
      }
    );

    totalVaultUsdc = firstDeposit + secondDeposit;
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(vaultUsdcAccount.amount),
      totalVaultUsdc
    );
    let secondUserRedeemableAccount =
      await redeemableMintAccount.getAccountInfo(secondUserRedeemable);
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(secondUserRedeemableAccount.amount),
      secondDeposit
    );
  });

  it("Deposit USDC into vault's Zeta margin account via CPI", async () => {
    // Deposit all vault USDC into its Zeta margin acct
    const tx = await program.rpc.depositZeta(
      new anchor.BN(zetaUtils.convertDecimalToNativeInteger(totalVaultUsdc)),
      {
        accounts: {
          zetaProgram: Exchange.programId,
          vaultAuthority: vaultAuthority.publicKey,
          vault,
          usdcMint,
          depositCpiAccounts: {
            zetaGroup: Exchange.zetaGroupAddress,
            marginAccount: vaultMargin,
            vault: Exchange.vaultAddress,
            userTokenAccount: vaultUsdc,
            socializedLossAccount: Exchange.socializedLossAccountAddress,
            authority: vaultPayer,
            tokenProgram: TOKEN_PROGRAM_ID,
            state: Exchange.stateAddress,
            greeks: Exchange.greeksAddress,
          },
        },
        signers: [vaultAuthority],
      }
    );
    console.log("Your transaction signature", tx);
    let vaultMarginAccount = await Exchange.program.account.marginAccount.fetch(
      vaultMargin
    );
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(vaultMarginAccount.balance),
      totalVaultUsdc
    );
  });

  let market, openOrders, openOrdersMap;

  it("Initialize open orders account via CPI", async () => {
    // Select instrument for vault to trade
    // For puposes of this put selling vault we choose the market closest to 1w expiry and 5-delta strike
    market = getClosestMarket(Exchange, 0.05);

    [openOrders] = await zetaUtils.getOpenOrders(
      Exchange.programId,
      market.address,
      vaultPayer
    );

    [openOrdersMap] = await zetaUtils.getOpenOrdersMap(
      Exchange.programId,
      openOrders
    );

    const tx = await program.rpc.initializeZetaOpenOrders({
      accounts: {
        zetaProgram: Exchange.programId,
        vaultAuthority: vaultAuthority.publicKey,
        vault,
        usdcMint,
        initializeOpenOrdersCpiAccounts: {
          state: Exchange.stateAddress,
          zetaGroup: Exchange.zetaGroupAddress,
          dexProgram: constants.DEX_PID,
          systemProgram: anchor.web3.SystemProgram.programId,
          openOrders: openOrders,
          marginAccount: vaultMargin,
          authority: vaultPayer,
          market: market.address,
          serumAuthority: Exchange.serumAuthority,
          openOrdersMap: openOrdersMap,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
      },
      signers: [vaultAuthority],
    });
    console.log("Your transaction signature", tx);
  });

  // params of the auction
  let price, size, side;

  it("Place auction put sell order on Zeta DEX", async () => {
    // Wait until the vault has opened.
    if (Date.now() < epochTimes.startAuction.toNumber() * 1000) {
      await sleep(
        epochTimes.startAuction.toNumber() * 1000 - Date.now() + 2000
      );
    }
    // Determine sizing of trade
    // size = total_vault_usdc / K
    let vaultMarginAccount = await Exchange.program.account.marginAccount.fetch(
      vaultMargin
    );
    let strike = new anchor.BN(market.strike);
    let scale = new anchor.BN(Math.pow(10, constants.PLATFORM_PRECISION));
    size = vaultMarginAccount.balance.div(strike).div(scale).toNumber();

    // Price - arbitrary rn basically don't want to get traded against
    price = 1000;
    side = types.Side.ASK;
    console.log(`Selling Put on Zeta at \$${price}x${size}`);
    assert.ok(size > 0);

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

    const tx = await program.rpc.placeAuctionOrder(
      new anchor.BN(zetaUtils.convertDecimalToNativeInteger(price)),
      new anchor.BN(zetaUtils.convertDecimalToNativeLotSize(size)),
      types.toProgramSide(side),
      null,
      {
        accounts: {
          zetaProgram: Exchange.programId,
          vaultAuthority: vaultAuthority.publicKey,
          vault,
          usdcMint,
          placeOrderCpiAccounts: {
            state: Exchange.stateAddress,
            zetaGroup: Exchange.zetaGroupAddress,
            marginAccount: vaultMargin,
            authority: vaultPayer,
            dexProgram: constants.DEX_PID,
            tokenProgram: TOKEN_PROGRAM_ID,
            serumAuthority: Exchange.serumAuthority,
            greeks: Exchange.greeksAddress,
            openOrders: openOrders,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            marketAccounts: marketAccounts,
            oracle: pythOracle,
            marketNode: Exchange.greeks.nodeKeys[market.marketIndex],
            marketMint:
              side == types.Side.BID
                ? market.serumMarket.quoteMintAddress
                : market.serumMarket.baseMintAddress,
            mintAuthority: Exchange.mintAuthority,
          },
        },
        signers: [vaultAuthority],
      }
    );
    console.log("Your transaction signature", tx);
  });

  // End auction + settlement
  // TODO: MM buys on auction

  it("Cancel auction on Zeta DEX (in place of taker + settlement)", async () => {
    // Wait until the vault has opened.
    if (Date.now() < epochTimes.startSettlement.toNumber() * 1000) {
      await sleep(
        epochTimes.startSettlement.toNumber() * 1000 - Date.now() + 2000
      );
    }

    await market.updateOrderbook();
    let orders = market.getOrdersForAccount(openOrders);
    assert(orders.length > 0);

    const cancelAccounts = {
      zetaGroup: Exchange.zetaGroupAddress,
      state: Exchange.stateAddress,
      marginAccount: vaultMargin,
      dexProgram: constants.DEX_PID,
      serumAuthority: Exchange.serumAuthority,
      openOrders: openOrders,
      market: market.address,
      bids: market.serumMarket.decoded.bids,
      asks: market.serumMarket.decoded.asks,
      eventQueue: market.serumMarket.decoded.eventQueue,
    };

    const tx = await program.rpc.cancelAuctionOrder(
      types.toProgramSide(side),
      orders[0].orderId,
      {
        accounts: {
          zetaProgram: Exchange.programId,
          vaultAuthority: vaultAuthority.publicKey,
          vault,
          usdcMint,
          cancelOrderCpiAccounts: {
            authority: vaultPayer,
            cancelAccounts: cancelAccounts,
          },
        },
        signers: [vaultAuthority],
      }
    );
    console.log("Your transaction signature", tx);
  });

  it("Withdraw USDC into vault's Zeta margin account via CPI", async () => {
    let vaultMarginAccount = await Exchange.program.account.marginAccount.fetch(
      vaultMargin
    );
    // Withdraw all vault USDC from its Zeta margin acct
    const tx = await program.rpc.withdrawZeta(
      new anchor.BN(vaultMarginAccount.balance),
      {
        accounts: {
          zetaProgram: Exchange.programId,
          vaultAuthority: vaultAuthority.publicKey,
          vault,
          usdcMint,
          withdrawCpiAccounts: {
            state: Exchange.stateAddress,
            zetaGroup: Exchange.zetaGroupAddress,
            vault: Exchange.vaultAddress,
            marginAccount: vaultMargin,
            userTokenAccount: vaultUsdc,
            tokenProgram: TOKEN_PROGRAM_ID,
            authority: vaultPayer,
            greeks: Exchange.greeksAddress,
            oracle: pythOracle,
            socializedLossAccount: Exchange.socializedLossAccountAddress,
          },
        },
        signers: [vaultAuthority],
      }
    );
    console.log("Your transaction signature", tx);
    vaultMarginAccount = await Exchange.program.account.marginAccount.fetch(
      vaultMargin
    );
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(vaultMarginAccount.balance),
      0
    );
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(vaultUsdcAccount.amount),
      totalVaultUsdc
    );
  });

  // Epoch end

  it("Roll over vault into next epoch", async () => {
    if (Date.now() < epochTimes.endEpoch.toNumber() * 1000) {
      await sleep(epochTimes.endEpoch.toNumber() * 1000 - Date.now() + 3000);
    }
    const nowBn = new anchor.BN(Date.now() / 1000);
    epochTimes = {
      startEpoch: nowBn.add(new anchor.BN(4)),
      endDeposits: nowBn.add(new anchor.BN(18)),
      startAuction: nowBn.add(new anchor.BN(20)),
      endAuction: nowBn.add(new anchor.BN(22)),
      startSettlement: nowBn.add(new anchor.BN(24)),
      endEpoch: nowBn.add(new anchor.BN(25)),
    };

    await program.rpc.rolloverVault(vaultName, bumps, epochTimes, {
      accounts: {
        vaultAuthority: vaultAuthority.publicKey,
        vault,
      },
      signers: [vaultAuthority],
    });

    let vaultAccount = await program.account.vault.fetch(vault);
    assert.deepEqual(vaultAccount.epochTimes, epochTimes);
  });

  // Withdraw Phase

  const firstWithdrawal = 2;

  it("Exchanges user Redeemable tokens for USDC", async () => {
    if (Date.now() < epochTimes.startEpoch.toNumber() * 1000) {
      await sleep(epochTimes.startEpoch.toNumber() * 1000 - Date.now() + 3000);
    }
    await program.rpc.exchangeRedeemableForUsdc(
      new anchor.BN(zetaUtils.convertDecimalToNativeInteger(firstWithdrawal)),
      {
        accounts: {
          userAuthority: userKeypair.publicKey,
          userUsdc,
          userRedeemable,
          vault,
          vaultPayer,
          usdcMint,
          redeemableMint,
          vaultUsdc,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [userKeypair],
      }
    );

    totalVaultUsdc = totalVaultUsdc - firstWithdrawal;
    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(vaultUsdcAccount.amount),
      totalVaultUsdc
    );
    let userUsdcAccount = await usdcMintAccount.getAccountInfo(userUsdc);
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(userUsdcAccount.amount),
      firstWithdrawal
    );
  });

  it("Withdraws total USDC from vault account", async () => {
    await program.rpc.withdrawVaultUsdc({
      accounts: {
        vaultAuthority: vaultAuthority.publicKey,
        vaultAuthorityUsdc,
        vault,
        vaultPayer,
        usdcMint,
        vaultUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [vaultAuthority],
    });

    let vaultUsdcAccount = await usdcMintAccount.getAccountInfo(vaultUsdc);
    assert.equal(
      zetaUtils.convertNativeBNToDecimal(vaultUsdcAccount.amount),
      0
    );
    let vaultAuthorityUsdcAccount = await usdcMintAccount.getAccountInfo(
      vaultAuthorityUsdc
    );
    assert.ok(
      zetaUtils.convertNativeBNToDecimal(vaultAuthorityUsdcAccount.amount),
      totalVaultUsdc
    );
  });

  // Closes the account subscriptions so the test won't hang.
  it("BOILERPLATE: Close websockets.", async () => {
    await Exchange.close();
  });
});
