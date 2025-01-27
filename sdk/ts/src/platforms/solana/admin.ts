import * as anchor from "@coral-xyz/anchor";
import { Endpoint } from "../../idls/endpoint";
import EndpointIdl from "../../idls/endpoint.json";
import { deriveIntegratorConfigPDA } from "./helpers";

export async function claimAdmin(
  integrator: string,
  walletPrivateKey: string,
): Promise<void> {
  const endpointPublicKey = new anchor.web3.PublicKey(EndpointIdl.address);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const integratorConfigPDA = await deriveIntegratorConfigPDA(
    integratorPublicKey,
    endpointPublicKey,
  );
  const integratorConfig = integratorConfigPDA[0];
  const signerKeyBuffer: Buffer = Buffer.from(walletPrivateKey, "hex");
  const signerKeyArray: Uint8Array = new Uint8Array(signerKeyBuffer);
  const signerKey = anchor.web3.Keypair.fromSecretKey(signerKeyArray);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  await endpointProgram.methods
    .claimAdmin()
    .accounts({
      newAdmin: integratorPublicKey,
      integratorConfig,
    })
    .signers([signerKey])
    .rpc();
}

// This function is extra in case we need to have one function to cancel the pending admin
// and another to claim the admin.
export async function cancelPendingAdmin(integrator: string): Promise<void> {
  const endpointPublicKey = new anchor.web3.PublicKey(EndpointIdl.address);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const integratorConfigPDA = await deriveIntegratorConfigPDA(
    integratorPublicKey,
    endpointPublicKey,
  );
  const integratorConfig = integratorConfigPDA[0];
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  await endpointProgram.methods
    .claimAdmin()
    .accounts({
      integratorConfig,
    })
    .rpc();
}

export async function discardAdmin(integrator: string): Promise<void> {
  const endpointPublicKey = new anchor.web3.PublicKey(EndpointIdl.address);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  const [integratorConfigPDA] = await deriveIntegratorConfigPDA(
    integratorPublicKey,
    endpointPublicKey,
  );
  await endpointProgram.methods
    .discardAdmin()
    .accountsPartial({ integratorConfig: integratorConfigPDA })
    .rpc();
}

export async function getAdmin(integrator: string): Promise<string> {
  const endpointPublicKey = new anchor.web3.PublicKey(EndpointIdl.address);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  const integratorConfigPDA = await deriveIntegratorConfigPDA(
    integratorPublicKey,
    endpointPublicKey,
  );
  const integratorConfig = integratorConfigPDA[0];
  const result =
    await endpointProgram.account.integratorConfig.fetch(integratorConfig);
  if (!result.admin) {
    throw new Error("No admin found");
  }
  return result.admin.toBase58();
}

export async function getPendingAdmin(integrator: string): Promise<string> {
  const endpointPublicKey = new anchor.web3.PublicKey(EndpointIdl.address);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  const integratorConfigPDA = await deriveIntegratorConfigPDA(
    integratorPublicKey,
    endpointPublicKey,
  );
  const integratorConfig = integratorConfigPDA[0];
  const result =
    await endpointProgram.account.integratorConfig.fetch(integratorConfig);
  if (!result.pendingAdmin) {
    throw new Error("No admin found");
  }
  return result.pendingAdmin.toBase58();
}

export async function transferAdmin(
  integrator: string,
  admin: string,
): Promise<void> {
  const newAdmin = new anchor.web3.PublicKey(admin);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  const endpointPublicKey = new anchor.web3.PublicKey(EndpointIdl.address);

  const [integratorConfigPDA] = await deriveIntegratorConfigPDA(
    integratorPublicKey,
    endpointPublicKey,
  );
  await endpointProgram.methods
    .transferAdmin({
      newAdmin: newAdmin,
      integratorProgramId: integratorPublicKey,
    })
    .rpc();
}

export async function updateAdmin(
  integrator: string,
  admin: string,
): Promise<void> {
  const newAdmin = new anchor.web3.PublicKey(admin);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );

  await endpointProgram.methods
    .updateAdmin({
      newAdmin: newAdmin,
      integratorProgramId: integratorPublicKey,
    })
    .rpc();
}
