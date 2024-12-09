import { ethers } from "ethers";
import { Endpoint__factory } from "../abi";

export async function claimAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<void> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the claimAdmin function
  await endpoint["claimAdmin"](integrator);
}

export async function discardAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<void> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the discardAdmin function
  await endpoint["discardAdmin"](integrator);
}

export async function getAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<string> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the getAdmin function
  const admin = await endpoint["getAdmin"](integrator);

  return admin;
}

export async function getPendingAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<string> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the getPendingAdmin function
  const admin = await endpoint["getPendingAdmin"](integrator);

  return admin;
}

export async function transferAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  admin: string,
): Promise<void> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the transferAdmin function
  await endpoint["transferAdmin"](integrator, admin);
}

export async function updateAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
  admin: string,
): Promise<void> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the updateAdmin function
  await endpoint["updateAdmin"](integrator, admin);
}
