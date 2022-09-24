NETWORK_NAME="devnet" # devnet, testnet, mainnet

ENTITY_ADDRESS=$(erdpy data load --partition $NETWORK_NAME --key=entity--address)
ENTITY_DEPLOY_TRANSACTION=$(erdpy data load --partition $NETWORK_NAME --key=entity--deploy-transaction)
MANAGER_ADDRESS=$(erdpy data load --partition $NETWORK_NAME --key=manager--address)
MANAGER_DEPLOY_TRANSACTION=$(erdpy data load --partition $NETWORK_NAME --key=manager--deploy-transaction)
PROXY=$(erdpy data load --partition $NETWORK_NAME --key=proxy)
CHAIN_ID=$(erdpy data load --partition $NETWORK_NAME --key=chain-id)
TRUSTED_HOST_ADDRESS=$(erdpy data load --partition $NETWORK_NAME --key=trusted-host-address)
COST_TOKEN_ID=$(erdpy data load --partition $NETWORK_NAME --key=cost-token-id)
COST_ENTITY_CREATION_AMOUNT=$(erdpy data load --partition $NETWORK_NAME --key=cost-entity-creation-amount)
COST_DAILY_BASE_AMOUNT=$(erdpy data load --partition $NETWORK_NAME --key=cost-daily-base-amount)
DEX_WEGLD_TOKEN_ID=$(erdpy data load --partition $NETWORK_NAME --key=dex-wegld-token-id)
DEX_COST_TOKEN_WEGLD_SWAP_CONTRACT=$(erdpy data load --partition $NETWORK_NAME --key=dex-cost-token-wegld-swap-contract)
DEX_WRAP_EGLD_SWAP_CONTRACT=$(erdpy data load --partition $NETWORK_NAME --key=dex-wrap-egld-contract)
ORGANIZATION_CONTRACT=$(erdpy data load --partition $NETWORK_NAME --key=organization-contract)
BOOST_REWARD_TOKEN_ID=$(erdpy data load --partition $NETWORK_NAME --key=boost-reward-token-id)

deploy() {
    echo "accidental deploy protection is activated."
    exit 1;

    erdpy --verbose contract build entity || return
    erdpy --verbose contract build manager || return

    cargo test || return

    erdpy --verbose contract deploy --project entity \
        --arguments $TRUSTED_HOST_ADDRESS \
        --recall-nonce --gas-limit=200000000 \
        --outfile="deploy-$NETWORK_NAME-entity.interaction.json" \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return

    ENTITY_ADDRESS=$(erdpy data parse --file="deploy-$NETWORK_NAME-entity.interaction.json" --expression="data['contractAddress']")
    ENTITY_TRANSACTION=$(erdpy data parse --file="deploy-$NETWORK_NAME-entity.interaction.json" --expression="data['emittedTransactionHash']")

    erdpy data store --partition $NETWORK_NAME --key=entity--address --value=$ENTITY_ADDRESS
    erdpy data store --partition $NETWORK_NAME --key=entity--deploy-transaction --value=$ENTITY_TRANSACTION

    sleep 6

    erdpy --verbose contract deploy --project manager \
        --arguments $ENTITY_ADDRESS $TRUSTED_HOST_ADDRESS "str:$COST_TOKEN_ID" $COST_ENTITY_CREATION_AMOUNT \
        --recall-nonce --gas-limit=80000000 \
        --outfile="deploy-$NETWORK_NAME-manager.interaction.json" \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --metadata-payable \
        --metadata-payable-by-sc \
        --ledger \
        --send || return

    MANAGER_ADDRESS=$(erdpy data parse --file="deploy-$NETWORK_NAME-manager.interaction.json" --expression="data['contractAddress']")
    MANAGER_TRANSACTION=$(erdpy data parse --file="deploy-$NETWORK_NAME-manager.interaction.json" --expression="data['emittedTransactionHash']")

    erdpy data store --partition $NETWORK_NAME --key=manager--address --value=$MANAGER_ADDRESS
    erdpy data store --partition $NETWORK_NAME --key=manager--deploy-transaction --value=$MANAGER_TRANSACTION

    sleep 6
    setDailyBaseCost

    sleep 6
    initDexModule

    sleep 6
    initOrgModule

    sleep 6
    initCreditsModule

    echo ""
    echo "deployed ENTITY TEMPLATE: $ENTITY_ADDRESS"
    echo "deployed MANAGER: $MANAGER_ADDRESS"
}

upgrade() {
    erdpy --verbose contract clean manager || return
    erdpy --verbose contract build manager || return

    cargo test || return

    erdpy --verbose contract upgrade $MANAGER_ADDRESS --project manager \
        --arguments $ENTITY_ADDRESS $TRUSTED_HOST_ADDRESS "str:$COST_TOKEN_ID" $COST_ENTITY_CREATION_AMOUNT \
        --metadata-payable \
        --metadata-payable-by-sc \
        --recall-nonce --gas-limit=100000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

upgradeEntityTemplate() {
    erdpy --verbose contract clean entity || return
    erdpy --verbose contract build entity || return

    cargo test || return

    erdpy --verbose contract upgrade $ENTITY_ADDRESS --project entity \
        --arguments $TRUSTED_HOST_ADDRESS \
        --recall-nonce --gas-limit=200000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# paras:
#   $1 = entity address
upgradeEntity() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="upgradeEntity" \
        --arguments $1 \
        --recall-nonce --gas-limit=100000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

initCreditsModule() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="initCreditsModule" \
        --arguments "str:$BOOST_REWARD_TOKEN_ID" \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

initDexModule() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="initDexModule" \
        --arguments "str:$DEX_WEGLD_TOKEN_ID" $DEX_COST_TOKEN_WEGLD_SWAP_CONTRACT $DEX_WRAP_EGLD_SWAP_CONTRACT \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

initOrgModule() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="initOrgModule" \
        --arguments $ORGANIZATION_CONTRACT \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

setEntityCreationCost() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="setEntityCreationCost" \
        --arguments $COST_ENTITY_CREATION_AMOUNT \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

setDailyBaseCost() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="setDailyBaseCost" \
        --arguments $COST_DAILY_BASE_AMOUNT \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = feature name
#   $2 = amount
setDailyFeatureCost() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="setDailyFeatureCost" \
        --arguments "str:$1" $2 \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

createEntity() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$COST_TOKEN_ID" $COST_ENTITY_CREATION_AMOUNT "str:createEntity" \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = address
#   $2 = amount
boost() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$COST_TOKEN_ID" $2 "str:boost" $1 \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = token id
#   $2 = amount
#   $3 = entity address
#   $4 = dex swap contract address
boostWithSwap() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$1" $2 "str:boostWithSwap" $3 $4 \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = value
#   $2 = entity address
boostWithSwapEgld() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="boostWithSwap" \
        --arguments $2 \
        --value=$1 \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

getEntityTemplateAddress() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getEntityTemplateAddress" \
        --proxy=$PROXY || return
}

getCostTokenId() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getCostTokenId" \
        --proxy=$PROXY || return
}

getEntityCreationCost() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getEntityCreationCost" \
        --proxy=$PROXY || return
}

getBaseDailyCost() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getBaseDailyCost" \
        --proxy=$PROXY || return
}

# params:
#   $1 = entity address
getCredits() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getCredits" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = entity address
getFeatures() {
    erdpy contract query $ADDRESS \
        --function="getFeatures" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}
