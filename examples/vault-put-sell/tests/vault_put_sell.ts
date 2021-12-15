import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { VaultPutSell } from "../target/types/vault_put_sell";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import assert from "assert";
import { sleep, IVaultBumps, IEpochTimes } from "./utils";

// TODO: 
// [] for our purposes can sack "watermelon"
// [] remove permissionless redeem for watermelon

const DECIMALS = 6;

describe("vault-put-sell", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.VaultPutSell as Program<VaultPutSell>;

  const vaultAuthority = anchor.web3.Keypair.generate();
  const userKeypair = anchor.web3.Keypair.generate();

  // All mints default to 6 decimal places.
  const watermelonIdoAmount = new anchor.BN(5000000);

  // These are all of the variables we assume exist in the world already and
  // are available to the client.
  let usdcMintAccount: Token;
  let usdcMint: anchor.web3.PublicKey;
  let watermelonMintAccount: Token;
  let watermelonMint: anchor.web3.PublicKey;
  let idoAuthorityUsdc: anchor.web3.PublicKey;
  let idoAuthorityWatermelon: anchor.web3.PublicKey;

  it("Initializes the state-of-the-world", async () => {
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
    watermelonMintAccount = await Token.createMint(
      provider.connection,
      (provider.wallet as anchor.Wallet).payer,
      vaultAuthority.publicKey,
      null,
      DECIMALS,
      TOKEN_PROGRAM_ID
    );
    usdcMint = usdcMintAccount.publicKey;
    watermelonMint = watermelonMintAccount.publicKey;
    idoAuthorityUsdc = await usdcMintAccount.createAssociatedTokenAccount(
      vaultAuthority.publicKey
    );
    idoAuthorityWatermelon =
      await watermelonMintAccount.createAssociatedTokenAccount(
        vaultAuthority.publicKey
      );
    // Mint Watermelon tokens that will be distributed from the IDO pool.
    await watermelonMintAccount.mintTo(
      idoAuthorityWatermelon,
      vaultAuthority,
      [],
      watermelonIdoAmount.toNumber()
    );
    let idoAuthorityWatermelonAccount =
      await watermelonMintAccount.getAccountInfo(idoAuthorityWatermelon);

    assert.ok(idoAuthorityWatermelonAccount.amount.eq(watermelonIdoAmount));
  });

  // These are all variables the client will need to create in order to
  // initialize the IDO pool
  const idoName = "test_ido";

  let idoAccount,
    idoAccountBump,
    redeemableMint,
    redeemableMintAccount,
    redeemableMintBump,
    poolWatermelon,
    poolWatermelonBump,
    poolUsdc,
    poolUsdcBump,
    userRedeemable,
    escrowUsdc,
    secondUserRedeemable,
    bumps: IVaultBumps,
    epochTimes: IEpochTimes;

  it("Initializes the IDO pool", async () => {
    [idoAccount, idoAccountBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(idoName)],
        program.programId
      );

    [redeemableMint, redeemableMintBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(idoName), Buffer.from("redeemable_mint")],
        program.programId
      );

    [poolWatermelon, poolWatermelonBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(idoName), Buffer.from("pool_watermelon")],
        program.programId
      );

    [poolUsdc, poolUsdcBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName), Buffer.from("pool_usdc")],
      program.programId
    );

    bumps = {
      idoAccount: idoAccountBump,
      redeemableMint: redeemableMintBump,
      poolWatermelon: poolWatermelonBump,
      poolUsdc: poolUsdcBump,
    };

    const nowBn = new anchor.BN(Date.now() / 1000);
    epochTimes = {
      startIdo: nowBn.add(new anchor.BN(5)),
      endDeposits: nowBn.add(new anchor.BN(10)),
      endIdo: nowBn.add(new anchor.BN(15)),
      endEscrow: nowBn.add(new anchor.BN(16)),
    };

    await program.rpc.initializePool(
      idoName,
      bumps,
      watermelonIdoAmount,
      epochTimes,
      {
        accounts: {
          idoAuthority: vaultAuthority.publicKey,
          idoAuthorityWatermelon,
          idoAccount,
          watermelonMint,
          usdcMint,
          redeemableMint,
          poolWatermelon,
          poolUsdc,
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

    let idoAuthorityWatermelonAccount =
      await watermelonMintAccount.getAccountInfo(idoAuthorityWatermelon);
    assert.ok(idoAuthorityWatermelonAccount.amount.eq(new anchor.BN(0)));
  });

  // We're going to need to start using the associated program account for creating token accounts
  // if not in testing, then definitely in production.

  let userUsdc;
  // 10 usdc
  const firstDeposit = new anchor.BN(10_000_000);

  it("Exchanges user USDC for redeemable tokens", async () => {
    // Wait until the IDO has opened.
    if (Date.now() < epochTimes.startIdo.toNumber() * 1000) {
      await sleep(epochTimes.startIdo.toNumber() * 1000 - Date.now() + 2000);
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
        Buffer.from(idoName),
        Buffer.from("user_redeemable"),
      ],
      program.programId
    );

    try {
      await program.rpc.exchangeUsdcForRedeemable(firstDeposit, {
        accounts: {
          userAuthority: userKeypair.publicKey,
          userUsdc,
          userRedeemable,
          idoAccount,
          usdcMint,
          redeemableMint,
          poolUsdc,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
          program.instruction.initUserRedeemable({
            accounts: {
              userAuthority: userKeypair.publicKey,
              userRedeemable,
              idoAccount,
              redeemableMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            },
          }),
        ],
        signers: [userKeypair],
      });
    } catch (err) {
      console.log("This is the error message", err.toString());
    }
    let poolUsdcAccount = await usdcMintAccount.getAccountInfo(poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(firstDeposit));
    let userRedeemableAccount = await redeemableMintAccount.getAccountInfo(
      userRedeemable
    );
    assert.ok(userRedeemableAccount.amount.eq(firstDeposit));
  });

  // 420 usdc
  const secondDeposit = new anchor.BN(420_000_000);
  let totalPoolUsdc, secondUserKeypair, secondUserUsdc;

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
        Buffer.from(idoName),
        Buffer.from("user_redeemable"),
      ],
      program.programId
    );

    await program.rpc.exchangeUsdcForRedeemable(secondDeposit, {
      accounts: {
        userAuthority: secondUserKeypair.publicKey,
        userUsdc: secondUserUsdc,
        userRedeemable: secondUserRedeemable,
        idoAccount,
        usdcMint,
        redeemableMint,
        poolUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      instructions: [
        program.instruction.initUserRedeemable({
          accounts: {
            userAuthority: secondUserKeypair.publicKey,
            userRedeemable: secondUserRedeemable,
            idoAccount,
            redeemableMint,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
        }),
      ],
      signers: [secondUserKeypair],
    });

    let secondUserRedeemableAccount =
      await redeemableMintAccount.getAccountInfo(secondUserRedeemable);
    assert.ok(secondUserRedeemableAccount.amount.eq(secondDeposit));

    totalPoolUsdc = firstDeposit.add(secondDeposit);
    let poolUsdcAccount = await usdcMintAccount.getAccountInfo(poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(totalPoolUsdc));
  });

  const firstWithdrawal = new anchor.BN(2_000_000);

  it("Exchanges user Redeemable tokens for USDC", async () => {
    [escrowUsdc] = await anchor.web3.PublicKey.findProgramAddress(
      [
        userKeypair.publicKey.toBuffer(),
        Buffer.from(idoName),
        Buffer.from("escrow_usdc"),
      ],
      program.programId
    );

    await program.rpc.exchangeRedeemableForUsdc(firstWithdrawal, {
      accounts: {
        userAuthority: userKeypair.publicKey,
        escrowUsdc,
        userRedeemable,
        idoAccount,
        usdcMint,
        redeemableMint,
        watermelonMint,
        poolUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      instructions: [
        program.instruction.initEscrowUsdc({
          accounts: {
            userAuthority: userKeypair.publicKey,
            escrowUsdc,
            idoAccount,
            usdcMint,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
        }),
      ],
      signers: [userKeypair],
    });

    totalPoolUsdc = totalPoolUsdc.sub(firstWithdrawal);
    let poolUsdcAccount = await usdcMintAccount.getAccountInfo(poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(totalPoolUsdc));
    let escrowUsdcAccount = await usdcMintAccount.getAccountInfo(escrowUsdc);
    assert.ok(escrowUsdcAccount.amount.eq(firstWithdrawal));
  });

  it("Exchanges user Redeemable tokens for watermelon", async () => {
    // Wait until the IDO has ended.
    if (Date.now() < epochTimes.endIdo.toNumber() * 1000) {
      await sleep(epochTimes.endIdo.toNumber() * 1000 - Date.now() + 3000);
    }

    let firstUserRedeemable = firstDeposit.sub(firstWithdrawal);
    let userWatermelon =
      await watermelonMintAccount.createAssociatedTokenAccount(
        userKeypair.publicKey
      );

    await program.rpc.exchangeRedeemableForWatermelon(firstUserRedeemable, {
      accounts: {
        payer: provider.wallet.publicKey,
        userAuthority: userKeypair.publicKey,
        userWatermelon,
        userRedeemable,
        idoAccount,
        watermelonMint,
        redeemableMint,
        poolWatermelon,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      // signers: [userKeypair],
    });

    let poolWatermelonAccount = await watermelonMintAccount.getAccountInfo(
      poolWatermelon
    );
    let redeemedWatermelon = firstUserRedeemable
      .mul(watermelonIdoAmount)
      .div(totalPoolUsdc);
    let remainingWatermelon = watermelonIdoAmount.sub(redeemedWatermelon);
    assert.ok(poolWatermelonAccount.amount.eq(remainingWatermelon));
    let userWatermelonAccount = await watermelonMintAccount.getAccountInfo(
      userWatermelon
    );
    assert.ok(userWatermelonAccount.amount.eq(redeemedWatermelon));
  });

  it("Exchanges second user's Redeemable tokens for watermelon", async () => {
    let secondUserWatermelon =
      await watermelonMintAccount.createAssociatedTokenAccount(
        secondUserKeypair.publicKey
      );

    await program.rpc.exchangeRedeemableForWatermelon(secondDeposit, {
      accounts: {
        payer: provider.wallet.publicKey,
        userAuthority: secondUserKeypair.publicKey,
        userWatermelon: secondUserWatermelon,
        userRedeemable: secondUserRedeemable,
        idoAccount,
        watermelonMint,
        redeemableMint,
        poolWatermelon,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      // signers: []
    });

    let poolWatermelonAccount = await watermelonMintAccount.getAccountInfo(
      poolWatermelon
    );
    assert.ok(poolWatermelonAccount.amount.eq(new anchor.BN(0)));
  });

  it("Withdraws total USDC from pool account", async () => {
    await program.rpc.withdrawPoolUsdc({
      accounts: {
        idoAuthority: vaultAuthority.publicKey,
        idoAuthorityUsdc,
        idoAccount,
        usdcMint,
        watermelonMint,
        poolUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [vaultAuthority],
    });

    let poolUsdcAccount = await usdcMintAccount.getAccountInfo(poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(new anchor.BN(0)));
    let idoAuthorityUsdcAccount = await usdcMintAccount.getAccountInfo(
      idoAuthorityUsdc
    );
    assert.ok(idoAuthorityUsdcAccount.amount.eq(totalPoolUsdc));
  });

  it("Withdraws USDC from the escrow account after waiting period is over", async () => {
    // Wait until the escrow period is over.
    if (Date.now() < epochTimes.endEscrow.toNumber() * 1000 + 1000) {
      await sleep(epochTimes.endEscrow.toNumber() * 1000 - Date.now() + 4000);
    }

    await program.rpc.withdrawFromEscrow(firstWithdrawal, {
      accounts: {
        payer: provider.wallet.publicKey,
        userAuthority: userKeypair.publicKey,
        userUsdc,
        escrowUsdc,
        idoAccount,
        usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    let userUsdcAccount = await usdcMintAccount.getAccountInfo(userUsdc);
    assert.ok(userUsdcAccount.amount.eq(firstWithdrawal));
  });
});
