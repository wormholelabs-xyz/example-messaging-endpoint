import * as anchor from "@coral-xyz/anchor";
import { Endpoint } from "../../idls/endpoint";
import EndpointIdl from "../../idls/endpoint.json";
import {
  deriveIntegratorChainConfigPDA,
  deriveIntegratorConfigPDA,
} from "./helpers";

export async function addAdapter(
  integrator: string,
  adapter: string,
): Promise<bigint> {
  // Create all the public keys:
  const adapterPublicKey = new anchor.web3.PublicKey(adapter);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);

  // Create the endpoint program instance
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );

  // Use the integratorProgram object in your transaction
  const tx = await endpointProgram.methods
    .addAdapter({
      integratorProgramId: integratorPublicKey,
      adapterProgramId: adapterPublicKey,
    })
    .rpc();

  console.log("Transaction Signature:", tx);

  return 0n;
}

export async function disableRecvAdapter(
  integrator: string,
  adapter: string,
  chain: number,
): Promise<void> {
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const adapterPublicKey = new anchor.web3.PublicKey(adapter);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  await endpointProgram.methods
    .disableRecvAdapter({
      chainId: chain,
      adapterProgramId: adapterPublicKey,
      integratorProgramId: integratorPublicKey,
    })
    .rpc();
}

export async function disableSendAdapter(
  integrator: string,
  adapter: string,
  chain: number,
): Promise<void> {
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const adapterPublicKey = new anchor.web3.PublicKey(adapter);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  await endpointProgram.methods
    .disableSendAdapter({
      chainId: chain,
      adapterProgramId: adapterPublicKey,
      integratorProgramId: integratorPublicKey,
    })
    .rpc();
}

export async function enableRecvAdapter(
  integrator: string,
  adapter: string,
  chain: number,
): Promise<void> {
  // const endpointPublicKey = new anchor.web3.PublicKey(contractAddress);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const adapterPublicKey = new anchor.web3.PublicKey(adapter);
  const recvChainBuffer = Buffer.alloc(2);
  recvChainBuffer.writeUInt16BE(chain);
  // const [integratorRecvChainConfigPDA] =
  //   anchor.web3.PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from("integrator_chain_config"), // Constant seed
  //       integratorPublicKey.toBuffer(), // Integrator program ID
  //       recvChainBuffer, // Chain ID
  //     ],
  //     endpointPublicKey, // Program ID for the endpoint program
  //   );
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  await endpointProgram.methods
    .enableRecvAdapter({
      chainId: chain,
      adapterProgramId: adapterPublicKey,
      integratorProgramId: integratorPublicKey,
    })
    // .accountsPartial({ integratorChainConfig: integratorRecvChainConfigPDA })
    .rpc();
}

export async function enableSendAdapter(
  integrator: string,
  adapter: string,
  chain: number,
): Promise<void> {
  // const endpointPublicKey = new anchor.web3.PublicKey(contractAddress);
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const adapterPublicKey = new anchor.web3.PublicKey(adapter);
  const recvChainBuffer = Buffer.alloc(2);
  recvChainBuffer.writeUInt16BE(chain);
  // const [integratorRecvChainConfigPDA] =
  //   anchor.web3.PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from("integrator_chain_config"), // Constant seed
  //       integratorPublicKey.toBuffer(), // Integrator program ID
  //       recvChainBuffer, // Chain ID
  //     ],
  //     endpointPublicKey, // Program ID for the endpoint program
  //   );
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  await endpointProgram.methods
    .enableSendAdapter({
      chainId: chain,
      adapterProgramId: adapterPublicKey,
      integratorProgramId: integratorPublicKey,
    })
    // .accountsPartial({ integratorChainConfig: integratorRecvChainConfigPDA })
    .rpc();
}

export async function getAdapterByIndex(
  integrator: string,
  index: bigint,
): Promise<string> {
  const adapters = await getAdapters(integrator);
  if (index >= BigInt(adapters.length)) {
    throw new Error("Index out of bounds");
  }
  return adapters[Number(index)];
}

export async function getAdapterIndex(
  integrator: string,
  adapter: string,
): Promise<bigint> {
  const adapters = await getAdapters(integrator);
  const index = adapters.indexOf(adapter);
  if (index < 0) {
    throw new Error("Adapter not found");
  }
  return BigInt(index);
}

export async function getAdapters(integrator: string): Promise<string[]> {
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  const endpointPublicKey = new anchor.web3.PublicKey(EndpointIdl.address);
  const integratorConfigPDA = await deriveIntegratorConfigPDA(
    integratorPublicKey,
    endpointPublicKey,
  );
  const integratorConfig = integratorConfigPDA[0];
  const result =
    await endpointProgram.account.integratorConfig.fetch(integratorConfig);

  return result.adapterInfos.map((key) => key.toBase58()); // Convert each PublicKey to a Base58 string
}

export async function getRecvAdaptersByChain(
  integrator: string,
  chain: number,
): Promise<string[]> {
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  const [integratorChainConfigPDA] = await deriveIntegratorChainConfigPDA(
    integratorPublicKey, // Integrator program ID
    endpointProgram.programId, // Program ID for the endpoint program
    chain,
  );
  const result = await endpointProgram.account.integratorChainConfig.fetch(
    integratorChainConfigPDA,
  );
  const bitmap = result.recvAdapterBitmap.map.toNumber();
  const indices = getSetBitIndices(bitmap);
  const allAdapters = await getAdapters(integrator);
  const adapters: string[] = [];
  for (const index of indices) {
    adapters.push(allAdapters[index]);
  }
  return adapters;
}

export async function getSendAdaptersByChain(
  integrator: string,
  chain: number,
): Promise<string[]> {
  const integratorPublicKey = new anchor.web3.PublicKey(integrator);
  const endpointProgram = new anchor.Program<Endpoint>(
    EndpointIdl as Endpoint,
    anchor.AnchorProvider.env(),
  );
  const [integratorChainConfigPDA] = await deriveIntegratorChainConfigPDA(
    integratorPublicKey, // Integrator program ID
    endpointProgram.programId, // Program ID for the endpoint program
    chain,
  );
  const result = await endpointProgram.account.integratorChainConfig.fetch(
    integratorChainConfigPDA,
  );
  const bitmap = result.sendAdapterBitmap.map.toNumber();
  const indices = getSetBitIndices(bitmap);
  const allAdapters = await getAdapters(integrator);
  const adapters: string[] = [];
  for (const index of indices) {
    adapters.push(allAdapters[index]);
  }
  return adapters;
}

function getSetBitIndices(bitmap: number): number[] {
  const indices: number[] = [];
  let index = 0;

  while (bitmap !== 0) {
    // Check if the least significant bit is set
    if ((bitmap & 1) === 1) {
      indices.push(index);
    }
    // Shift the bitmap to the right by 1 bit
    bitmap >>= 1;
    index++;
  }

  return indices;
}
