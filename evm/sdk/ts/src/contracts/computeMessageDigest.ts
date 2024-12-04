import { ethers } from "ethers";
import { Endpoint__factory } from "../abi";

export async function computeMessageDigest(
  contractAddress: string,
  provider: ethers.Provider | ethers.Signer,
  srcChain: number,
  srcAddr: string,
  sequence: number,
  dstChain: number,
  dstAddr: string,
  payloadHash: string,
): Promise<string> {
  // Use Endpoint__factory to create a typed instance of the Endpoint contract
  const endpoint = Endpoint__factory.connect(contractAddress, provider);

  // Call the computeMessageDigest function
  const digest = await endpoint["computeMessageDigest"](
    srcChain,
    srcAddr,
    sequence,
    dstChain,
    dstAddr,
    payloadHash,
  );

  return digest;
}
