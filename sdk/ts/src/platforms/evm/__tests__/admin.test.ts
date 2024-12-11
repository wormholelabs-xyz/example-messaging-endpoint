import { jest, expect, test } from "@jest/globals";
import { ethers } from "ethers";
import {
  claimAdmin,
  getAdmin,
  getPendingAdmin,
  register,
  transferAdmin,
} from "../src/admin";

jest.setTimeout(180000);

const sepoliaProvider = new ethers.JsonRpcProvider(
  "https://rpc-sepolia.rockx.com",
);
const sepoliaRouter = "0xdc007Dc26E4038317331E49536CF17845FC1cA11";
const sepoliaIntegrator = "0xBE8810549d0e003D5da83004F0aE29c6fcDC30A6";
const fakeIntegrator = "0x1B5Ba8B47e656Afe522634ca7F058b2BE33075Af";
const nullAdmin = "0x0000000000000000000000000000000000000000";

const anvilPrivateKey =
  "0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d";
const anvilEthProvider = new ethers.JsonRpcProvider("http://127.0.0.1:8545");
const anvilEthSigner = new ethers.Wallet(anvilPrivateKey, anvilEthProvider);
const anvilEthRouter = "0x8186Eaa8CE62Bb3a1a72DA4B103D98AFff417B4A";

const routerAbi = [
  "function register(address integrator) public",
  "function claimAdmin(address integrator) public",
];

// Initialize the local router contract.
const RouterContract = new ethers.Contract(
  anvilEthRouter,
  routerAbi,
  anvilEthSigner,
);

describe("TS as Admin Tests", () => {
  // Can only be run once per Anvil deployment.

  describe("TS program as admin", () => {
    test("register", async () => {
      const firstNonce = await anvilEthProvider.getTransactionCount(
        anvilEthSigner.address,
      );
      console.log("Nonce check before register:", firstNonce);
      if (firstNonce <= 2) {
        // Set the signer as the admin on a register.
        console.log("Registering the signer as the admin.");
        // await RouterContract.register(anvilEthSigner.address);
        const registerTx = await register(
          anvilEthRouter,
          anvilEthSigner,
          anvilEthSigner.address,
        );
        await registerTx.wait();
      } else {
        console.log("Contract is already likely registered.");
      }
    });
  });

  describe("cancel transfer", () => {
    test("getAdmin, getPendingAdmin, transferAdmin, claimAdmin", async () => {
      const firstNonce = await anvilEthProvider.getTransactionCount(
        anvilEthSigner.address,
      );
      console.log("Nonce before transferAdmin:", firstNonce);
      let admin = await getAdmin(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      let pendingAdmin = await getPendingAdmin(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      console.log("Admin:", admin);
      console.log("Pending Admin:", pendingAdmin);
      expect(admin).toBe(anvilEthSigner.address);
      expect(pendingAdmin).toBe(nullAdmin);

      let nonce = await anvilEthProvider.getTransactionCount(
        anvilEthSigner.address,
      );
      console.log("Nonce before transferAdmin:", nonce);
      const transferTx = await transferAdmin(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        fakeIntegrator,
      );
      await transferTx.wait();
      nonce = await anvilEthProvider.getTransactionCount(
        anvilEthSigner.address,
      );
      console.log("Nonce after transferAdmin:", nonce);
      admin = await getAdmin(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      pendingAdmin = await getPendingAdmin(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      expect(admin).toBe(anvilEthSigner.address);
      expect(pendingAdmin).toBe(fakeIntegrator);
      const claimTx = await claimAdmin(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      await claimTx.wait();
      nonce = await anvilEthProvider.getTransactionCount(
        anvilEthSigner.address,
      );
      console.log("Nonce after getAdmin:", nonce);

      admin = await getAdmin(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      pendingAdmin = await getPendingAdmin(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      expect(admin).toBe(anvilEthSigner.address);
      expect(pendingAdmin).toBe(nullAdmin);
    });
  });
});
