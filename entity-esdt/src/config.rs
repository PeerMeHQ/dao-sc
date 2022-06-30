use elrond_wasm::api::{ED25519_SIGNATURE_BYTE_LEN, KECCAK256_RESULT_LEN};

use crate::governance::proposal::Proposal;

elrond_wasm::imports!();

pub const VOTING_PERIOD_MINUTES_DEFAULT: usize = 4320; // 3 days

pub const SEALED_NOT_SET: u8 = 0;
pub const SEALED_ON: u8 = 1;

#[elrond_wasm::module]
pub trait ConfigModule {
    fn require_caller_self(&self) {
        let caller = self.blockchain().get_caller();
        let sc_address = self.blockchain().get_sc_address();
        require!(caller == sc_address, "action not allowed by user");
    }

    fn require_caller_self_or_unsealed(&self) {
        if self.is_sealed() {
            self.require_caller_self();
        }
    }

    fn require_not_sealed(&self) {
        require!(!self.is_sealed(), "entity is sealed");
    }

    fn require_sealed(&self) {
        require!(self.is_sealed(), "entity is not sealed yet");
    }

    fn is_sealed(&self) -> bool {
        self.sealed().get() == SEALED_ON
    }

    fn require_payment_token_governance_token(&self) {
        require!(self.call_value().single_esdt().token_identifier == self.governance_token_id().get(), "invalid token");
    }

    fn require_governance_tokens_available(&self, amount: &BigUint) {
        let gov_token_id = self.governance_token_id().get();
        let protected = self.protected_vote_tokens(&gov_token_id).get();
        let balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(gov_token_id), 0u64);
        let available = balance - protected;

        require!(amount <= &available, "not enough governance tokens available");
    }

    fn require_signed_by_trusted_host(&self, signable: &ManagedBuffer, signature: &ManagedByteArray<Self::Api, ED25519_SIGNATURE_BYTE_LEN>) {
        require!(!self.trusted_host_address().is_empty(), "trusted host address must be set");

        let trusted_host = self.trusted_host_address().get();
        let signable_hashed = self.crypto().keccak256(signable).as_managed_buffer().clone();
        let trusted = self.crypto().verify_ed25519_legacy_managed::<KECCAK256_RESULT_LEN>(trusted_host.as_managed_byte_array(), &signable_hashed, &signature);

        require!(trusted, "not a trusted host");
    }

    fn try_change_governance_token(&self, token_id: TokenIdentifier) {
        require!(token_id.is_valid_esdt_identifier(), "invalid governance token id");
        self.governance_token_id().set(token_id);
    }

    fn try_change_quorum(&self, quorum: BigUint) {
        require!(quorum != 0, "invalid quorum");
        self.quorum().set(&quorum);
    }

    fn try_change_min_proposal_vote_weight(&self, vote_weight: BigUint) {
        require!(vote_weight != 0, "min proposal vote weight can not be zero");
        self.min_proposal_vote_weight().set(&vote_weight);
    }

    fn try_change_voting_period_in_minutes(&self, voting_period: usize) {
        require!(voting_period != 0, "voting period (in minutes) can not be zero");
        self.voting_period_in_minutes().set(&voting_period);
    }

    #[storage_mapper("users")]
    fn users(&self) -> UserMapper;

    #[view(getTrustedHostAddress)]
    #[storage_mapper("trusted_host_addr")]
    fn trusted_host_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getSealed)]
    #[storage_mapper("sealed")]
    fn sealed(&self) -> SingleValueMapper<u8>;

    #[view(getTokenId)]
    #[storage_mapper("token")]
    fn token(&self) -> FungibleTokenMapper<Self::Api>;

    #[view(getGovernanceTokenId)]
    #[storage_mapper("governance_token_id")]
    fn governance_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getVoteNftTokenId)]
    #[storage_mapper("vote_nft_token")]
    fn vote_nft_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    #[view(getProtectedVoteTokens)]
    #[storage_mapper("protected_vote_tokens")]
    fn protected_vote_tokens(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("proposals")]
    fn proposals(&self, id: u64) -> SingleValueMapper<Proposal<Self::Api>>;

    #[view(getProposalIdCounter)]
    #[storage_mapper("proposals_id_counter")]
    fn next_proposal_id(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("proposal_signers")]
    fn proposal_signers(&self, proposal_id: u64, role_name: &ManagedBuffer) -> UnorderedSetMapper<usize>;

    #[storage_mapper("known_th_proposals_ids")]
    fn known_trusted_host_proposal_ids(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[view(getQuorum)]
    #[storage_mapper("quorum")]
    fn quorum(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinProposalVoteWeight)]
    #[storage_mapper("min_proposal_vote_weight")]
    fn min_proposal_vote_weight(&self) -> SingleValueMapper<BigUint>;

    #[view(getVotingPeriodMinutes)]
    #[storage_mapper("voting_period_minutes")]
    fn voting_period_in_minutes(&self) -> SingleValueMapper<usize>;
}
