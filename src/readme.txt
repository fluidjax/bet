# Install to Chain instructions

 export NODE=(--node http://127.0.0.1:26657)
 export TXFLAG=($NODE --chain-id zenrock --gas-prices 0.25urock --gas auto --gas-adjustment 1.3)
 export INIT='{"count":0}'

cargo wasm

docker run --rm -v "$(pwd)":/code \
--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
cosmwasm/workspace-optimizer:0.16.0


zenrockd  tx wasm store artifacts/random1.wasm --from alice $TXFLAG -y  -b sync

zenrockd tx wasm instantiate 1 $INIT --from alice --label "random1"  --no-admin $TXFLAG -y

zenrockd query wasm list-contract-by-code 1 $NODE --output json


CONTRACT=zen14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s38wvxu


zenrockd query wasm contract-state smart $CONTRACT '{"get_random":{}}' --output json



zenrockd  tx wasm store artifacts/random1.wasm --from alice $TXFLAG -y  -b sync
zenrockd tx wasm instantiate 1 $INIT --from alice --label "random1"  --no-admin $TXFLAG -y
zenrockd query wasm contract-state smart $CONTRACT '{"get_random":{}}' --output json

 zenrockd tx wasm execute 1 '{"bet":{"guess":1,"to":"zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty"}}' --from alice $TXFLAG -y