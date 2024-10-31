//
// Eventually, remove this file in favor of possible test file.
//
import { ethers } from "ethers";
import {
  addAdapter,
  getAdapterByIndex,
  getAdapterIndex,
  getAdapters,
} from "../contracts/Adapter";
import contractAbi from "../contractABI.json";
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
  const baseEndpoint = "0x27a8c4e91FF0C7E28934ec38610e28e7d7c47Ac2";
  const baseSepoliaIntegrator: string =
    "0xA4088dBe6C9B9e04B03dE7075406Fb751a569396";
  const baseSepoliaAdapter: string =
    "0xb076c20C1Bc2BC18c7e687DeF7686A87Ef4B9A3E";

  const sepoliaEndpoint = "0xdc007Dc26E4038317331E49536CF17845FC1cA11";
  const sepoliaIntegrator: string =
    "0xBE8810549d0e003D5da83004F0aE29c6fcDC30A6";
  const sepoliaAdapter: string = "0x1bB02c672C0A9E58DD2085E9bF68396C07Af0708";

  // console.log("*********************");
  // const iface = new ethers.Interface(contractAbi);
  // console.log(
  //   iface.parseTransaction({
  //     data: "0x6dacd3d7000000000000000000000000be8810549d0e003d5da83004f0ae29c6fcdc30a60000000000000000000000001bb02c672c0a9e58dd2085e9bf68396c07af0708",
  //   }),
  // );

  // console.log("Calling addAdapter...");
  // const index = await addAdapter(
  //   sepoliaEndpoint,
  //   privateKey,
  //   sepoliaIntegrator,
  //   sepoliaAdapter,
  // );
  // const index = await getAdapterIndex(
  //   sepoliaEndpoint,
  //   wallet,
  //   sepoliaIntegrator,
  //   sepoliaAdapter,
  // );
  // console.log("Index:", index);
  // const adapter = await getAdapterByIndex(
  //   sepoliaEndpoint,
  //   wallet,
  //   sepoliaIntegrator,
  //   0n,
  // );
  // console.log("adapter:", adapter);
  const adapters = await getAdapters(
    sepoliaEndpoint,
    wallet,
    sepoliaIntegrator,
  );
  console.log("Adapters:", adapters);
})();
