import { jest, expect, test } from "@jest/globals";
import { ethers } from "ethers";
import {
  claimAdmin,
  getAdmin,
  getPendingAdmin,
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
const anvilEthIntegrator = "0xCfEB869F69431e42cdB54A4F4f105C19C080A601";
const transIntegrator = "0x3F31bEADe977BB848E159F5a443C0Bf59625CC36";
const transIntegrator1 = "0x8B0aB003F4e47f6b2A724E6C896B48a12f17caB1";
const transIntegrator2 = "0x7cE9CC35dD40E3169c6CeDB50E0F8a42E84586d5";
const transIntegrator3 = "0x50f8Dffe642803730B888c8A1320f9D7d32D25ac";

const tiAbi = [
  "function test() public",
  "function claimAdmin() public",
  "function discardAdmin() public",
  "function transferAdmin(address newAdmin) public",
  "function updateAdmin(address newAdmin) public",
  "function addAdapter(address adapterAddr) public",
  "function enableRecvAdapter(uint16 chain, address adapter) public",
  "function enableSendAdapter(uint16 chain, address adapter) public",
];

const routerAbi = [
  "function register(address integrator) public",
  "function claimAdmin(address integrator) public",
];

// Initialize the transparent integrator contract
const TIContract = new ethers.Contract(transIntegrator1, tiAbi, anvilEthSigner);
const RouterContract = new ethers.Contract(
  anvilEthRouter,
  routerAbi,
  anvilEthSigner,
);

// describe("EVM Admin Tests", () => {
//   describe("getAdmin", () => {
//     test("should return the admin", async () => {
//       const admin = await getAdmin(
//         anvilEthRouter,
//         anvilEthProvider,
//         transIntegrator1,
//       );
//       expect(admin).toBe(transIntegrator1);
//     });
//   });

describe("TS as Admin Tests", () => {
  // describe("TS program as admin", () => {
  //   test("register", async () => {
  //     // Set the signer as the admin on a register.
  //     await RouterContract.register(anvilEthSigner.address);
  //   });
  // });

  describe("cancel transfer", () => {
    test("getAdmin, getPendingAdmin, transferAdmin, claimAdmin", async () => {
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
