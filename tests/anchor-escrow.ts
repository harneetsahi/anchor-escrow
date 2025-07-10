import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import { Program } from "@coral-xyz/anchor";
import { AnchorEscrow } from "../target/types/anchor_escrow";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { assert } from "chai";

describe("anchor-escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.anchorEscrow as Program<AnchorEscrow>;

  let maker: Keypair;
  let taker: Keypair;
  let mintA: PublicKey;
  let mintB: PublicKey;
  let makerAtaA: PublicKey;
  let makerAtaB: PublicKey;
  let takerAtaA: PublicKey;
  let takerAtaB: PublicKey;

  let escrowPDA: PublicKey;
  let escrowSeed: anchor.BN;
  let bump: number;
  let vaultAta: PublicKey;

  const toBuffer = (num: BN) => {
    return num.toArrayLike(Buffer, "le", 8);
  };

  before(async () => {
    maker = Keypair.generate();
    taker = Keypair.generate();

    const makerAirdrop = await provider.connection.requestAirdrop(
      maker.publicKey,
      200 * LAMPORTS_PER_SOL
    );

    await provider.connection.confirmTransaction(makerAirdrop, "confirmed");

    const takerAirdrop = await provider.connection.requestAirdrop(
      taker.publicKey,
      200 * LAMPORTS_PER_SOL
    );

    await provider.connection.confirmTransaction(takerAirdrop, "confirmed");

    mintA = await createMint(
      provider.connection,
      maker,
      maker.publicKey,
      null,
      6
    );

    mintB = await createMint(
      provider.connection,
      taker,
      taker.publicKey,
      null,
      6
    );

    makerAtaA = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        maker,
        mintA,
        maker.publicKey
      )
    ).address;

    takerAtaB = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        taker,
        mintB,
        taker.publicKey
      )
    ).address;

    await mintTo(provider.connection, maker, mintA, makerAtaA, maker, 100n);

    await mintTo(provider.connection, taker, mintB, takerAtaB, taker, 100n);

    escrowSeed = new BN(Date.now());

    [escrowPDA, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.publicKey.toBuffer(), toBuffer(escrowSeed)],
      program.programId
    );

    vaultAta = await getAssociatedTokenAddress(
      mintA,
      escrowPDA,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    try {
      const tx = await program.methods
        .initEscrow(escrowSeed, new anchor.BN(100))
        .accounts({
          maker: maker.publicKey,
          mintA,
          mintB,
          makerAtaA,
          vault: vaultAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([maker])
        .rpc();
      console.log("Your transaction signature", tx);
    } catch (error) {
      console.error("init failed ", error);
    }

    const vaultAccount = await getAccount(provider.connection, vaultAta);

    assert.equal(
      vaultAccount.amount.toString(),
      "0",
      "initial vault amount should be 0"
    );
  });

  it("Deposits into the vault", async () => {
    const depositAmount = new BN(100);
    try {
      await program.methods
        .deposit(depositAmount)
        .accounts({
          maker: maker.publicKey,
          mintA,
          mintB,
          makerAtaA,
          escrow: escrowPDA,
          vault: vaultAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([maker])
        .rpc();
    } catch (error) {
      console.error("deposit failed ", error);
    }

    const vaultBalance = await getAccount(provider.connection, vaultAta);
    console.log(vaultBalance);

    assert.equal(
      vaultBalance.amount,
      100n,
      "vault balance should match the deposit amount"
    );
  });

  it("Withdraws from the vault", async () => {
    const transferAmount = new BN(100);
    try {
      program.methods
        .takeOffer(transferAmount)
        .accounts({
          taker,
          tokenProgram: TOKEN_PROGRAM_ID,
          maker: maker.publicKey,
          mintA,
          mintB,
          takerAtaA,
          takerAtaB,
          makerAtaB,
          escrowPDA,
          vaultAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([taker])
        .rpc();
    } catch (error) {
      console.error("withdrawal failed ", error);
    }
  });
});
