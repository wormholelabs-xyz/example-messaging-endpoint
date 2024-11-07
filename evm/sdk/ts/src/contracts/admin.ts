import { ethers } from "ethers";
import { Router__factory } from "../abi";

export async function getAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<string> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the getAdmin function
  const admin = await router["getAdmin"](integrator);

  return admin;
}

export async function getPendingAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<string> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the getPendingAdmin function
  const admin = await router["getPendingAdmin"](integrator);

  return admin;
}

export async function updateAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  admin: string,
): Promise<void> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the updateAdmin function
  await router["updateAdmin"](integrator, admin);
}

export async function transferAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  admin: string,
): Promise<void> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the transferAdmin function
  await router["transferAdmin"](integrator, admin);
}

export async function claimAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<void> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the claimAdmin function
  await router["claimAdmin"](integrator);
}

export async function discardAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<void> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the discardAdmin function
  await router["discardAdmin"](integrator);
}
