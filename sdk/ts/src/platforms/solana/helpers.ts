import * as anchor from "@coral-xyz/anchor";

export async function deriveIntegratorConfigPDA(
  integratorProgramId: anchor.web3.PublicKey,
  programId: anchor.web3.PublicKey,
): Promise<[anchor.web3.PublicKey, number]> {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("integrator_config"), integratorProgramId.toBuffer()],
    programId,
  );
}

export async function deriveIntegratorChainConfigPDA(
  integratorProgramId: anchor.web3.PublicKey,
  programId: anchor.web3.PublicKey,
  chain: number,
): Promise<[anchor.web3.PublicKey, number]> {
  const chainBuffer = Buffer.alloc(2);
  chainBuffer.writeUInt16BE(chain);
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("integrator_chain_config"), // Constant seed
      integratorProgramId.toBuffer(), // Integrator program ID
      chainBuffer, // Chain ID
    ],
    programId, // Program ID for the endpoint program
  );
}
