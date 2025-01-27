import { ethers } from "ethers";
import { Endpoint__factory } from "../abi";

export async function sendMessage(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  dstChain: number,
  dstAddr: string,
  payloadHash: string,
  refundAddress: string,
): Promise<bigint> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the getMessageStatus function and destructure the result
  const result: ethers.ContractTransactionResponse = await endpoint[
    "sendMessage"
  ](dstChain, dstAddr, payloadHash, refundAddress);

  return BigInt(result.data);
}

export async function attestMessage(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  srcChain: number,
  srcAddr: string,
  sequence: number,
  dstChain: number,
  dstAddr: string,
  payloadHash: string,
): Promise<void> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the getMessageStatus function and destructure the result
  await endpoint["attestMessage"](
    srcChain,
    srcAddr,
    sequence,
    dstChain,
    dstAddr,
    payloadHash,
  );
}

export async function recvMessage(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  srcChain: number,
  srcAddr: string,
  sequence: number,
  payloadHash: string,
): Promise<{
  enabledBitmap: bigint;
  attestedBitmap: bigint;
}> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the getMessageStatus function and destructure the result
  const result = await endpoint["recvMessage"](
    srcChain,
    srcAddr,
    sequence,
    payloadHash,
  );
  const [enabledBitmap, attestedBitmap] = result.data;

  // Convert the values to bigint
  return {
    enabledBitmap: BigInt(enabledBitmap),
    attestedBitmap: BigInt(attestedBitmap),
  };
}
