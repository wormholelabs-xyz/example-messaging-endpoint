#!/bin/bash

#
# This script receives a message using the TestIntegrator contract.
# Usage: RPC_URL= MNEMONIC= INTEGRATOR= SRC_CHAIN= SRC_ADDR= SEQ= DST_CHAIN= DST_ADDR= PAYLOAD_HASH= ./sh/receiveTestMessage.sh
#

[[ -z $INTEGRATOR ]] && { echo "Missing INTEGRATOR"; exit 1; }
[[ -z $SRC_CHAIN ]] && { echo "Missing SRC_CHAIN"; exit 1; }
[[ -z $SRC_ADDR ]] && { echo "Missing SRC_ADDR"; exit 1; }
[[ -z $SEQ ]] && { echo "Missing SEQ"; exit 1; }
[[ -z $DST_CHAIN ]] && { echo "Missing DST_CHAIN"; exit 1; }
[[ -z $DST_ADDR ]] && { echo "Missing DST_ADDR"; exit 1; }
[[ -z $PAYLOAD_HASH ]] && { echo "Missing PAYLOAD_HASH"; exit 1; }

if [ "${RPC_URL}X" == "X" ]; then
  RPC_URL=http://localhost:8545
fi

if [ "${MNEMONIC}X" == "X" ]; then
  MNEMONIC=0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d
fi

forge script ./script/ReceiveTestMessage.s.sol:ReceiveTestMessage \
	--sig "run(address,uint16,bytes32,uint64,uint16,bytes32,bytes32)" $INTEGRATOR $SRC_CHAIN $SRC_ADDR $SEQ $DST_CHAIN $DST_ADDR $PAYLOAD_HASH \
	--rpc-url $RPC_URL \
	--private-key $MNEMONIC \
	--broadcast
