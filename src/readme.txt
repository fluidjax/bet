# Install to Chain instructions

# Run the Chain
    ignite chain serve --reset-once


# Create an empty contract using template
    cargo generate --git https://github.com/CosmWasm/cw-template.git --name myproject

---------------

# Build the Contract
cargo wasm
docker run --rm -v "$(pwd)":/code --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/workspace-optimizer:0.16.0

---------------

#Set some variables
    export NODE=(--node http://127.0.0.1:26657)
    export TXFLAG=($NODE --chain-id zenrock --gas-prices 0.25urock --gas auto --gas-adjustment 1.3)
    export ALICE=zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty
    export CONTRACT=zen14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s38wvxu


---------------

#Upload the contract to the chain
    zenrockd tx wasm store artifacts/bet.wasm --from $ALICE $TXFLAG -y  -b sync

---------------

# Initislise the contract
    zenrockd tx wasm instantiate 1 '{"admin":"","rake_basis_points":150}'  --from $ALICE --label "random1"  --no-admin $TXFLAG -y

---------------
#Display the contract ID - it should be the same as in $CONTRACT env var (if not change the var)
    zenrockd query wasm list-contract-by-code 1 $NODE --output json


#watch the results
watch 'for i in {1..20}; do zenrockd query wasm contract-state smart zen14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s38wvxu "{\"bet_at\":{\"address\":\"zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty\",\"index\":$i}}" --output json; done'


#Fund the bank
zenrockd tx wasm execute zen14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s38wvxu '{"bet":{"guess":0,"to":"zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty","odds":1000000}}' --from alice --amount=1000000urock  $TXFLAG -y

#Execute a single 2:1 Bet
zenrockd tx wasm execute zen14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s38wvxu '{"bet":{"guess":1,"to":"zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty","odds":2}}' --from alice --amount=100urock  $TXFLAG -y



#View the result of that Bet increment index each time you make a bet
zenrockd query wasm contract-state smart zen14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s38wvxu '{"bet_at":{"address":"zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty","index":1}}' --output json


#Execute a Bet operation 10 times & veiw the results
clear;for i in {1..10}; do zenrockd tx wasm execute zen14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s38wvxu '{"bet":{"guess":1,"to":"zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty","odds":5}}' --from alice --amount=100urock $TXFLAG -y; sleep 1;done
clear;for i in {1..10}; do zenrockd query wasm contract-state smart zen14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s38wvxu "{\"bet_at\":{\"address\":\"zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty\",\"index\":$i}}" --output json; done



#Get Balance
zenrockd query bank balance $ALICE urock
zenrockd query bank balance zen14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s38wvxu urock


