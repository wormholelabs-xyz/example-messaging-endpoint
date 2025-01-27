//
// Eventually remove this file in favor sdk/ts/src/platforms/evm.ts
//
import { ethers } from "ethers";
import { Endpoint__factory } from "../abi";

export async function addAdapter(
  contractAddress: string,
  walletPrivateKey: string,
  integrator: string,
  adapter: string,
): Promise<bigint> {
  // TODO: Define this somewhere else
  const sepoliaProvider = new ethers.JsonRpcProvider(
    "https://rpc-sepolia.rockx.com",
  );

  // Create wallet from private key
  const wallet = new ethers.Wallet(walletPrivateKey).connect(sepoliaProvider);

  console.log("Contract Address:", contractAddress);
  console.log("Integrator:", integrator);
  console.log("Adapter:", adapter);
  // console.log("Wallet:", wallet);

  if (!ethers.isAddress(integrator) || !ethers.isAddress(adapter)) {
    throw new Error("Invalid address format for integrator or adapter.");
  }
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, wallet);
  console.log("Endpoint:", endpoint);

  const result: ethers.ContractTransactionResponse = await endpoint[
    "addAdapter"
  ](integrator, adapter);

  return BigInt(result.index);
}

export async function enableSendAdapter(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  adapter: string,
): Promise<void> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  await endpoint["enableSendAdapter"](integrator, chain, adapter);
}

export async function enableRecvAdapter(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  adapter: string,
): Promise<void> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  await endpoint["enableRecvAdapter"](integrator, chain, adapter);
}

export async function disableSendAdapter(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  adapter: string,
): Promise<void> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  await endpoint["disableSendAdapter"](integrator, chain, adapter);
}

export async function disableRecvAdapter(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  adapter: string,
): Promise<void> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  await endpoint["disableRecvAdapter"](integrator, chain, adapter);
}

export async function getAdapterIndex(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  adapter: string,
): Promise<bigint> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const index = await endpoint["getAdapterIndex"](integrator, adapter);

  return BigInt(index);
}

export async function getSendAdaptersByChain(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  chain: number,
): Promise<string[]> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const adapters = await endpoint["getSendAdaptersByChain"](integrator, chain);

  return adapters;
}

export async function getRecvAdaptersByChain(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  chain: number,
): Promise<string[]> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const adapters = await endpoint["getRecvAdaptersByChain"](integrator, chain);

  return adapters;
}

export async function getAdapters(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<string[]> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const adapters = await endpoint["getAdapters"](integrator);

  return adapters;
}

export async function getAdapterByIndex(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  index: bigint,
): Promise<string> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const adapter = await endpoint["getAdapterByIndex"](integrator, index);

  return adapter;
}
