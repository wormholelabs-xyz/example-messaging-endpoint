import { ethers } from "ethers";
import {
  addAdapter,
  getAdapterByIndex,
  getAdapterIndex,
} from "../contracts/Adapter";
import * as dotenv from "dotenv";
dotenv.config();

(async () => {
  const baseProvider = new ethers.JsonRpcProvider(
    "https://base-sepolia-rpc.publicnode.com",
  );
  const sepoliaProvider = new ethers.JsonRpcProvider(
    "https://rpc-sepolia.rockx.com",
  );
  const privateKey = process.env.PRIVATE_KEY;
  if (!privateKey) {
    throw new Error("PRIVATE_KEY is not set");
  }
  // const wallet = new ethers.Wallet(privateKey, provider);
  const wallet = new ethers.Wallet(privateKey).connect(sepoliaProvider);
  console.log("Wallet Address:", wallet.address);
  const baseRouter = "0x745CD07CF3EAc22Cf9c89af570120aaf98EC0493";
  const baseSepoliaIntegrator: string =
    "0x24C9eA36b1b507D9113332e3E2bA158353153074";
  const baseSepoliaAdapter: string =
    "0xD19bB37fb212D5799895725D1858Fa4Ab2fcA1A7";

  const sepoliaRouter = "0x59d71dCf9355ad1328eF7DDd8E6f95Ef1e27AA82";
  const sepoliaIntegrator: string =
    "0x4A00c6716d77E473e3CD5ac8fDf0313022938256";
  const sepoliaAdapter: string = "0x5Fa768AF5994995cE2D3FF7F300E83855107cF0d";

  // const index = await addAdapter(sepoliaRouter, wallet, sepoliaIntegrator, sepoliaAdapter);
  // const index = await getAdapterIndex(sepoliaRouter, wallet, sepoliaIntegrator, sepoliaAdapter);
  const trans = await getAdapterByIndex(
    sepoliaRouter,
    wallet,
    sepoliaIntegrator,
    0n,
  );
  console.log("Trans:", trans);
  // console.log("Index:", index);
})();
