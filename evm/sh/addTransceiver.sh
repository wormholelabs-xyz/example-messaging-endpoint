#!/bin/bash

#
# This script adds a transceiver for a given chain for an integrator to the Router contract.
# Usage: RPC_URL= MNEMONIC= ROUTER= INTEGRATOR= TRANSCEIVER= ./sh/addTransceiver.sh
#  tilt: ROUTER= INTEGRATOR= TRANSCEIVER= ./sh/addTransceiver.sh
#

[[ -z $ROUTER ]] && { echo "Missing ROUTER"; exit 1; }
[[ -z $INTEGRATOR ]] && { echo "Missing INTEGRATOR"; exit 1; }
[[ -z $TRANSCEIVER ]] && { echo "Missing TRANSCEIVER"; exit 1; }

if [ "${RPC_URL}X" == "X" ]; then
  RPC_URL=http://localhost:8545
fi

if [ "${MNEMONIC}X" == "X" ]; then
  MNEMONIC=0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d
fi

forge script ./script/AddTransceiver.s.sol:AddTransceiver \
	--sig "run(address,address,address)" $ROUTER $INTEGRATOR $TRANSCEIVER \
	--rpc-url $RPC_URL \
	--private-key $MNEMONIC \
	--broadcast
