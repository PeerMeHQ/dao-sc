elrond_wasm::imports!();

use self::{proposal::ActionAsMultiArg, vote::VoteType};
use crate::config::{self, VOTING_PERIOD_MINUTES_DEFAULT};
use proposal::{Action, ProposalStatus};

pub mod events;
pub mod proposal;
pub mod vote;

#[elrond_wasm::module]
pub trait GovernanceModule: config::ConfigModule + events::GovEventsModule + proposal::ProposalModule + vote::VoteModule {
    fn init_governance_module(&self, gov_token_id: &TokenIdentifier, initial_tokens: &BigUint) {
        let initial_quorum = initial_tokens / &BigUint::from(20u64); // 5% of initial tokens
        let initial_min_tokens_for_proposing = initial_tokens / &BigUint::from(1000u64); // 0.1% of initial tokens

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
    fn change_voting_period_in_minutes_endpoint(&self, value: u32) {
        self.require_caller_self_or_unsealed();
        self.try_change_voting_period_in_minutes(value);
    }

    #[payable("*")]
    #[endpoint(propose)]
    fn propose_endpoint(
        &self,
        title: ManagedBuffer,
        description: ManagedBuffer,
        #[var_args] actions: MultiValueManagedVec<Action<Self::Api>>,
    ) -> u64 {
        self.require_sealed();
        self.require_payment_token_governance_token();

        let proposer = self.blockchain().get_caller();
        let payment = self.call_value().payment();
        let vote_weight = payment.amount.clone();
        let proposal = self.create_proposal(title, description, actions.into_vec(), vote_weight.clone());
        let proposal_id = proposal.id;

        self.protected_vote_tokens().update(|current| *current += &payment.amount);
        self.create_vote_nft_and_send(&proposer, proposal.id, VoteType::For, vote_weight.clone(), payment.clone());
        self.emit_propose_event(proposal, payment, vote_weight);

        proposal_id
    }

    #[payable("*")]
    #[endpoint(voteFor)]
    fn vote_for_endpoint(&self, proposal_id: u64) {
        self.vote(proposal_id, VoteType::For)
    }

    #[payable("*")]
    #[endpoint(voteAgainst)]
    fn vote_against_endpoint(&self, proposal_id: u64) {
        self.vote(proposal_id, VoteType::Against)
    }

    #[endpoint(execute)]
    fn execute_endpoint(&self, proposal_id: u64) {
        self.require_sealed();
        require!(!self.proposals(proposal_id).is_empty(), "proposal not found");

        let mut proposal = self.proposals(proposal_id).get();
        let status = self.get_proposal_status(&proposal);

        require!(status == ProposalStatus::Succeeded, "proposal is not executable");

        self.execute_proposal(&proposal);
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
                    proposal.title,
                    proposal.description,
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

    #[view(getProposalActions)]
    fn get_proposal_actions_view(&self, proposal_id: u64) -> MultiValueEncoded<ActionAsMultiArg<Self::Api>> {
        if !self.proposal_exists(proposal_id) {
            return MultiValueEncoded::new();
        }

        let actions = self.proposals(proposal_id).get().actions;
        let mut actions_as_multiarg = MultiValueEncoded::new();

        for action in actions.into_iter() {
            actions_as_multiarg.push(action.into_multiarg());
        }

        actions_as_multiarg
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
                if egld_returned > 0u32 {
                    self.send().direct_egld(&initial_caller, &egld_returned, &[]);
                }
            }
        }
    }
}
