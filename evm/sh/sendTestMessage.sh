#!/bin/bash

#
# This script sends a message using the TestIntegrator contract.
# Usage: RPC_URL= MNEMONIC= INTEGRATOR= DST_ADDR= ./sh/sendTestMessage.sh
#  tilt: INTEGRATOR=0x9e90054F4B6730cffAf1E6f6ea10e1bF9dD26dbb DST_ADDR=0x9e90054F4B6730cffAf1E6f6ea10e1bF9dD26dbb ./sh/sendTestMessage.sh
#

[[ -z $INTEGRATOR ]] && { echo "Missing INTEGRATOR"; exit 1; }
[[ -z $DST_ADDR ]] && { echo "Missing DST_ADDR"; exit 1; }

if [ "${RPC_URL}X" == "X" ]; then
  RPC_URL=http://localhost:8545
fi

if [ "${MNEMONIC}X" == "X" ]; then
  MNEMONIC=0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d
fi

forge script ./script/SendTestMessage.s.sol:SendTestMessage \
	--sig "run(address,address)" $INTEGRATOR $DST_ADDR \
	--rpc-url $RPC_URL \
	--private-key $MNEMONIC \
	--broadcast
