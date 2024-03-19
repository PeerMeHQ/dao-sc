NETWORK_NAME="devnet" # devnet, testnet, mainnet

ENTITY_ADDRESS=$(mxpy data load --partition $NETWORK_NAME --key=entity--address)
ENTITY_DEPLOY_TRANSACTION=$(mxpy data load --partition $NETWORK_NAME --key=entity--deploy-transaction)
MANAGER_ADDRESS=$(mxpy data load --partition $NETWORK_NAME --key=manager--address)
MANAGER_DEPLOY_TRANSACTION=$(mxpy data load --partition $NETWORK_NAME --key=manager--deploy-transaction)
PROXY=$(mxpy data load --partition $NETWORK_NAME --key=proxy)
CHAIN_ID=$(mxpy data load --partition $NETWORK_NAME --key=chain-id)
TRUSTED_HOST_ADDRESS=$(mxpy data load --partition $NETWORK_NAME --key=trusted-host-address)
NATIVE_TOKEN=$(mxpy data load --partition $NETWORK_NAME --key=native-token-id)
COST_ENTITY_CREATION_AMOUNT=$(mxpy data load --partition $NETWORK_NAME --key=cost-entity-creation-amount)
COST_DAILY_BASE_AMOUNT=$(mxpy data load --partition $NETWORK_NAME --key=cost-daily-base-amount)
DEX_WEGLD_TOKEN_ID=$(mxpy data load --partition $NETWORK_NAME --key=dex-wegld-token-id)
DEX_COST_TOKEN_WEGLD_SWAP_CONTRACT=$(mxpy data load --partition $NETWORK_NAME --key=dex-cost-token-wegld-swap-contract)
DEX_WRAP_EGLD_SWAP_CONTRACT=$(mxpy data load --partition $NETWORK_NAME --key=dex-wrap-egld-contract)
ORGANIZATION_CONTRACT=$(mxpy data load --partition $NETWORK_NAME --key=organization-contract)
CREDITS_REWARD_TOKEN_ID=$(mxpy data load --partition $NETWORK_NAME --key=credits-reward-token-id)
CREDITS_BONUS_FACTOR=$(mxpy data load --partition $NETWORK_NAME --key=credits-bonus-factor)

deploy() {
    echo "accidental deploy protection is activated."
    exit 1;

    mxpy contract build entity || return
    mxpy contract build manager || return

    cargo test || return

    mxpy contract deploy --project entity \
        --arguments $TRUSTED_HOST_ADDRESS \
        --recall-nonce --gas-limit=200000000 \
        --outfile="deploy-$NETWORK_NAME-entity.interaction.json" \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return

    ENTITY_ADDRESS=$(mxpy data parse --file="deploy-$NETWORK_NAME-entity.interaction.json" --expression="data['contractAddress']")
    ENTITY_TRANSACTION=$(mxpy data parse --file="deploy-$NETWORK_NAME-entity.interaction.json" --expression="data['emittedTransactionHash']")

    mxpy data store --partition $NETWORK_NAME --key=entity--address --value=$ENTITY_ADDRESS
    mxpy data store --partition $NETWORK_NAME --key=entity--deploy-transaction --value=$ENTITY_TRANSACTION

    sleep 6

    mxpy contract deploy --project manager \
        --arguments $ENTITY_ADDRESS $TRUSTED_HOST_ADDRESS "str:$NATIVE_TOKEN" $COST_ENTITY_CREATION_AMOUNT \
        --recall-nonce --gas-limit=80000000 \
        --outfile="deploy-$NETWORK_NAME-manager.interaction.json" \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --metadata-payable \
        --metadata-payable-by-sc \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return

    MANAGER_ADDRESS=$(mxpy data parse --file="deploy-$NETWORK_NAME-manager.interaction.json" --expression="data['contractAddress']")
    MANAGER_TRANSACTION=$(mxpy data parse --file="deploy-$NETWORK_NAME-manager.interaction.json" --expression="data['emittedTransactionHash']")

    mxpy data store --partition $NETWORK_NAME --key=manager--address --value=$MANAGER_ADDRESS
    mxpy data store --partition $NETWORK_NAME --key=manager--deploy-transaction --value=$MANAGER_TRANSACTION

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
    mxpy contract clean manager || return
    mxpy contract build manager || return

    cargo test || return

    mxpy contract upgrade $MANAGER_ADDRESS --project manager \
        --arguments $ENTITY_ADDRESS $TRUSTED_HOST_ADDRESS "str:$NATIVE_TOKEN" $COST_ENTITY_CREATION_AMOUNT \
        --metadata-payable \
        --metadata-payable-by-sc \
        --recall-nonce --gas-limit=100000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

upgradeEntityTemplate() {
    mxpy contract clean entity || return
    mxpy contract build entity || return

    cargo test || return

    mxpy contract upgrade $ENTITY_ADDRESS --project entity \
        --arguments $TRUSTED_HOST_ADDRESS \
        --recall-nonce --gas-limit=200000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# paras:
#   $1 = entity address
upgradeEntity() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="upgradeEntity" \
        --arguments $1 \
        --recall-nonce --gas-limit=100000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

initCreditsModule() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="initCreditsModule" \
        --arguments "str:$CREDITS_REWARD_TOKEN_ID" $CREDITS_BONUS_FACTOR \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

initDexModule() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="initDexModule" \
        --arguments "str:$DEX_WEGLD_TOKEN_ID" $DEX_COST_TOKEN_WEGLD_SWAP_CONTRACT $DEX_WRAP_EGLD_SWAP_CONTRACT \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

initOrgModule() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="initOrgModule" \
        --arguments $ORGANIZATION_CONTRACT \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

forwardCostTokensToOrg() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="forwardCostTokensToOrg" \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

setEntityCreationCost() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="setEntityCreationCost" \
        --arguments $COST_ENTITY_CREATION_AMOUNT \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

setDailyBaseCost() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="setDailyBaseCost" \
        --arguments $COST_DAILY_BASE_AMOUNT \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = feature name
#   $2 = amount
setDailyFeatureCost() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="setDailyFeatureCost" \
        --arguments "str:$1" $2 \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

createEntity() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$NATIVE_TOKEN" $COST_ENTITY_CREATION_AMOUNT "str:createEntity" \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = address
#   $2 = amount
boost() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$NATIVE_TOKEN" $2 "str:boost" $1 \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = token id
#   $2 = amount
#   $3 = entity address
#   $4 = dex swap contract address
boostWithSwap() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$1" $2 "str:boostWithSwap" $3 $4 \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = value
#   $2 = entity address
boostWithSwapEgld() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="boostWithSwap" \
        --arguments $2 \
        --value=$1 \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

getEntityTemplateAddress() {
    mxpy contract query $MANAGER_ADDRESS \
        --function="getEntityTemplateAddress" \
        --proxy=$PROXY || return
}

getCostTokenId() {
    mxpy contract query $MANAGER_ADDRESS \
        --function="getCostTokenId" \
        --proxy=$PROXY || return
}

getEntityCreationCost() {
    mxpy contract query $MANAGER_ADDRESS \
        --function="getEntityCreationCost" \
        --proxy=$PROXY || return
}

getBaseDailyCost() {
    mxpy contract query $MANAGER_ADDRESS \
        --function="getBaseDailyCost" \
        --proxy=$PROXY || return
}

# params:
#   $1 = entity address
getCredits() {
    mxpy contract query $MANAGER_ADDRESS \
        --function="getCredits" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = entity address
getFeatures() {
    mxpy contract query $ADDRESS \
        --function="getFeatures" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}

# params:
#   $1 = token
#   $2 = amount
#   $3 = address
forwardToken() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="forwardToken" \
        --arguments "str:$1" $2 $3 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = address
addAdmin() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="addAdmin" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = address
removeAdmin() {
    mxpy contract call $MANAGER_ADDRESS \
        --function="removeAdmin" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}
