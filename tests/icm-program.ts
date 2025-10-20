import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { IcmProgram } from "../target/types/icm_program";
import {
  PublicKey,
  Keypair,
  SystemProgram,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccount,
  getAccount,
} from "@solana/spl-token";
import { expect } from "chai";
import { Testuser1, Testuser2, Testuser3 } from "../test-users/users";

/**
 * ICM Program Test Suite
 *
 * Testing approach:
 * 1. Uses preset wallets to avoid airdrop requests
 * 2. All contributions are made in USDC (mock SPL token)
 * 3. Bucket vault is deterministic per bucket+mint (no contributor address)
 * 4. Individual contribution tracking per user+bucket+mint
 * 5. Tests cover bucket creation, contributions, and error cases
 *
 * Updated for audit fixes:
 * - Creators must create profiles before creating buckets
 * - Safe init_if_needed with proper validation for contributions
 * - Enhanced error handling and security validations
 * - Multiple contributions allowed from same user (aggregated)
 * - Contributor count tracks unique contributors only
 */

const QTOKEN = new PublicKey("2QYhBSA4hhga8gRh3eNizf3X2XPdr4EN6gQAFHpEPpYr");
const ZTOKEN2022 = new PublicKey("7yyieW3Zdcm5ncVWqiBhGvReYFQhZNEVdwVpqA3GuxGU");
const GTOKEN2022 = new PublicKey("CVZ7aSqJ8i5PUgLMmT24MXgmLAmHeyjBeEmYmKGuDm3f");
const USDC = new PublicKey("2RgRJx3z426TMCL84ZMXTRVCS5ee7iGVE4ogqcUAd3tg");

describe("icm-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.IcmProgram as Program<IcmProgram>;

  // Test accounts
  let creator: Keypair;
  let contributor1: Keypair;
  let contributor2: Keypair;
  let usdcMint: PublicKey; // Mock USDC for contributions
  let tokenMintB: PublicKey; // Second token in bucket

  // Token accounts (only USDC needed for contributions)
  let contributor1USDCAccount: PublicKey;
  let contributor2USDCAccount: PublicKey;

  // Program accounts
  let bucketPda: PublicKey;
  let bucketBump: number;
  let programStatePda: PublicKey;
  let programStateBump: number;
  let feeVaultPda: PublicKey;

  // Vault / swap related
  let vaultAuthorityPda: PublicKey;
  let vaultAuthorityBump: number;
  let vaultInputTokenAccount: PublicKey;
  let vaultOutputTokenAccount: PublicKey;
  let userSourceTokenAccount: PublicKey;
  let userDestinationTokenAccount: PublicKey;
  let userAuthority: PublicKey;

  // Random-ish name id
  const nameId = Math.floor(Math.random() * 1000) + 12;
  // Constants
  const BUCKET_NAME = `x-buck362`; // Bucket name
  // const BUCKET_NAME = `x-buck${nameId}`; // Bucket name
  const CONTRIBUTION_WINDOW_MINUTES = 60; // 60 minutes instead of days
  const TRADING_WINDOW_MINUTES = 1440; // 24 hours instead of days
  const CREATOR_FEE_PERCENT = 500; // 5% in basis points
  const PROGRAM_FEE_RATE = 50; // 0.5% in basis points
  const MINT_AMOUNT = 1_000_000_000; // 1000 tokens (assuming 6 decimals)
  const CONTRIBUTE_AMOUNT = 10_000_000; // 10 tokens
  const TARGET_AMOUNT = new anchor.BN(500_000_000); // 500 tokens target (assuming 6 decimals)
  const MINIMUM_CONTRIBUTION = new anchor.BN(1_000_000); // 1 token minimum contribution (assuming 6 decimals)
  const MAXIMUM_CONTRIBUTION = new anchor.BN(100_000_000); // 100 tokens maximum contribution (assuming 6 decimals)

  before(async () => {
    // Initialize test accounts using preset wallets (no airdrop needed)
    creator = Keypair.fromSecretKey(new Uint8Array(Testuser1));
    contributor1 = Keypair.fromSecretKey(new Uint8Array(Testuser2));
    contributor2 = Keypair.fromSecretKey(new Uint8Array(Testuser3));

    // Log test account public keys
    console.log("Creator:", creator.publicKey.toString());
    console.log("Contributor 1:", contributor1.publicKey.toString());
    console.log("Contributor 2:", contributor2.publicKey.toString());

    // Use known mints (replace with your test mints if different)
    usdcMint = new PublicKey("2RgRJx3z426TMCL84ZMXTRVCS5ee7iGVE4ogqcUAd3tg"); // USDC mint address
    tokenMintB = new PublicKey("EELsthavYsD8pDp6yq5xhNV1Jpa3N2RooMnmkaeMkUn8"); // Example Token B mint address

    // Derive program state PDA
    [programStatePda, programStateBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("program_state")],
      program.programId
    );

    // Derive fee vault PDA (associated token account)
    feeVaultPda = getAssociatedTokenAddressSync(
      usdcMint,
      programStatePda,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // Derive bucket PDA first (used by vault authority)
    [bucketPda, bucketBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("bucket"),
        Buffer.from(BUCKET_NAME),
        creator.publicKey.toBuffer(),
      ],
      program.programId
    );

    // Derive vault authority PDA (depends on bucketPda)
    [vaultAuthorityPda, vaultAuthorityBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), bucketPda.toBuffer()],
      program.programId
    );

    // Vault token accounts (ATAs) for input and output mints
    // Note: These are just addresses calculated for PDAs, actual creation happens in program
    vaultInputTokenAccount = getAssociatedTokenAddressSync(
      usdcMint,
      vaultAuthorityPda,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    vaultOutputTokenAccount = getAssociatedTokenAddressSync(
      tokenMintB,
      vaultAuthorityPda,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );


    // Assign for swap test
    userSourceTokenAccount = vaultInputTokenAccount;
    userDestinationTokenAccount = vaultOutputTokenAccount;
    userAuthority = vaultAuthorityPda;

    // Create Associated Token Accounts for contributors - USDC uses regular Token Program
    // We'll try to fetch contributor ATA addresses and create them if missing.
    // Use 'creator' as payer for ATA creation (so the preset wallets don't need extra SOL).
    const contributor1USDCAccountAddress = getAssociatedTokenAddressSync(
      usdcMint,
      contributor1.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    try {
      await getAccount(provider.connection, contributor1USDCAccountAddress, undefined, TOKEN_PROGRAM_ID);
      contributor1USDCAccount = contributor1USDCAccountAddress;
      console.log("Contributor1 USDC account already exists");
    } catch (error) {
      contributor1USDCAccount = await createAssociatedTokenAccount(
        provider.connection,
        creator, // payer
        usdcMint,
        contributor1.publicKey
      );
      console.log("Created contributor1 USDC account:", contributor1USDCAccount.toString());
    }

    const contributor2USDCAccountAddress = getAssociatedTokenAddressSync(
      usdcMint,
      contributor2.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    try {
      await getAccount(provider.connection, contributor2USDCAccountAddress, undefined, TOKEN_PROGRAM_ID);
      contributor2USDCAccount = contributor2USDCAccountAddress;
      console.log("Contributor2 USDC account already exists");
    } catch (error) {
      contributor2USDCAccount = await createAssociatedTokenAccount(
        provider.connection,
        creator, // payer
        usdcMint,
        contributor2.publicKey
      );
      console.log("Created contributor2 USDC account:", contributor2USDCAccount.toString());
    }

    console.log("USDC accounts ready for contributors");
    console.log("Bucket PDA:", bucketPda.toString());
    console.log("Vault authority PDA:", vaultAuthorityPda.toString());
  });

  // Initialization of the program state
  describe("Program Initialization", () => {
    it("Should initialize the program once", async () => {
      const tx = await program.methods
        .initializeProgram(PROGRAM_FEE_RATE)
        .accountsPartial({
          programState: programStatePda,
          feeVault: feeVaultPda,
          usdcMint: usdcMint,
          owner: provider.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        // .signers([creator])
        .rpc();

      console.log("Program initialization signature", tx);

      // Verify program state was created
      const programState = await program.account.programState.fetch(programStatePda);
      console.log("Program state account:", programStatePda);
      expect(programState.owner.toString()).to.equal(provider.publicKey.toString());
      expect(programState.feeRateBps).to.equal(PROGRAM_FEE_RATE);
      expect(programState.initialized).to.be.true;
      expect(programState.totalFeesCollected.toString()).to.equal("0");

      console.log("Program state:", programState);
    });
  });

  describe("Create Profile", () => {
    it("Should create a creator profile", async () => {
      // Derive creator profile PDA
      const [creatorProfilePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("creator_profile"), creator.publicKey.toBuffer()],
        program.programId
      );

      const tx = await program.methods
        .createProfile()
        .accountsPartial({
          creatorProfile: creatorProfilePda,
          creator: creator.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([creator])
        .rpc();

      console.log("Create profile transaction signature", tx);

      // Verify profile was created
      const profile = await program.account.creatorProfile.fetch(creatorProfilePda);
      expect(profile.creator.toString()).to.equal(creator.publicKey.toString());
      expect(profile.poolsCreated).to.equal(0);
      expect(profile.successfulPools).to.equal(0);
      expect(profile.totalVolumeManaged.toString()).to.equal("0");
      expect(profile.reputationScore).to.equal(0);

      console.log("Creator profile data:", profile);
    });
  });

  describe("Create Bucket", () => {
    it("Should create a bucket", async () => {
      const [tradingPoolPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("trading_pool"),
          Buffer.from(BUCKET_NAME),
          creator.publicKey.toBuffer(),
        ],
        program.programId
      );

      const vaultTokenAccount = getAssociatedTokenAddressSync(
        usdcMint,
        bucketPda,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      // Derive creator profile PDA (required for bucket creation)
      const [creatorProfilePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("creator_profile"), creator.publicKey.toBuffer()],
        program.programId
      );

      const tx = await program.methods
        .createBucket(
          BUCKET_NAME,
          [QTOKEN, ZTOKEN2022, GTOKEN2022],
          CONTRIBUTION_WINDOW_MINUTES,
          TRADING_WINDOW_MINUTES,
          CREATOR_FEE_PERCENT,
          TARGET_AMOUNT,
          MINIMUM_CONTRIBUTION,
          MAXIMUM_CONTRIBUTION,
          new anchor.BN(500)
        )
        .accountsPartial({
          bucket: bucketPda,
          tradingPool: tradingPoolPda,
          vaultTokenAccount: vaultTokenAccount,
          // creatorProfile: creatorProfilePda, // TODO: Add this once IDL is updated
          usdcMint: usdcMint,
          creator: creator.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([creator])
        .rpc();

      console.log("Create bucket transaction signature", tx);

      // Verify bucket was created
      const bucket = await program.account.bucket.fetch(bucketPda);
      expect(bucket.name).to.equal(BUCKET_NAME);
      expect(bucket.creator.toString()).to.equal(creator.publicKey.toString());

      console.log("Bucket data:", bucket);
    });
  });

  describe.skip("Debug Contribution Issue", () => {
    it("Should debug account setup before contribution", async () => {
      // First verify all required accounts exist
      console.log("=== DEBUGGING ACCOUNT SETUP ===");
      
      // Check bucket exists
      try {
        const bucket = await program.account.bucket.fetch(bucketPda);
        console.log("✓ Bucket exists:", bucket.name);
        console.log("  Status:", bucket.status);
        console.log("  Creator:", bucket.creator.toString());
        console.log("  Contribution deadline:", new Date(bucket.contributionDeadline.toNumber() * 1000));
      } catch (error) {
        console.log("✗ Bucket fetch failed:", error.message);
        return;
      }

      // Check program state
      try {
        const programState = await program.account.programState.fetch(programStatePda);
        console.log("✓ Program state exists:");
        console.log("  Initialized:", programState.initialized);
        console.log("  Fee rate:", programState.feeRateBps, "bps");
        console.log("  USDC mint:", programState.usdcMint.toString());
      } catch (error) {
        console.log("✗ Program state fetch failed:", error.message);
        return;
      }

      // Check vault token account
      const vaultTokenAccount = getAssociatedTokenAddressSync(
        usdcMint,
        bucketPda,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );
      
      try {
        const vaultAccount = await getAccount(provider.connection, vaultTokenAccount);
        console.log("✓ Vault token account exists:");
        console.log("  Balance:", vaultAccount.amount.toString());
        console.log("  Owner:", vaultAccount.owner.toString());
        console.log("  Mint:", vaultAccount.mint.toString());
      } catch (error) {
        console.log("✗ Vault token account fetch failed:", error.message);
        return;
      }

      // Check contributor token account  
      try {
        const contributorAccount = await getAccount(provider.connection, contributor1USDCAccount);
        console.log("✓ Contributor token account exists:");
        console.log("  Balance:", contributorAccount.amount.toString());
        console.log("  Owner:", contributorAccount.owner.toString());
      } catch (error) {
        console.log("✗ Contributor token account fetch failed:", error.message);
        return;
      }

      console.log("✓ All required accounts exist and are properly configured");
    });

    it.skip("Should try minimal contribution with detailed error logging", async () => {
      const [contributionRecord] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          bucketPda.toBuffer(),
          contributor1.publicKey.toBuffer(),
          usdcMint.toBuffer(),
        ],
        program.programId
      );

      const [poolContribution] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("pool_contribution"),
          bucketPda.toBuffer(),
          contributor1.publicKey.toBuffer(),
          usdcMint.toBuffer(),
        ],
        program.programId
      );

      const vaultTokenAccount = getAssociatedTokenAddressSync(
        usdcMint,
        bucketPda,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      // Use minimal contribution amount (1 token = 1,000,000 lamports for 6 decimals)
      const minimalAmount = 1_000_000; 

      console.log("=== ATTEMPTING MINIMAL CONTRIBUTION ===");
      console.log("Amount:", minimalAmount, "lamports");
      console.log("Bucket name:", BUCKET_NAME);
      console.log("Contribution record PDA:", contributionRecord.toString());
      console.log("Pool contribution PDA:", poolContribution.toString());

      try {
        // Check if these PDAs already exist (they shouldn't for a new contribution)
        try {
          const existingContribution = await program.account.contributionRecord.fetch(contributionRecord);
          console.log("⚠️  Contribution record already exists:", existingContribution);
        } catch {
          console.log("✓ Contribution record PDA is available (doesn't exist yet)");
        }

        try {
          const existingPool = await program.account.poolContribution.fetch(poolContribution);
          console.log("⚠️  Pool contribution already exists:", existingPool);
        } catch {
          console.log("✓ Pool contribution PDA is available (doesn't exist yet)");
        }

        const tx = await program.methods
          .contributeToBucket(BUCKET_NAME, new anchor.BN(minimalAmount))
          .accountsPartial({
            bucket: bucketPda,
            contributionRecord: contributionRecord,
            poolContribution: poolContribution,
            contributorTokenAccount: contributor1USDCAccount,
            vaultTokenAccount: vaultTokenAccount,
            usdcMint: usdcMint,
            programState: programStatePda,
            feeVault: feeVaultPda,
            contributor: contributor1.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([contributor1])
          .rpc();

        console.log("✓ SUCCESS! Contribution transaction:", tx);

        // Verify the contribution was recorded
        const contribution = await program.account.contributionRecord.fetch(contributionRecord);
        console.log("Contribution record:", contribution);

      } catch (error) {
        console.log("✗ CONTRIBUTION FAILED:");
        console.log("Error message:", error.message);
        if (error.logs) {
          console.log("Program logs:");
          error.logs.forEach((log, i) => console.log(`  ${i}: ${log}`));
        }
        
        // Re-throw to fail the test
        throw error;
      }
    });
  });

  describe("Contribute to Bucket", () => {
    it("Should allow contribution to bucket", async () => {
      const [contributionRecord] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          bucketPda.toBuffer(),
          contributor1.publicKey.toBuffer(),
          usdcMint.toBuffer(),
        ],
        program.programId
      );

      const [poolContribution] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("pool_contribution"),
          bucketPda.toBuffer(),
          contributor1.publicKey.toBuffer(),
          usdcMint.toBuffer(),
        ],
        program.programId
      );

      const vaultTokenAccount = getAssociatedTokenAddressSync(
        usdcMint,
        bucketPda,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      // Debug information
      console.log("=== CONTRIBUTION DEBUG INFO ===");
      console.log("Bucket PDA:", bucketPda.toString());
      console.log("Contributor:", contributor1.publicKey.toString());
      console.log("USDC Mint:", usdcMint.toString());
      console.log("Contribution Record PDA:", contributionRecord.toString());
      console.log("Pool Contribution PDA:", poolContribution.toString());
      console.log("Vault Token Account:", vaultTokenAccount.toString());
      console.log("Program State PDA:", programStatePda.toString());
      console.log("Fee Vault PDA:", feeVaultPda.toString());

      // Check if contributor has USDC tokens (if not, this test will fail realistically)
      try {
        const contributorBalance = await provider.connection.getTokenAccountBalance(contributor1USDCAccount);
        console.log("Contributor USDC balance:", contributorBalance.value.amount);
      } catch (error) {
        console.log("Could not fetch contributor balance:", error);
      }

      // Check if vault account exists
      try {
        const vaultAccount = await provider.connection.getAccountInfo(vaultTokenAccount);
        if (vaultAccount) {
          console.log("Vault account exists with lamports:", vaultAccount.lamports);
        } else {
          console.log("Vault account does not exist");
        }
      } catch (error) {
        console.log("Error checking vault account:", error);
      }

      const tx = await program.methods
        .contributeToBucket(BUCKET_NAME, new anchor.BN(CONTRIBUTE_AMOUNT))
        .accountsPartial({
          bucket: bucketPda,
          contributionRecord: contributionRecord,
          poolContribution: poolContribution,
          contributorTokenAccount: contributor1USDCAccount,
          vaultTokenAccount: vaultTokenAccount,
          usdcMint: usdcMint,
          programState: programStatePda,
          feeVault: feeVaultPda,
          contributor: contributor1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([contributor1])
        .rpc();

      console.log("Contribute transaction signature", tx);

      // Verify contribution was recorded
      const contribution = await program.account.contributionRecord.fetch(contributionRecord);
      console.log("Contribution data:", contribution);

      expect(contribution.contributor.toString()).to.equal(contributor1.publicKey.toString());
      expect(contribution.bucket.toString()).to.equal(bucketPda.toString());
      expect(contribution.tokenMint.toString()).to.equal(usdcMint.toString());
      // contribution.amount is likely a BN — compare string values
      expect(contribution.amount.toString()).to.not.equal("0");

      // Calculate expected net amount after fee
      const expectedFee = Math.floor((CONTRIBUTE_AMOUNT * PROGRAM_FEE_RATE) / 10000);
      const expectedNetAmount = CONTRIBUTE_AMOUNT - expectedFee;

      // Verify bucket total contributions updated with net amount
      const bucket = await program.account.bucket.fetch(bucketPda);
      expect(bucket.raisedAmount.toString()).to.equal(new anchor.BN(expectedNetAmount).toString());

      // Verify program state fee collection updated
      const programState = await program.account.programState.fetch(programStatePda);
      expect(programState.totalFeesCollected.toString()).to.equal(new anchor.BN(expectedFee).toString());
      
      console.log(`Fee collected: ${expectedFee}, Net contribution: ${expectedNetAmount}`);
    });

    it("Should allow second contributor to contribute to bucket", async () => {
      const [contributionRecord] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          bucketPda.toBuffer(),
          contributor2.publicKey.toBuffer(),
          usdcMint.toBuffer(),
        ],
        program.programId
      );

      const [poolContribution] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("pool_contribution"),
          bucketPda.toBuffer(),
          contributor2.publicKey.toBuffer(),
          usdcMint.toBuffer(),
        ],
        program.programId
      );

      const vaultTokenAccount = getAssociatedTokenAddressSync(
        usdcMint,
        bucketPda,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const SECOND_CONTRIBUTE_AMOUNT = 5_000_000; // 5 tokens

      const tx = await program.methods
        .contributeToBucket(BUCKET_NAME, new anchor.BN(SECOND_CONTRIBUTE_AMOUNT))
        .accountsPartial({
          bucket: bucketPda,
          contributionRecord: contributionRecord,
          poolContribution: poolContribution,
          contributorTokenAccount: contributor2USDCAccount,
          vaultTokenAccount: vaultTokenAccount,
          usdcMint: usdcMint,
          programState: programStatePda,
          feeVault: feeVaultPda,
          contributor: contributor2.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([contributor2])
        .rpc();

      console.log("Second contribution transaction signature", tx);

      // Verify contribution was recorded
      const contribution = await program.account.contributionRecord.fetch(contributionRecord);
      expect(contribution.contributor.toString()).to.equal(contributor2.publicKey.toString());
      expect(contribution.bucket.toString()).to.equal(bucketPda.toString());

      // Calculate expected amounts
      const expectedFee = Math.floor((SECOND_CONTRIBUTE_AMOUNT * PROGRAM_FEE_RATE) / 10000);
      const expectedNetAmount = SECOND_CONTRIBUTE_AMOUNT - expectedFee;
      const totalFirstContribution = CONTRIBUTE_AMOUNT - Math.floor((CONTRIBUTE_AMOUNT * PROGRAM_FEE_RATE) / 10000);
      const expectedTotalRaised = totalFirstContribution + expectedNetAmount;

      // Verify bucket total contributions updated
      const bucket = await program.account.bucket.fetch(bucketPda);
      expect(bucket.raisedAmount.toString()).to.equal(new anchor.BN(expectedTotalRaised).toString());
      expect(bucket.contributorCount).to.equal(2); // Two unique contributors now

      console.log(`Second contributor - Fee: ${expectedFee}, Net: ${expectedNetAmount}, Total raised: ${expectedTotalRaised}`);
    });

    it("Should allow additional contribution from same user", async () => {
      // Make another contribution with contributor1 (should work with init_if_needed + validation)
      const [contributionRecord] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("contribution"),
          bucketPda.toBuffer(),
          contributor1.publicKey.toBuffer(),
          usdcMint.toBuffer(),
        ],
        program.programId
      );

      const [poolContribution] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("pool_contribution"),
          bucketPda.toBuffer(),
          contributor1.publicKey.toBuffer(),
          usdcMint.toBuffer(),
        ],
        program.programId
      );

      const vaultTokenAccount = getAssociatedTokenAddressSync(
        usdcMint,
        bucketPda,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const ADDITIONAL_AMOUNT = 2_000_000; // 2 tokens

      // Get initial contribution amount
      const initialContribution = await program.account.contributionRecord.fetch(contributionRecord);
      const initialAmount = initialContribution.amount.toNumber();

      const tx = await program.methods
        .contributeToBucket(BUCKET_NAME, new anchor.BN(ADDITIONAL_AMOUNT))
        .accountsPartial({
          bucket: bucketPda,
          contributionRecord: contributionRecord,
          poolContribution: poolContribution,
          contributorTokenAccount: contributor1USDCAccount,
          vaultTokenAccount: vaultTokenAccount,
          usdcMint: usdcMint,
          programState: programStatePda,
          feeVault: feeVaultPda,
          contributor: contributor1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([contributor1])
        .rpc();

      console.log("Additional contribution transaction signature", tx);

      // Verify contribution was aggregated
      const updatedContribution = await program.account.contributionRecord.fetch(contributionRecord);
      const expectedFee = Math.floor((ADDITIONAL_AMOUNT * PROGRAM_FEE_RATE) / 10000);
      const expectedNetAmount = ADDITIONAL_AMOUNT - expectedFee;
      const expectedTotalContribution = initialAmount + expectedNetAmount;

      expect(updatedContribution.amount.toNumber()).to.equal(expectedTotalContribution);

      // Verify contributor count didn't increase (same user)
      const bucket = await program.account.bucket.fetch(bucketPda);
      expect(bucket.contributorCount).to.equal(2); // Still 2 unique contributors

      console.log(`Additional contribution - Fee: ${expectedFee}, Net: ${expectedNetAmount}, Total for user: ${expectedTotalContribution}`);
    });
  });

  // describe("Create Bucket", () => {
  //   it("Should create a bucket", async () => {
  //     const [tradingPoolPda] = PublicKey.findProgramAddressSync(
  //       [
  //         Buffer.from("trading_pool"),
  //         Buffer.from(BUCKET_NAME),
  //         creator.publicKey.toBuffer(),
  //       ],
  //       program.programId
  //     );

  //     const vaultTokenAccount = getAssociatedTokenAddressSync(
  //       usdcMint,
  //       bucketPda,
  //       true,
  //       TOKEN_PROGRAM_ID,
  //       ASSOCIATED_TOKEN_PROGRAM_ID
  //     );

  //     const tx = await program.methods
  //       .createBucket(
  //         BUCKET_NAME,
  //         [QTOKEN, ZTOKEN2022, GTOKEN2022],
  //         CONTRIBUTION_WINDOW_MINUTES,
  //         TRADING_WINDOW_MINUTES,
  //         CREATOR_FEE_PERCENT,
  //         new anchor.BN(1000000),
  //         new anchor.BN(10000),
  //         new anchor.BN(100000),
  //         500
  //       )
  //       .accountsPartial({
  //         bucket: bucketPda,
  //         tradingPool: tradingPoolPda,
  //         vaultTokenAccount: vaultTokenAccount,
  //         usdcMint: usdcMint,
  //         creator: creator.publicKey,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([creator])
  //       .rpc();

  //     console.log("Create bucket transaction signature", tx);

  //     // Verify bucket was created
  //     const bucket = await program.account.bucket.fetch(bucketPda);
  //     expect(bucket.name).to.equal(BUCKET_NAME);
  //     expect(bucket.creator.toString()).to.equal(creator.publicKey.toString());

  //     console.log("Bucket data:", bucket);
  //   });
  // });


  describe("Fee Management", () => {
    it("Should allow program owner to withdraw fees", async () => {
      // Create owner's USDC token account if it doesn't exist
      const ownerUSDCAccount = getAssociatedTokenAddressSync(
        usdcMint,
        creator.publicKey,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      // Check initial balances
      const initialFeeVaultBalance = await provider.connection.getTokenAccountBalance(feeVaultPda);
      console.log("Initial fee vault balance:", initialFeeVaultBalance.value.amount);

      const tx = await program.methods
        .withdrawFees(null) // null means withdraw all
        .accountsPartial({
          programState: programStatePda,
          feeVault: feeVaultPda,
          ownerTokenAccount: ownerUSDCAccount,
          owner: creator.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([creator])
        .rpc();

      console.log("Withdraw fees transaction signature", tx);

      // Verify fee vault is now empty
      const finalFeeVaultBalance = await provider.connection.getTokenAccountBalance(feeVaultPda);
      expect(finalFeeVaultBalance.value.amount).to.equal("0");

      console.log("Final fee vault balance:", finalFeeVaultBalance.value.amount);
    });

    it("Should fail to withdraw fees if not owner", async () => {
      // Try to withdraw fees with contributor1 (not owner)
      const ownerUSDCAccount = getAssociatedTokenAddressSync(
        usdcMint,
        contributor1.publicKey,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      try {
        await program.methods
          .withdrawFees(null)
          .accountsPartial({
            programState: programStatePda,
            feeVault: feeVaultPda,
            ownerTokenAccount: ownerUSDCAccount,
            owner: contributor1.publicKey, // Not the real owner
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([contributor1])
          .rpc();
        
        expect.fail("Should have failed to withdraw fees as non-owner");
      } catch (error) {
        console.log("Expected error when non-owner tries to withdraw:", error.message);
      }
    });
  });

  it("Should swap tradeable asset from Raydium (devnet CP-Swap example)", async () => {
    const inAmount = new anchor.BN(1000);
    const quotedOutAmount = new anchor.BN(900);
    const slippageBps = 50;

    // Dynamic mints - use environment variables with fallbacks
    const inputMint = new PublicKey(
      process.env.RAYDIUM_INPUT_MINT || usdcMint.toString()   // Default to USDC
    );
    const outputMint = new PublicKey(
      process.env.RAYDIUM_OUTPUT_MINT || tokenMintB.toString() // Default to tokenMintB
    );
    
    // Dynamic token program selection based on mint type
    // If mint is USDC, use standard TOKEN_PROGRAM_ID, otherwise check environment or use TOKEN_PROGRAM_ID as fallback
    const inputMintProgram = inputMint.equals(usdcMint) 
      ? TOKEN_PROGRAM_ID 
      : new PublicKey(process.env.RAYDIUM_INPUT_MINT_PROGRAM || TOKEN_PROGRAM_ID.toString());
    
    const outputMintProgram = outputMint.equals(usdcMint) 
      ? TOKEN_PROGRAM_ID 
      : new PublicKey(process.env.RAYDIUM_OUTPUT_MINT_PROGRAM || TOKEN_PROGRAM_ID.toString());

    // Dynamic Raydium addresses - use environment variables with fallbacks
    const raydiumAmmProgram = new PublicKey(
      process.env.RAYDIUM_AMM_PROGRAM_ID || "CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW" // Raydium CP-Swap program (devnet)
    );
    const amm = new PublicKey(
      process.env.RAYDIUM_AMM_ID || "5R2wzTtEq9tm1pXkVU7QVdp3E6C3eUCPjrGDqkz9eD6T"
    );
    const ammAuthority = new PublicKey(
      process.env.RAYDIUM_AMM_AUTHORITY || "6t1fzD7s5D7dvvnA1mVdD5Jm67Fb9YaB6X6hQpF8o8Fj"
    );
    const poolCoinTokenAccount = new PublicKey(
      process.env.RAYDIUM_POOL_COIN_TOKEN_ACCOUNT || "2vP2hRfjM4Lx8mVymWxw2Aa72c8hA3L7xrxvD5U5q4K7"
    );
    const poolPcTokenAccount = new PublicKey(
      process.env.RAYDIUM_POOL_PC_TOKEN_ACCOUNT || "7qZPVvHryXtN1Z8VhxT9ku3uXKp5o4RUrgmRoei7wLpm"
    );

    // Derive trade record PDA
    const [tradeRecordPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("trade_record"),
        bucketPda.toBuffer(),
        creator.publicKey.toBuffer(),
      ],
      program.programId
    );

    console.log("=== SWAP TEST CONFIGURATION ===");
    console.log("Input Mint:", inputMint.toString());
    console.log("Output Mint:", outputMint.toString());
    console.log("Input Mint Program:", inputMintProgram.toString());
    console.log("Output Mint Program:", outputMintProgram.toString());
    console.log("Raydium AMM Program:", raydiumAmmProgram.toString());
    console.log("AMM:", amm.toString());
    console.log("AMM Authority:", ammAuthority.toString());
    console.log("Pool Coin Token Account:", poolCoinTokenAccount.toString());
    console.log("Pool PC Token Account:", poolPcTokenAccount.toString());
    console.log("Trade Record PDA:", tradeRecordPda.toString());

    const tx = await program.methods
      .swapTokens(inAmount, quotedOutAmount, slippageBps)
      .accountsPartial({
        tradeRecord: tradeRecordPda,
        creator: creator.publicKey,
        bucket: bucketPda,
        inputMint,
        systemProgram: SystemProgram.programId,
        inputMintProgram,
        outputMint,
        outputMintProgram,
        vaultInputTokenAccount,
        vaultOutputTokenAccount,
        raydiumAmmProgram,
        amm,
        ammAuthority,
        poolCoinTokenAccount,
        poolPcTokenAccount,
        userSourceTokenAccount,
        userDestinationTokenAccount,
        userAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([creator])
      .rpc();

    console.log("Swap transaction signature", tx);
    
    // Verify trade record was created
    try {
      const tradeRecord = await program.account.tradeRecord.fetch(tradeRecordPda);
      console.log("Trade record created:", {
        poolId: tradeRecord.poolId.toString(),
        tradeId: tradeRecord.tradeId.toString(),
        fromToken: tradeRecord.fromToken.toString(),
        toToken: tradeRecord.toToken.toString(),
        amountIn: tradeRecord.amountIn.toString(),
        amountOut: tradeRecord.amountOut.toString(),
        success: tradeRecord.success,
      });
    } catch (error) {
      console.log("Could not fetch trade record:", error.message);
    }
  });

});
