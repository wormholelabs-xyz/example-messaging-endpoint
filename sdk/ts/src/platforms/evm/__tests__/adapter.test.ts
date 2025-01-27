import { jest, expect, test } from "@jest/globals";
import { ethers } from "ethers";
import {
  addAdapter,
  disableRecvAdapter,
  disableSendAdapter,
  enableRecvAdapter,
  enableSendAdapter,
  getAdapterByIndex,
  getAdapterIndex,
  getAdapters,
  getRecvAdaptersByChain,
  getSendAdaptersByChain,
} from "../src/adapter";

jest.setTimeout(180000);

const anvilPrivateKey =
  "0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d";
const anvilEthProvider = new ethers.JsonRpcProvider("http://127.0.0.1:8545");
const anvilEthSigner = new ethers.Wallet(anvilPrivateKey, anvilEthProvider);
const anvilEthRouter = "0x8186Eaa8CE62Bb3a1a72DA4B103D98AFff417B4A";
const whGuardiansAdapter = "0x8564C314028B778C968E11485E4bD6aC13CF0eeF";

describe("EVM Adapter Tests", () => {
  describe("get adapter", () => {
    test("getAdapters()", async () => {
      const adapters = await getAdapters(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      // Since no adapters have been added, should be empty
      expect(adapters.length).toBe(0);
    });
  });
  describe("add adapter", () => {
    test("addAdapter()", async () => {
      const index: bigint = await addAdapter(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        whGuardiansAdapter,
      );
      console.log("adapter index:", index);
      const adapters = await getAdapters(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      expect(adapters.length).toBe(1);
    });
  });
  describe("Various get functions", () => {
    const bigZero: bigint = BigInt(0);
    test("getAdapterByIndex()", async () => {
      const adapter = await getAdapterByIndex(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        bigZero,
      );
      console.log("adapter:", adapter);
      expect(adapter).toBe(whGuardiansAdapter);
    });
    test("getAdapterIndex()", async () => {
      const index: bigint = await getAdapterIndex(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        whGuardiansAdapter,
      );
      console.log("index: ", index);
      expect(index).toBe(bigZero);
    });
    test("getAdapters()", async () => {
      const adapters = await getAdapters(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
      );
      // Since no adapters have been added, should be empty
      expect(adapters.length).toBe(1);
      expect(adapters[0]).toBe(whGuardiansAdapter);
    });
  });
  describe("enable/disable adapter functions", () => {
    test("enable adapters", async () => {
      const sendChain = 42;
      let tx = await enableSendAdapter(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        sendChain,
        whGuardiansAdapter,
      );
      await tx.wait();
      const sendAdapters = await getSendAdaptersByChain(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        sendChain,
      );
      expect(sendAdapters.length).toBe(1);
      expect(sendAdapters[0]).toBe(whGuardiansAdapter);
      tx = await enableRecvAdapter(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        sendChain,
        whGuardiansAdapter,
      );
      await tx.wait();
      const recvAdapters = await getRecvAdaptersByChain(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        sendChain,
      );
      expect(recvAdapters.length).toBe(1);
      expect(recvAdapters[0]).toBe(whGuardiansAdapter);
    });
    test("disable adapters", async () => {
      const sendChain = 42;
      let tx = await disableSendAdapter(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        sendChain,
        whGuardiansAdapter,
      );
      await tx.wait();
      const sendAdapters = await getSendAdaptersByChain(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        sendChain,
      );
      expect(sendAdapters.length).toBe(0);
      tx = await disableRecvAdapter(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        sendChain,
        whGuardiansAdapter,
      );
      await tx.wait();
      const recvAdapters = await getRecvAdaptersByChain(
        anvilEthRouter,
        anvilEthSigner,
        anvilEthSigner.address,
        sendChain,
      );
      expect(recvAdapters.length).toBe(0);
    });
  });
});
