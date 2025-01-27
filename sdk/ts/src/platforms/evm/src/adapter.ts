import { ethers } from "ethers";
import { Endpoint__factory } from "../../../../../../evm/sdk/ts/src/abi";

export async function addAdapter(
  endpointAddr: string,
  signer: ethers.Wallet,
  integrator: string,
  adapter: string,
): Promise<bigint> {
  console.log("Contract Address:", endpointAddr);
  console.log("Integrator:", integrator);
  console.log("Adapter:", adapter);

  if (!ethers.isAddress(integrator) || !ethers.isAddress(adapter)) {
    throw new Error("Invalid address format for integrator or adapter.");
  }

  const endpoint = Endpoint__factory.connect(endpointAddr, signer);
  console.log("Endpoint:", endpoint);

  const result: ethers.ContractTransactionResponse = await endpoint.addAdapter(
    integrator,
    adapter,
  );
  await result.wait();
  console.log("result:", result);

  return BigInt(result.value);
}

export async function disableRecvAdapter(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  adapter: string,
): Promise<ethers.ContractTransactionResponse> {
  if (!ethers.isAddress(integrator) || !ethers.isAddress(adapter)) {
    throw new Error("Invalid address format for integrator or adapter.");
  }
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const result: ethers.ContractTransactionResponse =
    await endpoint.disableRecvAdapter(integrator, chain, adapter);
  return result;
}

export async function disableSendAdapter(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  adapter: string,
): Promise<ethers.ContractTransactionResponse> {
  if (!ethers.isAddress(integrator) || !ethers.isAddress(adapter)) {
    throw new Error("Invalid address format for integrator or adapter.");
  }
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const result: ethers.ContractTransactionResponse =
    await endpoint.disableSendAdapter(integrator, chain, adapter);
  return result;
}

export async function enableRecvAdapter(
  endpointAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  adapter: string,
): Promise<ethers.ContractTransactionResponse> {
  if (!ethers.isAddress(integrator) || !ethers.isAddress(adapter)) {
    throw new Error("Invalid address format for integrator or adapter.");
  }
  const endpoint = Endpoint__factory.connect(endpointAddress, provider);

  const result: ethers.ContractTransactionResponse =
    await endpoint.enableRecvAdapter(integrator, chain, adapter);
  return result;
}

export async function enableSendAdapter(
  endpointAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  adapter: string,
): Promise<ethers.ContractTransactionResponse> {
  if (!ethers.isAddress(integrator) || !ethers.isAddress(adapter)) {
    throw new Error("Invalid address format for integrator or adapter.");
  }
  const endpoint = Endpoint__factory.connect(endpointAddress, provider);

  const result: ethers.ContractTransactionResponse =
    await endpoint.enableSendAdapter(integrator, chain, adapter);
  return result;
}

export async function getAdapterByIndex(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  index: bigint,
): Promise<string> {
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const adapter = await endpoint.getAdapterByIndex(integrator, index);

  return adapter;
}

export async function getAdapterIndex(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  adapter: string,
): Promise<bigint> {
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const index = await endpoint.getAdapterIndex(integrator, adapter);

  return BigInt(index);
}

export async function getAdapters(
  endpointAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<string[]> {
  if (!ethers.isAddress(integrator)) {
    throw new Error("Invalid address format for integrator.");
  }

  const endpoint = Endpoint__factory.connect(endpointAddress, provider);
  const adapters = await endpoint.getAdapters(integrator);

  return adapters;
}

export async function getRecvAdaptersByChain(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  chain: number,
): Promise<string[]> {
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const adapters = await endpoint.getRecvAdaptersByChain(integrator, chain);

  return adapters;
}

export async function getSendAdaptersByChain(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  chain: number,
): Promise<string[]> {
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  const adapters = await endpoint.getSendAdaptersByChain(integrator, chain);

  return adapters;
}
