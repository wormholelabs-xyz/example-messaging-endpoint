import { ethers } from "ethers";
import { Endpoint__factory } from "../../../../../../evm/sdk/ts/src/abi";

export async function sendMessage(
  contractAddress: string,
  signer: ethers.Signer,
  dstChain: number,
  dstAddr: string,
  payloadHash: string,
  refundAddress: string,
): Promise<bigint> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, signer);

  // Call the getMessageStatus function and destructure the result
  const result: ethers.ContractTransactionResponse = await endpoint.sendMessage(
    dstChain,
    dstAddr,
    payloadHash,
    refundAddress,
  );
  await result.wait();

  return BigInt(result.data);
}

export async function attestMessage(
  contractAddress: string,
  signer: ethers.Signer,
  srcChain: number,
  srcAddr: string,
  sequence: number,
  dstChain: number,
  dstAddr: string,
  payloadHash: string,
): Promise<ethers.ContractTransactionResponse> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, signer);

  // Call the getMessageStatus function and destructure the result
  const result: ethers.ContractTransactionResponse =
    await endpoint.attestMessage(
      srcChain,
      srcAddr,
      sequence,
      dstChain,
      dstAddr,
      payloadHash,
    );

  return result;
}

export async function recvMessage(
  contractAddress: string,
  signer: ethers.Signer,
  srcChain: number,
  srcAddr: string,
  sequence: number,
  payloadHash: string,
): Promise<{
  enabledBitmap: bigint;
  attestedBitmap: bigint;
}> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, signer);

  // Call the getMessageStatus function and destructure the result
  const result: ethers.ContractTransactionResponse = await endpoint.recvMessage(
    srcChain,
    srcAddr,
    sequence,
    payloadHash,
  );
  await result.wait();
  const [enabledBitmap, attestedBitmap] = result.data;

  // Convert the values to bigint
  return {
    enabledBitmap: BigInt(enabledBitmap),
    attestedBitmap: BigInt(attestedBitmap),
  };
}

export async function getMessageStatus(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  srcChain: number,
  srcAddr: string,
  sequence: number,
  dstAddr: string,
  payloadHash: string,
): Promise<{
  enabledBitmap: ethers.BigNumberish;
  attestedBitmap: ethers.BigNumberish;
  executed: boolean;
}> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the getMessageStatus function and destructure the result
  const [enabledBitmap, attestedBitmap, executed] = await endpoint[
    "getMessageStatus(uint16,bytes32,uint64,bytes32,bytes32)"
  ](srcChain, srcAddr, sequence, dstAddr, payloadHash);

  // Return the structured response
  return { enabledBitmap, attestedBitmap, executed };
}
