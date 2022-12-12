NETWORK_NAME="devnet" # devnet, testnet, mainnet
ADDRESS=""
GOV_TOKEN_ID=""

PROXY=$(erdpy data load --partition $NETWORK_NAME --key=proxy)
CHAIN_ID=$(erdpy data load --partition $NETWORK_NAME --key=chain-id)
COST_TOKEN_ID=$(erdpy data load --partition $NETWORK_NAME --key=cost-token-id)

# params:
#   $1 = content hash
#   $2 = content signature
#   $3 = token amount
propose() {
    erdpy contract call $ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$GOV_TOKEN_ID" $3 "str:propose" "str:$1" "str:$2" \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = proposal id
sign() {
    erdpy contract call $ADDRESS \
        --function="sign" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = proposal id
execute() {
    erdpy contract call $ADDRESS \
        --function="execute" \
        --arguments $1 \
        --recall-nonce --gas-limit=600000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = token id
changeGovToken() {
    erdpy contract call $ADDRESS \
        --function="changeGovToken" \
        --arguments "str:$1" \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = amount
changeQuorum() {
    erdpy contract call $ADDRESS \
        --function="changeQuorum" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = amount
changeMinProposalVoteWeight() {
    erdpy contract call $ADDRESS \
        --function="changeMinProposalVoteWeight" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = minutes
changeVotingPeriodMinutes() {
    erdpy contract call $ADDRESS \
        --function="changeVotingPeriodMinutes" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

getTrustedHostAddress() {
    erdpy contract query $ADDRESS \
        --function="getTrustedHostAddress" \
        --proxy=$PROXY || return
}

getVersion() {
    erdpy contract query $ADDRESS \
        --function="getVersion" \
        --proxy=$PROXY || return
}

getTokenId() {
    erdpy contract query $ADDRESS \
        --function="getTokenId" \
        --proxy=$PROXY || return
}

# params:
#   $1 = role name
createRole() {
    erdpy contract call $ADDRESS \
        --function="createRole" \
        --arguments "str:$1" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

getRoles() {
    erdpy contract query $ADDRESS \
        --function="getRoles" \
        --proxy=$PROXY || return
}

# params:
#   $1 = role name
getRoleMemberAmount() {
    erdpy contract query $ADDRESS \
        --function="getRoleMemberAmount" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}

# params:
#   $1 = permission name
#   $2 = destination address
#   $3 = sc endpoint
createPermission() {
    erdpy contract call $ADDRESS \
        --function="createPermission" \
        --arguments "str:$1" $2 "str:$3" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

getPermissions() {
    erdpy contract query $ADDRESS \
        --function="getPermissions" \
        --proxy=$PROXY || return
}

# params:
#   $1 = role name
getPolicies() {
    erdpy contract query $ADDRESS \
        --function="getPolicies" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}

# params:
#   $1 = user address
#   $2 = role name
assignRole() {
    erdpy contract call $ADDRESS \
        --function="assignRole" \
        --arguments $1 "str:$2" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = role name
#   $2 = permission name
#   $3 = quorum
#   $4 = voting period minutes
createPolicyWeighted() {
    erdpy contract call $ADDRESS \
        --function="createPolicyWeighted" \
        --arguments "str:$1" "str:$2" $3 $4 \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = role name
#   $2 = permission name
createPolicyForOne() {
    erdpy contract call $ADDRESS \
        --function="createPolicyForOne" \
        --arguments "str:$1" "str:$2" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = role name
#   $2 = permission name
createPolicyForAll() {
    erdpy contract call $ADDRESS \
        --function="createPolicyForAll" \
        --arguments "str:$1" "str:$2" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = role name
#   $2 = permission name
#   $3 = quorum
createPolicyQuorum() {
    erdpy contract call $ADDRESS \
        --function="createPolicyQuorum" \
        --arguments "str:$1" "str:$2" $3 \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = address
getUserRoles() {
    erdpy contract query $ADDRESS \
        --function="getUserRoles" \
        --arguments $1 \
        --proxy=$PROXY || return
}

getGovTokenId() {
    erdpy contract query $ADDRESS \
        --function="getGovTokenId" \
        --proxy=$PROXY || return
}

# backwards compatibility
getProtectedVoteTokens() {
    erdpy contract query $ADDRESS \
        --function="getProtectedVoteTokens" \
        --arguments "str:$GOV_TOKEN_ID" \
        --proxy=$PROXY || return
}

# params:
#   $1 = token id
#   $2 = nonce
getGuardedVoteTokens() {
    erdpy contract query $ADDRESS \
        --function="getGuardedVoteTokens" \
        --arguments "str:$1" $2 \
        --proxy=$PROXY || return
}

# params:
#   $1 = token id
isLockingVoteTokens() {
    erdpy contract query $ADDRESS \
        --function="isLockingVoteTokens" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}

getQuorum() {
    erdpy contract query $ADDRESS \
        --function="getQuorum" \
        --proxy=$PROXY || return
}

getMinProposeWeight() {
    erdpy contract query $ADDRESS \
        --function="getMinProposeWeight" \
        --proxy=$PROXY || return
}

getVotingPeriodMinutes() {
    erdpy contract query $ADDRESS \
        --function="getVotingPeriodMinutes" \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposal() {
    erdpy contract query $ADDRESS \
        --function="getProposal" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalStatus() {
    erdpy contract query $ADDRESS \
        --function="getProposalStatus" \
        --arguments $1 \
        --proxy=$PROXY || return
}

getProposalIdCounter() {
    erdpy contract query $ADDRESS \
        --function="getProposalIdCounter" \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalVotes() {
    erdpy contract query $ADDRESS \
        --function="getProposalVotes" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalSigners() {
    erdpy contract query $ADDRESS \
        --function="getProposalSigners" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalSignatureRoleCounts() {
    erdpy contract query $ADDRESS \
        --function="getProposalSignatureRoleCounts" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalPollResults() {
    erdpy contract query $ADDRESS \
        --function="getProposalPollResults" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = address
getWithdrawableProposalIds() {
    erdpy contract query $ADDRESS \
        --function="getWithdrawableProposalIds" \
        --arguments $1 \
        --proxy=$PROXY || return
}
