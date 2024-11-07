import { ethers } from "ethers";
import { Router__factory } from "../abi";

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
  // Use Router__factory to create a typed instance of the Router contract
  const router = Router__factory.connect(contractAddress, provider);

  // Call the computeMessageDigest function
  const digest = await router["computeMessageDigest"](
    srcChain,
    srcAddr,
    sequence,
    dstChain,
    dstAddr,
    payloadHash,
  );

  return digest;
}
