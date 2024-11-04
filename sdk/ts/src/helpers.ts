import { Network } from "@wormhole-foundation/sdk-base/constants";

export const assertEnvironmentVariable = (varName: string) => {
  if (varName in process.env) return process.env[varName]!;
  throw new Error(`Missing required environment variable: ${varName}`);
};

export function getNetwork(): Network {
  const network: string = assertEnvironmentVariable("NETWORK").toLowerCase();
  if (network === "mainnet") {
    return "Mainnet";
  }
  if (network === "testnet") {
    return "Testnet";
  }
  if (network === "devnet") {
    return "Devnet";
  }
  throw new Error(`Unknown network: ${network}`);
}
