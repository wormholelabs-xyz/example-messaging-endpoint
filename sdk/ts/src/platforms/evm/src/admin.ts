import { ethers } from "ethers";
import { Endpoint__factory } from "../../../../../../evm/sdk/ts/src/abi";

export async function claimAdmin(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
): Promise<ethers.ContractTransactionResponse> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the claimAdmin function
  // const tx = await endpoint["claimAdmin"](integrator);
  const tx = await endpoint.claimAdmin(integrator);
  return tx;
}

export async function discardAdmin(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
): Promise<ethers.ContractTransactionResponse> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the discardAdmin function
  const tx = await endpoint.discardAdmin(integrator);
  return tx;
}

export async function getAdmin(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  integrator: string,
): Promise<string> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the getAdmin function
  const admin = await endpoint.getAdmin(integrator);

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
  const admin = await endpoint.getPendingAdmin(integrator);

  return admin;
}

export async function register(
  contractAddress: string,
  provider: ethers.Signer,
  initialAdmin: string,
): Promise<ethers.ContractTransactionResponse> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the register function
  const tx = await endpoint.register(initialAdmin);
  return tx;
}

export async function transferAdmin(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  admin: string,
): Promise<ethers.ContractTransactionResponse> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the transferAdmin function
  const tx = await endpoint.transferAdmin(integrator, admin);
  return tx;
}

export async function updateAdmin(
  contractAddress: string,
  provider: ethers.Signer,
  integrator: string,
  admin: string,
): Promise<ethers.ContractTransactionResponse> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the updateAdmin function
  const tx = await endpoint.updateAdmin(integrator, admin);
  return tx;
}
