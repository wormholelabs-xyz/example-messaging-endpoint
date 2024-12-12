import { jest, expect, test } from "@jest/globals";
import { ethers } from "ethers";
import { sendMessage } from "../src/message";

jest.setTimeout(180000);

const anvilPrivateKey =
  "0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d";
const anvilEthProvider = new ethers.JsonRpcProvider("http://127.0.0.1:8545");
const anvilEthSigner = new ethers.Wallet(anvilPrivateKey, anvilEthProvider);
const anvilEthRouter = "0x8186Eaa8CE62Bb3a1a72DA4B103D98AFff417B4A";
const whGuardiansAdapter = "0x8564C314028B778C968E11485E4bD6aC13CF0eeF";

describe("Message tests", () => {
  describe("send a message", () => {
    test("sendMessage()", async () => {
      const destChain = 42;
      const destAddr = "";
      const payloadHash = "";
      const refundAddr = "";
      // Send a message.
      const sequence: bigint = await sendMessage(
        anvilEthRouter,
        anvilEthSigner,
        destChain,
        destAddr,
        payloadHash,
        refundAddr,
      );
      console.log("sequence: ", sequence);
    });
  });
});
