import { ethers } from "ethers";
import { Endpoint__factory } from "../abi";

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
