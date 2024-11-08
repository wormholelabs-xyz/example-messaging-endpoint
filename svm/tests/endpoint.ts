import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Endpoint } from "../target/types/endpoint";
import EndpointIdl from "../target/idl/endpoint.json";
import { MockIntegrator } from "../target/types/mock_integrator";

import {
  PublicKey,
  Keypair,
  TransactionInstruction,
  Connection,
  Signer,
  TransactionMessage,
  VersionedTransaction,
  AddressLookupTableAccount,
  ConfirmOptions,
} from "@solana/web3.js";
import { expect } from "chai";
import { Err, Ok } from "ts-results";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { workspace } from "@coral-xyz/anchor";

// Modified from `example-liquidity-layer` to use async/await instead of Err and Ok from `ts-results``
// Link: https://github.com/wormhole-foundation/example-liquidity-layer/blob/main/solana/ts/src/testing/utils.ts#L39
export async function expectIxOk(
  connection: Connection,
  instructions: TransactionInstruction[],
  signers: Signer[],
  options: {
    addressLookupTableAccounts?: AddressLookupTableAccount[];
    confirmOptions?: ConfirmOptions;
  } = {},
) {
  const { addressLookupTableAccounts, confirmOptions } = options;
  return await debugSendAndConfirmTransaction(
    connection,
    instructions,
    signers,
    {
      addressLookupTableAccounts,
      logError: true,
      confirmOptions,
    },
  );
}

async function debugSendAndConfirmTransaction(
  connection: Connection,
  instructions: TransactionInstruction[],
  signers: Signer[],
  options: {
    addressLookupTableAccounts?: AddressLookupTableAccount[];
    logError?: boolean;
    confirmOptions?: ConfirmOptions;
  } = {},
): Promise<string> {
  const { logError, confirmOptions, addressLookupTableAccounts } = options;

  try {
    const latestBlockhash = await connection.getLatestBlockhash();

    const messageV0 = new TransactionMessage({
      payerKey: signers[0].publicKey,
      recentBlockhash: latestBlockhash.blockhash,
      instructions,
    }).compileToV0Message(addressLookupTableAccounts);

    const tx = new VersionedTransaction(messageV0);
    tx.sign(signers);

    const signature = await connection.sendTransaction(tx, confirmOptions);
    await connection.confirmTransaction(
      {
        signature,
        ...latestBlockhash,
      },
      confirmOptions === undefined ? "confirmed" : confirmOptions.commitment,
    );

    return signature;
  } catch (err) {
    if (logError) {
      console.log(err);
    }
    if (err.logs !== undefined) {
      throw new Error(err.logs.join("\n"));
    } else {
      throw new Error(err.message);
    }
  }
}

describe("endpoint", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = workspace.Endpoint as Program<Endpoint>;
  const mockIntegratorProgram =
    workspace.MockIntegrator as Program<MockIntegrator>;

  it("registers an integrator", async () => {
    const payer = anchor.web3.Keypair.generate();
    const admin = anchor.web3.Keypair.generate();

    // Airdrop some SOL to the payer
    await anchor
      .getProvider()
      .connection.requestAirdrop(
        payer.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL,
      );
    await new Promise((resolve) => setTimeout(resolve, 1000));
    // Check the balance of the payer account
    const payerBalance = await anchor
      .getProvider()
      .connection.getBalance(payer.publicKey);
    console.log(
      `Payer balance: ${payerBalance / anchor.web3.LAMPORTS_PER_SOL} SOL`,
    );

    // Derive PDAs
    const [integratorConfig] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("integrator_config"),
        mockIntegratorProgram.programId.toBuffer(),
      ],
      program.programId,
    );

    const [sequenceTracker] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("sequence_tracker"),
        mockIntegratorProgram.programId.toBuffer(),
      ],
      program.programId,
    );

    const [integratorProgramPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("endpoint_integrator")],
      mockIntegratorProgram.programId,
    );

    const eventAuthority = PublicKey.findProgramAddressSync(
      [Buffer.from("__event_authority")],
      program.programId,
    )[0];

    // Prepare the instruction
    const ix = await mockIntegratorProgram.methods
      .invokeRegister({
        admin: admin.publicKey,
      })
      .accounts({
        payer: payer.publicKey,
        integratorConfig,
        sequenceTracker,
        program: program.programId,
      })
      .accountsPartial({
        integratorProgramPda,
        eventAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        endpointProgram: program.programId,
      })
      .instruction();

    // Send and confirm the transaction
    const tx = await expectIxOk(
      anchor.getProvider().connection,
      [ix],
      [payer],
      { confirmOptions: { commitment: "confirmed" } },
    );
    console.log("Transaction signature:", tx);

    // Fetch and verify the IntegratorConfig account
    const integratorConfigAccount =
      await program.account.integratorConfig.fetch(integratorConfig);
    expect(integratorConfigAccount.admin.toString()).to.equal(
      admin.publicKey.toString(),
    );
    expect(integratorConfigAccount.integratorProgramId.toString()).to.equal(
      mockIntegratorProgram.programId.toString(),
    );
    expect(integratorConfigAccount.adapterInfos).to.be.empty;

    // Fetch and verify the SequenceTracker account
    const sequenceTrackerAccount =
      await program.account.sequenceTracker.fetch(sequenceTracker);
    expect(sequenceTrackerAccount.integratorProgramId.toString()).to.equal(
      mockIntegratorProgram.programId.toString(),
    );
    expect(sequenceTrackerAccount.sequence.toNumber()).to.equal(0);

    // Fetch the transaction details
    const txDetails = await anchor.getProvider().connection.getTransaction(tx, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });

    expect(txDetails).to.not.be.null;

    const borshEventCoder = new anchor.BorshEventCoder(EndpointIdl as any);

    // Get the third instruction (index 3) from the inner instructions
    const thirdInstruction =
      txDetails.meta.innerInstructions[0].instructions[3];

    // Decode the Base58 encoded data
    const decodedData = bs58.decode(thirdInstruction.data);

    // Remove the instruction discriminator and re-encode the rest as Base58
    const eventData = Buffer.from(decodedData.slice(8)).toString("base64");

    const borshEvents = borshEventCoder.decode(eventData);

    expect(borshEvents).to.deep.equal({
      data: {
        integrator: mockIntegratorProgram.programId,
        admin: admin.publicKey,
      },
      name: "IntegratorRegistered",
    });
  });
});
