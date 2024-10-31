import { ethers } from "ethers";
import { getMessageStatus } from "../contracts/getMessageStatus";

(async () => {
  const provider = new ethers.JsonRpcProvider(
    "https://base-sepolia-rpc.publicnode.com	",
  );
  const contractAddress = "0x745CD07CF3EAc22Cf9c89af570120aaf98EC0493";
  // Use canned inputs to test the function
  const srcChain = 10002;
  const srcAddr =
    "0x000000000000000000000000894593bdf8999f32e4c95fb5f6d372f4365fc40f" as "0x${string}";
  const sequence = 2;
  const dstAddr =
    "0x0000000000000000000000001b5ba8b47e656afe522634ca7f058b2be33075af" as "0x${string}";
  const payloadHash =
    "0x29bf7021020ea89dbd91ef52022b5a654b55ed418c9e7aba71ef3b43a51669f2" as "0x${string}";
  const messageStatus = await getMessageStatus(
    contractAddress,
    provider,
    srcChain,
    srcAddr,
    sequence,
    dstAddr,
    payloadHash,
  );
  console.log("Enabled Bitmap:", messageStatus.enabledBitmap.toString());
  console.log("Attested Bitmap:", messageStatus.attestedBitmap.toString());
  console.log("Executed:", messageStatus.executed);
})();
