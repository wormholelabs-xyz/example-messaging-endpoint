#!/bin/bash

#
# This script deploys the Endpoint contract.
# Usage: RPC_URL= MNEMONIC= OUR_CHAIN_ID= EVM_CHAIN_ID= ./sh/deployEndpoint.sh
#  tilt: ./sh/deployEndpoint.sh
#

if [ "${RPC_URL}X" == "X" ]; then
  RPC_URL=http://localhost:8545
fi

if [ "${MNEMONIC}X" == "X" ]; then
  MNEMONIC=0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d
fi

if [ "${OUR_CHAIN_ID}X" == "X" ]; then
  OUR_CHAIN_ID=2
fi

if [ "${EVM_CHAIN_ID}X" == "X" ]; then
  EVM_CHAIN_ID=1337
fi

forge script ./script/DeployEndpoint.s.sol:DeployEndpoint \
	--sig "run(uint16)" $OUR_CHAIN_ID \
	--rpc-url "$RPC_URL" \
	--private-key "$MNEMONIC" \
	--broadcast ${FORGE_ARGS}

returnInfo=$(cat ./broadcast/DeployEndpoint.s.sol/$EVM_CHAIN_ID/run-latest.json)

DEPLOYED_ADDRESS=$(jq -r '.returns.deployedAddress.value' <<< "$returnInfo")
echo "Deployed endpoint address: $DEPLOYED_ADDRESS"
