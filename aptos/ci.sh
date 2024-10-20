#!/bin/bash

for dir in */     # list directories in the form "/tmp/dirname/"
do
    dir=${dir%*/}      # remove the trailing "/"
    cd ${dir}
    aptos init --skip-faucet
    sh -c "aptos move test --move-2 --coverage --named-addresses ${dir}=default"
    sh -c "aptos move coverage summary --move-2 --named-addresses ${dir}=default | grep \"Move Coverage: 100.00\""
    cd ..
done
