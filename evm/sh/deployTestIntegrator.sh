#!/bin/bash

#
# This script deploys the test integrator contract to test the Router contract.
# Usage: RPC_URL= MNEMONIC= EVM_CHAIN_ID= ROUTER= PEER_CHAIN_ID= TRANSCEIVER= ./sh/deployTestIntegrator.sh
#  tilt: ROUTER=0x1aBE68277AE236083947f2551FEe8b885efCA8f5 PEER_CHAIN_ID=4 TRANSCEIVER=0xdFccc9C59c7361307d47c558ffA75840B32DbA29 ./sh/deployTestIntegrator.sh
#

if [ "${RPC_URL}X" == "X" ]; then
  RPC_URL=http://localhost:8545
fi

if [ "${MNEMONIC}X" == "X" ]; then
  MNEMONIC=0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d
fi

if [ "${EVM_CHAIN_ID}X" == "X" ]; then
  EVM_CHAIN_ID=1337
fi

if [ "${PEER_CHAIN_ID}X" == "X" ]; then
  PEER_CHAIN_ID=4
fi

[[ -z $ROUTER ]] && { echo "Missing ROUTER"; exit 1; }
[[ -z $TRANSCEIVER ]] && { echo "Missing TRANSCEIVER"; exit 1; }

forge create ./script/TestIntegrator.s.sol:TestIntegrator \
	--rpc-url $RPC_URL \
	--private-key $MNEMONIC \
	--verify --chain $EVM_CHAIN_ID \
  --constructor-args $ROUTER $PEER_CHAIN_ID $TRANSCEIVER
