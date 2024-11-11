#!/bin/bash

#
# This script adds an adapter for a given chain for an integrator to the Endpoint contract.
# Usage: RPC_URL= MNEMONIC= ENDPOINT= INTEGRATOR= ADAPTER= ./sh/addAdapter.sh
#  tilt: ENDPOINT= INTEGRATOR= ADAPTER= ./sh/addAdapter.sh
#

[[ -z $ENDPOINT ]] && { echo "Missing ENDPOINT"; exit 1; }
[[ -z $INTEGRATOR ]] && { echo "Missing INTEGRATOR"; exit 1; }
[[ -z $ADAPTER ]] && { echo "Missing ADAPTER"; exit 1; }

if [ "${RPC_URL}X" == "X" ]; then
  RPC_URL=http://localhost:8545
fi

if [ "${MNEMONIC}X" == "X" ]; then
  MNEMONIC=0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d
fi

forge script ./script/AddAdapter.s.sol:AddAdapter \
	--sig "run(address,address,address)" $ENDPOINT $INTEGRATOR $ADAPTER \
	--rpc-url $RPC_URL \
	--private-key $MNEMONIC \
	--broadcast
