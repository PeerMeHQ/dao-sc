NETWORK_NAME="devnet" # devnet, testnet, mainnet
ADDRESS=""
GOV_TOKEN_ID=""

PROXY=$(mxpy data load --partition $NETWORK_NAME --key=proxy)
CHAIN_ID=$(mxpy data load --partition $NETWORK_NAME --key=chain-id)
NATIVE_TOKEN=$(mxpy data load --partition $NETWORK_NAME --key=native-token-id)

# params:
#   $1 = content hash
#   $2 = content signature
#   $3 = token amount
propose() {
    mxpy contract call $ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$GOV_TOKEN_ID" $3 "str:propose" "str:$1" "str:$2" \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = proposal id
sign() {
    mxpy contract call $ADDRESS \
        --function="sign" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = proposal id
execute() {
    mxpy contract call $ADDRESS \
        --function="execute" \
        --arguments $1 \
        --recall-nonce --gas-limit=600000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = token id
changeGovToken() {
    mxpy contract call $ADDRESS \
        --function="changeGovToken" \
        --arguments "str:$1" \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = amount
changeQuorum() {
    mxpy contract call $ADDRESS \
        --function="changeQuorum" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = amount
changeMinProposalVoteWeight() {
    mxpy contract call $ADDRESS \
        --function="changeMinProposalVoteWeight" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = minutes
changeVotingPeriodMinutes() {
    mxpy contract call $ADDRESS \
        --function="changeVotingPeriodMinutes" \
        --arguments $1 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

getTrustedHostAddress() {
    mxpy contract query $ADDRESS \
        --function="getTrustedHostAddress" \
        --proxy=$PROXY || return
}

getVersion() {
    mxpy contract query $ADDRESS \
        --function="getVersion" \
        --proxy=$PROXY || return
}

# params:
#   $1 = role name
createRole() {
    mxpy contract call $ADDRESS \
        --function="createRole" \
        --arguments "str:$1" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

getRoles() {
    mxpy contract query $ADDRESS \
        --function="getRoles" \
        --proxy=$PROXY || return
}

# params:
#   $1 = role name
getRoleMemberAmount() {
    mxpy contract query $ADDRESS \
        --function="getRoleMemberAmount" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}

# params:
#   $1 = permission name
#   $2 = destination address
#   $3 = sc endpoint
createPermission() {
    mxpy contract call $ADDRESS \
        --function="createPermission" \
        --arguments "str:$1" $2 "str:$3" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

getPermissions() {
    mxpy contract query $ADDRESS \
        --function="getPermissions" \
        --proxy=$PROXY || return
}

# params:
#   $1 = role name
getPolicies() {
    mxpy contract query $ADDRESS \
        --function="getPolicies" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}

# params:
#   $1 = user address
#   $2 = role name
assignRole() {
    mxpy contract call $ADDRESS \
        --function="assignRole" \
        --arguments $1 "str:$2" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = role name
#   $2 = permission name
#   $3 = quorum
#   $4 = voting period minutes
createPolicyWeighted() {
    mxpy contract call $ADDRESS \
        --function="createPolicyWeighted" \
        --arguments "str:$1" "str:$2" $3 $4 \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = role name
#   $2 = permission name
createPolicyForOne() {
    mxpy contract call $ADDRESS \
        --function="createPolicyForOne" \
        --arguments "str:$1" "str:$2" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = role name
#   $2 = permission name
createPolicyForAll() {
    mxpy contract call $ADDRESS \
        --function="createPolicyForAll" \
        --arguments "str:$1" "str:$2" \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = role name
#   $2 = permission name
#   $3 = quorum
createPolicyQuorum() {
    mxpy contract call $ADDRESS \
        --function="createPolicyQuorum" \
        --arguments "str:$1" "str:$2" $3 \
        --recall-nonce --gas-limit=20000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        "${SNIPPETS_SECURE_SIGN_METHOD[@]}" \
        --send || return
}

# params:
#   $1 = address
getUserRoles() {
    mxpy contract query $ADDRESS \
        --function="getUserRoles" \
        --arguments $1 \
        --proxy=$PROXY || return
}

getGovTokenId() {
    mxpy contract query $ADDRESS \
        --function="getGovTokenId" \
        --proxy=$PROXY || return
}

# params:
#   $1 = token id
#   $2 = nonce
getGuardedVoteTokens() {
    mxpy contract query $ADDRESS \
        --function="getGuardedVoteTokens" \
        --arguments "str:$1" $2 \
        --proxy=$PROXY || return
}

# params:
#   $1 = token id
isLockingVoteTokens() {
    mxpy contract query $ADDRESS \
        --function="isLockingVoteTokens" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}

getQuorum() {
    mxpy contract query $ADDRESS \
        --function="getQuorum" \
        --proxy=$PROXY || return
}

getMinProposeWeight() {
    mxpy contract query $ADDRESS \
        --function="getMinProposeWeight" \
        --proxy=$PROXY || return
}

getVotingPeriodMinutes() {
    mxpy contract query $ADDRESS \
        --function="getVotingPeriodMinutes" \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposal() {
    mxpy contract query $ADDRESS \
        --function="getProposal" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalStatus() {
    mxpy contract query $ADDRESS \
        --function="getProposalStatus" \
        --arguments $1 \
        --proxy=$PROXY || return
}

getProposalIdCounter() {
    mxpy contract query $ADDRESS \
        --function="getProposalIdCounter" \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalVotes() {
    mxpy contract query $ADDRESS \
        --function="getProposalVotes" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalSigners() {
    mxpy contract query $ADDRESS \
        --function="getProposalSigners" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalSignatureRoleCounts() {
    mxpy contract query $ADDRESS \
        --function="getProposalSignatureRoleCounts" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalPollResults() {
    mxpy contract query $ADDRESS \
        --function="getProposalPollResults" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = address
getWithdrawableProposalIds() {
    mxpy contract query $ADDRESS \
        --function="getWithdrawableProposalIds" \
        --arguments $1 \
        --proxy=$PROXY || return
}
