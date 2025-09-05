import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { IcmProgram } from "../target/types/icm_program";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram
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
 */

describe("icm-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.IcmProgram as Program<IcmProgram>;

  // Test accounts
  let creator: Keypair;
  let contributor1: Keypair;
  let contributor2: Keypair;
  let usdcMint: PublicKey; // Mock USDC for contributions
  let tokenMintB: PublicKey; // Second token in bucket
  let tokenMintC: PublicKey; // Third token (for error testing)

  // Token accounts (only USDC needed for contributions)
  let contributor1USDCAccount: PublicKey;
  let contributor2USDCAccount: PublicKey;

  // Program accounts
  let bucketPda: PublicKey;
  let bucketBump: number;

  // Constants
  const BUCKET_NAME = "xMANGA"; // Bucket name
  const CONTRIBUTION_WINDOW_DAYS = 7; // 7 days
  const TRADING_WINDOW_DAYS = 30; // 30 days
  const CREATOR_FEE_PERCENT = 500; // 5% in basis points
  const MINT_AMOUNT = 1_000_000_000; // 1000 tokens (assuming 6 decimals)
  const CONTRIBUTE_AMOUNT = 100_000_000_000; // 100 tokens

  before(async () => {
    // Initialize test accounts using preset wallets (no airdrop needed)
    creator = Keypair.fromSecretKey(new Uint8Array(Testuser1));
    contributor1 = Keypair.fromSecretKey(new Uint8Array(Testuser2));
    contributor2 = Keypair.fromSecretKey(new Uint8Array(Testuser3));

    // console.log("Creator:", creator.publicKey.toString());
    // console.log("Contributor1:", contributor1.publicKey.toString());
    // console.log("Contributor2:", contributor2.publicKey.toString());

    // Create the token mints - USDC uses regular Token Program, others use Token 2022
    usdcMint = new PublicKey("7efeK5MMfmgcNeJkutSduzBGskFHziBhvmoPcPrJBmuF"); // USDC mint address

    tokenMintB = new PublicKey("EELsthavYsD8pDp6yq5xhNV1Jpa3N2RooMnmkaeMkUn8"); // Example Token B mint address

    tokenMintC = new PublicKey("EELsthavYsD8pDp6yq5xhNV1Jpa3N2RooMnmkaeMkUn8"); // Example Token C mint address
  
    
    // console.log("USDC Mint:", usdcMint.toString());
    // console.log("Token B:", tokenMintB.toString());
    // console.log("Token C:", tokenMintC.toString());

    // Create Associated Token Accounts for contributors - USDC uses regular Token Program
    // Only create for contributor accounts (Keypair), not for vaults (PDAs)
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
        contributor1,
        usdcMint,
        contributor1.publicKey,
        undefined,
        TOKEN_PROGRAM_ID
      );
      console.log("Created contributor1 USDC account");
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
        contributor2,
        usdcMint,
        contributor2.publicKey,
        undefined,
        TOKEN_PROGRAM_ID
      );
      console.log("Created contributor2 USDC account");
    }

    console.log("USDC accounts created for contributors");

    // Derive bucket PDA
    [bucketPda, bucketBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("bucket"),
        Buffer.from(BUCKET_NAME),
        creator.publicKey.toBuffer()
      ],
      program.programId
    );

    console.log("Bucket PDA:", bucketPda.toString());
  });

  // describe("Create Profile", () => {
  //   it("Should create a creator profile successfully", async () => {
  //     const [creatorProfilePda, _] = PublicKey.findProgramAddressSync(
  //       [
  //         Buffer.from("creator_profile"),
  //         creator.publicKey.toBuffer()
  //       ],
  //       program.programId
  //     );

  //     const tx = await program.methods
  //       .createProfile()
  //       .accounts({
  //         creatorProfile: creatorProfilePda,
  //         creator: creator.publicKey,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([creator])
  //       .rpc();

  //     console.log("Create profile transaction signature", tx);

  //     // Verify profile was created with correct data
  //     const profile = await program.account.creatorProfile.fetch(creatorProfilePda);
  //     console.log("Creator profile data:", profile);
      
  //     expect(profile.creator.toString()).to.equal(creator.publicKey.toString());
  //     expect(profile.poolsCreated).to.equal(0);
  //     expect(profile.successfulPools).to.equal(0);
  //     expect(profile.totalVolumeManaged.toString()).to.equal("0");
  //     expect(profile.reputationScore).to.equal(0);
  //     expect(profile.createdAt).to.be.greaterThan(0);
  //   });

  //   it("Should fail to create duplicate profile", async () => {
  //     const [creatorProfilePda, _] = PublicKey.findProgramAddressSync(
  //       [
  //         Buffer.from("creator_profile"),
  //         creator.publicKey.toBuffer()
  //       ],
  //       program.programId
  //     );

  //     try {
  //       const tx =await program.methods
  //         .createProfile()
  //         .accounts({
  //           creatorProfile: creatorProfilePda,
  //           creator: creator.publicKey,
  //           systemProgram: SystemProgram.programId,
  //         })
  //         .signers([creator])
  //         .rpc();

  //       console.log("Create profile transaction signature", tx);

  //       // Should not reach this point
  //       expect.fail("Expected transaction to fail");
  //     } catch (error) {
  //       // Expect the transaction to fail because profile already exists
  //       expect(error.message).to.include("already in use");
  //     }
  //   });
  // });

  // describe("Create Bucket", () => {
  //   it("Should create a bucket successfully", async () => {
  //     const TARGET_AMOUNT = new anchor.BN(1_000_000_000_000); // 100 tokens in smallest units
  //     // min_contribution: u64,
  //     const MIN_CONTRIBUTION = new anchor.BN(100_000_000); // 10 tokens in smallest units
  //     // max_contribution: u64,
  //     const MAX_CONTRIBUTION = new anchor.BN(1_000_000_000_000); // 100 tokens in smallest units
  //     // management_fee: u16,
  //     const MANAGEMENT_FEE = 100; // 1% in basis points

  //     const tokenMints = [usdcMint, tokenMintB]; // USDC + Token B

  //     const [tradingPoolPda, tradingPoolBump] = PublicKey.findProgramAddressSync(
  //       [
  //         Buffer.from("trading_pool"),
  //         Buffer.from(BUCKET_NAME),
  //         creator.publicKey.toBuffer()
  //       ],
  //       program.programId
  //     );

  //     const [creatorProfilePda, _] = PublicKey.findProgramAddressSync(
  //       [
  //         Buffer.from("creator_profile"),
  //         creator.publicKey.toBuffer()
  //       ],
  //       program.programId
  //     );

  //     const vaultTokenAccount = getAssociatedTokenAddressSync(
  //       usdcMint,
  //       bucketPda,
  //       true,
  //       // TOKEN_PROGRAM_ID,
  //       // ASSOCIATED_TOKEN_PROGRAM_ID
  //     );

  //     const tx = await program.methods
  //       .createBucket(
  //         BUCKET_NAME,
  //         tokenMints,
  //         CONTRIBUTION_WINDOW_DAYS,
  //         TRADING_WINDOW_DAYS,
  //         CREATOR_FEE_PERCENT,
  //         TARGET_AMOUNT,
  //         MIN_CONTRIBUTION,
  //         MAX_CONTRIBUTION,
  //         MANAGEMENT_FEE,
  //       )
  //       .accounts({
  //         bucket: bucketPda,
  //         tradingPool: tradingPoolPda,
  //         vaultTokenAccount: vaultTokenAccount,
  //         usdcMint: usdcMint,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //         creator: creator.publicKey,
  //         systemProgram: SystemProgram.programId,
  //       })
  //       .signers([creator])
  //       .rpc();

  //     console.log("Create bucket transaction signature", tx);

  //     // Verify bucket was created with correct data
  //     const bucket = await program.account.bucket.fetch(bucketPda);
  //     console.log("Bucket data:", bucket);
  //     expect(bucket.creator.toString()).to.equal(creator.publicKey.toString());
  //     expect(bucket.tokenMints.length).to.equal(2);
  //     expect(bucket.tokenMints[0].toString()).to.equal(usdcMint.toString());
  //     expect(bucket.tokenMints[1].toString()).to.equal(tokenMintB.toString());
  //     expect(bucket.creatorFeePercent).to.equal(CREATOR_FEE_PERCENT);
  //     expect(bucket.name).to.equal(BUCKET_NAME);
  //     expect(bucket.totalContributions.toString()).to.equal("0");
  //     expect(bucket.bump).to.equal(bucketBump);
  //   });
  // });

  describe("Contribute to Bucket", () => {
    it("Should allow contribution to bucket", async () => {

      const [contributionRecord, _] = PublicKey.findProgramAddressSync(
          [
              Buffer.from("contribution"),
              bucketPda.toBuffer(),
              contributor1.publicKey.toBuffer(),
              usdcMint.toBuffer()
          ],
          program.programId
      );

      const [poolContribution, _poolBump] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("pool_contribution"),
            bucketPda.toBuffer(),
            contributor1.publicKey.toBuffer(),
            usdcMint.toBuffer()
          ],
          program.programId
        );

      const vaultTokenAccount = getAssociatedTokenAddressSync(
        usdcMint,
        bucketPda, // bucket PDA, not contributor
        true, // isPDA
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const tx = await program.methods
          .contributeToBucket(
              BUCKET_NAME,
              usdcMint,
              new anchor.BN(CONTRIBUTE_AMOUNT)
          )
          .accountsPartial({
              bucket: bucketPda,
              contributionRecord: contributionRecord,
              pool_contribution: poolContribution,
              contributorTokenAccount: contributor1USDCAccount,
              vaultTokenAccount: vaultTokenAccount,
              tokenMint: usdcMint,
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
        expect(contribution.amount.toString()).to.equal(CONTRIBUTE_AMOUNT.toString());

        // Verify bucket total contributions updated
        const bucket = await program.account.bucket.fetch(bucketPda);
        expect(bucket.totalContributions.toString()).to.equal(CONTRIBUTE_AMOUNT.toString());
    });
  });
});
