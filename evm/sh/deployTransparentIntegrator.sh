#!/bin/bash

#
# This script deploys the TransparentIntegrator contract.
# Usage: RPC_URL= MNEMONIC= ROUTER= EVM_CHAIN_ID= ./sh/deployTransparentIntegrator.sh
#

if [ "${RPC_URL}X" == "X" ]; then
  RPC_URL=http://localhost:8545
fi

if [ "${MNEMONIC}X" == "X" ]; then
  MNEMONIC=0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d
fi

if [ "${ROUTER}X" == "X" ]; then
  ROUTER=0x8186Eaa8CE62Bb3a1a72DA4B103D98AFff417B4A
fi

if [ "${EVM_CHAIN_ID}X" == "X" ]; then
  EVM_CHAIN_ID=31337
fi

[[ -z $INSTANCE ]] && { echo "Missing INSTANCE string"; exit 1; }

forge script ./script/DeployTransparentIntegrator.s.sol:DeployTransparentIntegrator \
	--sig "run(address,string)" $ROUTER "$INSTANCE" \
	--rpc-url "$RPC_URL" \
	--private-key "$MNEMONIC" \
	--broadcast ${FORGE_ARGS}

returnInfo=$(cat ./broadcast/DeployTransparentIntegrator.s.sol/$EVM_CHAIN_ID/run-latest.json)

DEPLOYED_ADDRESS=$(jq -r '.returns.deployedAddress.value' <<< "$returnInfo")
echo "Deployed TransparentIntegrator address: $DEPLOYED_ADDRESS"
