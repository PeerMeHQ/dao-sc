elrond_wasm::imports!();

use self::{vote::VoteType};
use crate::config::{self, VOTING_PERIOD_MINUTES_DEFAULT};
use crate::permission;
use proposal::{Action, ProposalStatus};

pub mod events;
pub mod proposal;
pub mod vote;

#[elrond_wasm::module]
pub trait GovernanceModule: config::ConfigModule + permission::PermissionModule + events::GovEventsModule + proposal::ProposalModule + vote::VoteModule {
    fn init_governance_module(&self, gov_token_id: &TokenIdentifier, initial_tokens: &BigUint) {
        let initial_quorum = initial_tokens / &BigUint::from(20u64); // 5% of initial tokens
        let initial_min_tokens_for_proposing = initial_tokens / &BigUint::from(1000u64); // 0.1% of initial tokens

        self.next_proposal_id().set_if_empty(1);

        self.try_change_governance_token(gov_token_id.clone());
        self.try_change_quorum(BigUint::from(initial_quorum));
        self.try_change_min_proposal_vote_weight(BigUint::from(initial_min_tokens_for_proposing));
        self.try_change_voting_period_in_minutes(VOTING_PERIOD_MINUTES_DEFAULT);
    }

    #[endpoint(changeGovernanceToken)]
    fn change_gov_token_endpoint(&self, token_id: TokenIdentifier) {
        self.require_not_sealed();
        self.try_change_governance_token(token_id);
    }

    #[endpoint(changeQuorum)]
    fn change_quorum_endpoint(&self, value: BigUint) {
        self.require_caller_self_or_unsealed();
        self.try_change_quorum(value);
    }

    #[endpoint(changeMinProposalVoteWeight)]
    fn change_min_proposal_vote_weight_endpoint(&self, value: BigUint) {
        self.require_caller_self_or_unsealed();
        self.try_change_min_proposal_vote_weight(value);
    }

    #[endpoint(changeVotingPeriodMinutes)]
    fn change_voting_period_in_minutes_endpoint(&self, value: usize) {
        self.require_caller_self_or_unsealed();
        self.try_change_voting_period_in_minutes(value);
    }

    #[payable("*")]
    #[endpoint(propose)]
    fn propose_endpoint(
        &self,
        trusted_host_id: ManagedBuffer,
        content_hash: ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: ManagedBuffer,
        permissions: MultiValueManagedVec<ManagedBuffer>,
    ) -> u64 {
        let payment = self.call_value().single_esdt();
        let proposer = self.blockchain().get_caller();
        let permissions = permissions.into_vec();

        // self.require_proposed_via_trusted_host(&trusted_host_id, &content_hash, content_sig, &actions_hash, &permissions);
        self.require_payment_token_governance_token();
        self.require_sealed();

        require!(payment.amount > 0, "token ownership proof required");
        require!(!self.known_trusted_host_proposal_ids().contains(&trusted_host_id), "proposal already registered");

        let (policies, allowed) = self.can_propose(&proposer, &actions_hash, &permissions);
        let vote_weight = payment.amount.clone();
        require!(allowed, "action not allowed for user");

        if policies.is_empty() || self.has_token_weighted_policy(&policies) {
            require!(vote_weight >= self.min_proposal_vote_weight().get(), "insufficient vote weight");
        }

        let proposal = self.create_proposal(content_hash, actions_hash, vote_weight.clone(), permissions, &policies);
        let proposal_id = proposal.id;

        if !policies.is_empty() {
            self.sign_for_all_roles(&proposer, &proposal);
        }

        self.protected_vote_tokens(&payment.token_identifier).update(|current| *current += &payment.amount);
        self.known_trusted_host_proposal_ids().insert(trusted_host_id);
        self.create_vote_nft_and_send(&proposer, proposal.id, VoteType::For, vote_weight.clone(), payment.clone());
        self.emit_propose_event(proposal, payment, vote_weight);

        proposal_id
    }

    #[payable("*")]
    #[endpoint(voteFor)]
    fn vote_for_endpoint(&self, proposal_id: u64) {
        self.require_sealed();
        self.vote(proposal_id, VoteType::For)
    }

    #[payable("*")]
    #[endpoint(voteAgainst)]
    fn vote_against_endpoint(&self, proposal_id: u64) {
        self.require_sealed();
        self.vote(proposal_id, VoteType::Against)
    }

    #[endpoint(sign)]
    fn sign_endpoint(&self, proposal_id: u64) {
        self.require_sealed();
        self.sign(proposal_id);
    }

    #[endpoint(execute)]
    fn execute_endpoint(&self, proposal_id: u64, actions: MultiValueManagedVec<Action<Self::Api>>) {
        require!(!actions.is_empty(), "no actions to execute");
        require!(!self.proposals(proposal_id).is_empty(), "proposal not found");
        self.require_sealed();

        let actions = actions.into_vec();
        let actions_hash = self.calculate_actions_hash(&actions);
        let mut proposal = self.proposals(proposal_id).get();
        let actual_permissions = self.get_actual_permissions(&proposal, &actions);

        require!(proposal.actions_hash == actions_hash, "actions have been corrupted");
        require!(proposal.permissions.eq(&actual_permissions), "untruthful permissions announced");

        let status = self.get_proposal_status(&proposal);

        require!(status == ProposalStatus::Succeeded, "proposal is not executable");

        self.execute_actions(&actions);
        proposal.was_executed = true;

        self.proposals(proposal_id).set(&proposal);
        self.emit_execute_event(proposal);
    }

    #[payable("*")]
    #[endpoint(redeem)]
    fn redeem_endpoint(&self) {
        let payments = self.call_value().all_esdt_transfers();

        for payment in payments.into_iter() {
            self.redeem_vote_tokens(payment);
        }
    }

    #[view(getProposal)]
    fn get_proposal_view(&self, proposal_id: u64) -> OptionalValue<MultiValue6<ManagedBuffer, ManagedBuffer, ManagedAddress, u64, u64, bool>> {
        if !self.proposal_exists(proposal_id) {
            OptionalValue::None
        } else {
            let proposal = self.proposals(proposal_id).get();
            OptionalValue::Some(
                (
                    proposal.content_hash,
                    proposal.actions_hash,
                    proposal.proposer,
                    proposal.starts_at,
                    proposal.ends_at,
                    proposal.was_executed,
                )
                    .into(),
            )
        }
    }

    #[view(getProposalStatus)]
    fn get_proposal_status_view(&self, proposal_id: u64) -> ProposalStatus {
        require!(!self.proposals(proposal_id).is_empty(), "proposal not found");

        self.get_proposal_status(&self.proposals(proposal_id).get())
    }

    #[view(getProposalVotes)]
    fn get_proposal_votes_view(&self, proposal_id: u64) -> MultiValue2<BigUint, BigUint> {
        let proposal = self.proposals(proposal_id).get();

        (proposal.votes_for, proposal.votes_against).into()
    }

    #[view(getProposalSigners)]
    fn get_proposal_signers_view(&self, proposal_id: u64) -> MultiValueEncoded<ManagedAddress> {
        let proposal = self.proposals(proposal_id).get();
        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let proposer_roles = self.user_roles(proposer_id);
        let mut signers = MultiValueEncoded::new();

        for role in proposer_roles.iter() {
            for signer_id in self.proposal_signers(proposal.id, &role).iter() {
                let address = self.users().get_user_address_unchecked(signer_id);
                if !signers.to_vec().contains(&address) {
                    signers.push(address);
                }
            }
        }
        signers
    }

    #[view(getProposalSignatureRoleCounts)]
    fn get_proposal_signature_role_counts_view(&self, proposal_id: u64) -> MultiValueEncoded<MultiValue2<ManagedBuffer, usize>> {
        let proposal = self.proposals(proposal_id).get();
        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let proposer_roles = self.user_roles(proposer_id);
        let mut signers = MultiValueEncoded::new();

        for role in proposer_roles.iter() {
            let signer_count = self.proposal_signers(proposal.id, &role).len();
            if signer_count > 0 {
                signers.push((role, signer_count).into());
            }
        }
        signers
    }

    #[payable("EGLD")]
    #[endpoint(issueNftVoteToken)]
    fn issue_nft_vote_token(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer) {
        let caller = self.blockchain().get_caller();

        self.vote_nft_token().issue(
            EsdtTokenType::NonFungible,
            self.call_value().egld_value(),
            token_name,
            token_ticker,
            18usize,
            Option::Some(self.callbacks().vote_nft_issue_callback(&caller)),
        );
    }

    #[callback]
    fn vote_nft_issue_callback(&self, initial_caller: &ManagedAddress, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => self.vote_nft_token().set_token_id(&token_id),
            ManagedAsyncCallResult::Err(_) => {
                let egld_returned = self.call_value().egld_value();
                if egld_returned > 0 {
                    self.send().direct_egld(&initial_caller, &egld_returned);
                }
            }
        }
    }
}
