import {
  asChainId,
  ChainId,
  chainIdToChain,
  chainToPlatform,
  wormhole,
} from "@wormhole-foundation/sdk";
import evm from "@wormhole-foundation/sdk/evm";
import solana from "@wormhole-foundation/sdk/solana";
import { ethers } from "ethers";
import {
  claimAdmin as claimAdminEvm,
  discardAdmin as discardAdminEvm,
  getAdmin as getAdminEvm,
  getPendingAdmin as getPendingAdminEvm,
  transferAdmin as transferAdminEvm,
  updateAdmin as updateAdminEvm,
} from "./platforms/evm/src/admin";
import {
  claimAdmin as claimAdminSolana,
  discardAdmin as discardAdminSolana,
  getAdmin as getAdminSolana,
  getPendingAdmin as getPendingAdminSolana,
  transferAdmin as transferAdminSolana,
  updateAdmin as updateAdminSolana,
} from "./platforms/solana/admin";

export async function claimAdmin(
  chain: number,
  endpointAddress: string,
  integrator: string,
  walletPrivateKey: string,
): Promise<void> {
  const chainId: ChainId = asChainId(chain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    await claimAdminEvm(endpointAddress, wallet, integrator);
  } else if (platform === "Solana") {
    // In this case the walletPrivateKey is the solana wallet private key as a string.
    await claimAdminSolana(integrator, walletPrivateKey);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function discardAdmin(
  chain: number,
  endpointAddress: string,
  integrator: string,
  walletPrivateKey: string,
): Promise<void> {
  const chainId: ChainId = asChainId(chain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    await discardAdminEvm(endpointAddress, wallet, integrator);
  } else if (platform === "Solana") {
    await discardAdminSolana(integrator);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function getAdmin(
  chain: number,
  endpointAddress: string,
  integrator: string,
  walletPrivateKey: string,
): Promise<string> {
  const chainId: ChainId = asChainId(chain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    return await getAdminEvm(endpointAddress, wallet, integrator);
  } else if (platform === "Solana") {
    return await getAdminSolana(integrator);
  } else {
    throw new Error("Unsupported platform");
  }
}
export async function getPendingAdmin(
  chain: number,
  endpointAddress: string,
  integrator: string,
  walletPrivateKey: string,
): Promise<string> {
  const chainId: ChainId = asChainId(chain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    return await getPendingAdminEvm(endpointAddress, wallet, integrator);
  } else if (platform === "Solana") {
    return await getPendingAdminSolana(integrator);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function transferAdmin(
  chain: number,
  endpointAddress: string,
  integrator: string,
  admin: string,
  walletPrivateKey: string,
): Promise<void> {
  const chainId: ChainId = asChainId(chain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    await transferAdminEvm(endpointAddress, wallet, integrator, admin);
  } else if (platform === "Solana") {
    await transferAdminSolana(integrator, admin);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function updateAdmin(
  chain: number,
  endpointAddress: string,
  integrator: string,
  admin: string,
  walletPrivateKey: string,
): Promise<void> {
  const chainId: ChainId = asChainId(chain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    await updateAdminEvm(endpointAddress, wallet, integrator, admin);
  } else if (platform === "Solana") {
    await updateAdminSolana(integrator, admin);
  } else {
    throw new Error("Unsupported platform");
  }
}
