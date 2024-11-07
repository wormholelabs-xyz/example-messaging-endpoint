import { ethers } from "ethers";
import { Router__factory } from "../abi";

export async function addTransceiver(
  contractAddress: string,
  signer: ethers.Signer,
  integrator: string,
  transceiver: string,
): Promise<bigint> {
  console.log("Contract Address:", contractAddress);
  console.log("Signer:", signer);
  console.log("Integrator:", integrator);
  console.log("Transceiver:", transceiver);

  if (!ethers.isAddress(integrator) || !ethers.isAddress(transceiver)) {
    throw new Error("Invalid address format for integrator or transceiver.");
  }
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, signer);
  console.log("Router:", router);

  const result: ethers.ContractTransactionResponse = await router[
    "addTransceiver"
  ](integrator, transceiver);

  return BigInt(result.index);
}

export async function enableSendTransceiver(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  transceiver: string,
): Promise<void> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  await router["enableSendTransceiver"](integrator, chain, transceiver);
}

export async function enableRecvTransceiver(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  transceiver: string,
): Promise<void> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  await router["enableRecvTransceiver"](integrator, chain, transceiver);
}

export async function disableSendTransceiver(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  transceiver: string,
): Promise<void> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  await router["disableSendTransceiver"](integrator, chain, transceiver);
}

export async function disableRecvTransceiver(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  chain: number,
  transceiver: string,
): Promise<void> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  await router["disableRecvTransceiver"](integrator, chain, transceiver);
}

export async function getTransceiverIndex(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  transceiver: string,
): Promise<bigint> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  const index = await router["getTransceiverIndex"](integrator, transceiver);

  return BigInt(index);
}

export async function getSendTransceiversByChain(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  chain: number,
): Promise<string[]> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  const transceivers = await router["getSendTransceiversByChain"](
    integrator,
    chain,
  );

  return transceivers;
}

export async function getRecvTransceiversByChain(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  chain: number,
): Promise<string[]> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  const transceivers = await router["getRecvTransceiversByChain"](
    integrator,
    chain,
  );

  return transceivers;
}

export async function getTransceivers(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<string[]> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  const transceivers = await router["getTransceivers"](integrator);

  return transceivers;
}

export async function getTransceiverByIndex(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  index: bigint,
): Promise<string> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  const transceiver = await router["getTransceiverByIndex"](integrator, index);

  return transceiver;
}
