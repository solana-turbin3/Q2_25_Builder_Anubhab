import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";
import { assert } from "chai";

describe("vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.vault as Program<Vault>;

  const signer = provider.wallet;
  let vaultPda: anchor.web3.PublicKey;
  let vaultBump: number;

  const SYSTEM_PROGRAM_ID = anchor.web3.SystemProgram.programId;

  before(async () => {
    [vaultPda, vaultBump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), signer.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Performs deposit and withdraw correctly", async () => {
    const depositAmount = 1_000_000; // 0.001 SOL

    const vaultBefore = await provider.connection.getBalance(vaultPda);
    const userBefore = await provider.connection.getBalance(signer.publicKey);

    // Deposit
    await program.methods
      .deposit(new anchor.BN(depositAmount))
      .accounts({
        signer: signer.publicKey,
        vault: vaultPda,
        systemAccount: SYSTEM_PROGRAM_ID,
      })
      .signers([])
      .rpc();

    const vaultAfterDeposit = await provider.connection.getBalance(vaultPda);
    const userAfterDeposit = await provider.connection.getBalance(
      signer.publicKey
    );

    assert.ok(
      vaultAfterDeposit - vaultBefore === depositAmount,
      "Vault should receive the deposit"
    );
    assert.ok(
      userBefore - userAfterDeposit >= depositAmount,
      "User should be debited"
    );

    // Withdraw
    await program.methods
      .withdraw()
      .accounts({
        signer: signer.publicKey,
        vault: vaultPda,
        systemAccount: SYSTEM_PROGRAM_ID,
      })
      .signers([])
      .rpc();

    const vaultAfterWithdraw = await provider.connection.getBalance(vaultPda);
    const userAfterWithdraw = await provider.connection.getBalance(
      signer.publicKey
    );

    assert.equal(vaultAfterWithdraw, 0, "Vault should be empty after withdraw");
    assert.ok(
      userAfterWithdraw > userAfterDeposit,
      "User should receive lamports back"
    );
  });
});
