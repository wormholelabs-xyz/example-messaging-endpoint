import {
  asChainId,
  ChainId,
  chainIdToChain,
  chainToPlatform,
  wormhole,
} from "@wormhole-foundation/sdk";
import evm from "@wormhole-foundation/sdk/evm";
import solana from "@wormhole-foundation/sdk/solana";
import {
  addAdapter as addAdapterEvm,
  disableRecvAdapter as disableRecvAdapterEvm,
  disableSendAdapter as disableSendAdapterEvm,
  enableRecvAdapter as enableRecvAdapterEvm,
  enableSendAdapter as enableSendAdapterEvm,
  getAdapterByIndex as getAdapterByIndexEvm,
  getAdapterIndex as getAdapterIndexEvm,
  getAdapters as getAdaptersEvm,
  getRecvAdaptersByChain as getRecvAdaptersByChainEvm,
  getSendAdaptersByChain as getSendAdaptersByChainEvm,
} from "./platforms/evm/src/adapter";
import {
  addAdapter as addAdapterSolana,
  disableRecvAdapter as disableRecvAdapterSolana,
  disableSendAdapter as disableSendAdapterSolana,
  enableRecvAdapter as enableRecvAdapterSolana,
  enableSendAdapter as enableSendAdapterSolana,
  getAdapterByIndex as getAdapterByIndexSolana,
  getAdapterIndex as getAdapterIndexSolana,
  getAdapters as getAdaptersSolana,
  getRecvAdaptersByChain as getRecvAdaptersByChainSolana,
  getSendAdaptersByChain as getSendAdaptersByChainSolana,
} from "./platforms/solana/adapter";
import { ethers } from "ethers";

export async function addAdapter(
  chain: number,
  endpointAddress: string,
  integrator: string,
  adapter: string,
  walletPrivateKey: string,
): Promise<bigint> {
  const chainId: ChainId = asChainId(chain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    return await addAdapterEvm(endpointAddress, wallet, integrator, adapter);
  } else if (platform === "Solana") {
    return await addAdapterSolana(integrator, adapter);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function disableRecvAdapter(
  contractChain: number,
  endpointAddress: string,
  integrator: string,
  adapter: string,
  disabledChain: number,
  walletPrivateKey: string,
): Promise<void> {
  const chainId: ChainId = asChainId(contractChain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    await disableRecvAdapterEvm(
      endpointAddress,
      wallet,
      integrator,
      disabledChain,
      adapter,
    );
  } else if (platform === "Solana") {
    await disableRecvAdapterSolana(integrator, adapter, disabledChain);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function disableSendAdapter(
  contractChain: number,
  endpointAddress: string,
  integrator: string,
  adapter: string,
  disabledChain: number,
  walletPrivateKey: string,
): Promise<void> {
  const chainId: ChainId = asChainId(contractChain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    await disableSendAdapterEvm(
      endpointAddress,
      wallet,
      integrator,
      disabledChain,
      adapter,
    );
  } else if (platform === "Solana") {
    await disableSendAdapterSolana(integrator, adapter, disabledChain);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function enableRecvAdapter(
  contractChain: number,
  endpointAddress: string,
  integrator: string,
  adapter: string,
  enabledChain: number,
  walletPrivateKey: string,
): Promise<void> {
  const chainId: ChainId = asChainId(contractChain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    await enableRecvAdapterEvm(
      endpointAddress,
      wallet,
      integrator,
      enabledChain,
      adapter,
    );
  } else if (platform === "Solana") {
    await enableRecvAdapterSolana(integrator, adapter, enabledChain);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function enableSendAdapter(
  contractChain: number,
  endpointAddress: string,
  integrator: string,
  adapter: string,
  enabledChain: number,
  walletPrivateKey: string,
): Promise<void> {
  const chainId: ChainId = asChainId(contractChain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    await enableSendAdapterEvm(
      endpointAddress,
      wallet,
      integrator,
      enabledChain,
      adapter,
    );
  } else if (platform === "Solana") {
    await enableSendAdapterSolana(integrator, adapter, enabledChain);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function getAdapterByIndex(
  contractChain: number,
  endpointAddress: string,
  integrator: string,
  walletPrivateKey: string,
  index: bigint,
): Promise<string> {
  const chainId: ChainId = asChainId(contractChain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    return await getAdapterByIndexEvm(
      endpointAddress,
      wallet,
      integrator,
      index,
    );
  } else if (platform === "Solana") {
    return await getAdapterByIndexSolana(integrator, index);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function getAdapterIndex(
  contractChain: number,
  endpointAddress: string,
  integrator: string,
  adapter: string,
  walletPrivateKey: string,
): Promise<bigint> {
  const chainId: ChainId = asChainId(contractChain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    return await getAdapterIndexEvm(
      endpointAddress,
      wallet,
      integrator,
      adapter,
    );
  } else if (platform === "Solana") {
    return await getAdapterIndexSolana(integrator, adapter);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function getAdapters(
  contractChain: number,
  endpointAddress: string,
  integrator: string,
  walletPrivateKey: string,
): Promise<string[]> {
  const chainId: ChainId = asChainId(contractChain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    return await getAdaptersEvm(endpointAddress, wallet, integrator);
  } else if (platform === "Solana") {
    return await getAdaptersSolana(integrator);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function getRecvAdaptersByChain(
  contractChain: number,
  endpointAddress: string,
  integrator: string,
  walletPrivateKey: string,
  chain: number,
): Promise<string[]> {
  const chainId: ChainId = asChainId(contractChain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    return await getRecvAdaptersByChainEvm(
      endpointAddress,
      wallet,
      integrator,
      chain,
    );
  } else if (platform === "Solana") {
    return await getRecvAdaptersByChainSolana(integrator, chain);
  } else {
    throw new Error("Unsupported platform");
  }
}

export async function getSendAdaptersByChain(
  contractChain: number,
  endpointAddress: string,
  integrator: string,
  walletPrivateKey: string,
  chain: number,
): Promise<string[]> {
  const chainId: ChainId = asChainId(contractChain);
  const wh = await wormhole("Mainnet", [evm, solana]);
  const chainContext = wh.getChain(chainIdToChain(chainId));
  const platform = chainToPlatform(chainIdToChain(chainId));
  if (platform === "Evm") {
    const wallet = new ethers.Wallet(walletPrivateKey).connect(
      await chainContext.getRpc(),
    );
    return await getSendAdaptersByChainEvm(
      endpointAddress,
      wallet,
      integrator,
      chain,
    );
  } else if (platform === "Solana") {
    return await getSendAdaptersByChainSolana(integrator, chain);
  } else {
    throw new Error("Unsupported platform");
  }
}
