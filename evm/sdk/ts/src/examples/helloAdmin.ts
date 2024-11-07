import { ethers } from "ethers";
import { getAdmin } from "../contracts/admin";

(async () => {
  const baseProvider = new ethers.JsonRpcProvider(
    "https://base-sepolia-rpc.publicnode.com",
  );
  const sepoliaProvider = new ethers.JsonRpcProvider(
    "https://rpc-sepolia.rockx.com",
  );
  const contractAddress = "0x745CD07CF3EAc22Cf9c89af570120aaf98EC0493";
  const sepoliaRouter = "0x59d71dCf9355ad1328eF7DDd8E6f95Ef1e27AA82";
  const integrator = "0x1B5Ba8B47e656Afe522634ca7F058b2BE33075Af";
  const sepoliaIntegrator = "0x4A00c6716d77E473e3CD5ac8fDf0313022938256";
  // Use canned inputs to test the function
  const admin = await getAdmin(
    sepoliaRouter,
    sepoliaProvider,
    sepoliaIntegrator,
  );
  console.log("Admin:", admin);
})();
