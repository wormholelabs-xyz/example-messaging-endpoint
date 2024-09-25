import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Router } from "../target/types/router";

describe("router", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Router as Program<Router>;

  it("Is initialized!", async () => {
    // Add your test here.
    const owner = anchor.web3.Keypair.generate();
    const tx = await program.methods.initialize({ owner: owner.publicKey }).rpc();
    console.log("Your transaction signature", tx);
  });
});
