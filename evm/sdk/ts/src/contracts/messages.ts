import { ethers } from "ethers";
import { Router__factory } from "../abi";

export async function sendMessage(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  dstChain: number,
  dstAddr: string,
  payloadHash: string,
  refundAddress: string,
): Promise<bigint> {
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the getMessageStatus function and destructure the result
  const result: ethers.ContractTransactionResponse = await router[
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
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the getMessageStatus function and destructure the result
  await router["attestMessage"](
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
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the getMessageStatus function and destructure the result
  const result = await router["recvMessage"](
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
