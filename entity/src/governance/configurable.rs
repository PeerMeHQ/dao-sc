elrond_wasm::imports!();

use super::storage;
use crate::config::{
    DEFAULT_PROPOSAL_MIN_TOKENS, DEFAULT_VOTING_DELAY, DEFAULT_VOTING_LOCKTIME, DEFAULT_VOTING_MAX_ACTIONS, DEFAULT_VOTING_PERIOD,
    DEFAULT_VOTING_QUORUM,
};

#[elrond_wasm::module]
pub trait GovConfigurableModule: storage::GovStorageModule {
    #[endpoint(changeQuorum)]
    fn change_quorum(&self, new_value: BigUint) {
        self.require_caller_self();
        self.try_change_quorum(new_value);
    }

    #[endpoint(changeMinTokenBalanceForProposing)]
    fn change_min_token_balance_for_proposing(&self, new_value: BigUint) {
        self.require_caller_self();
        self.try_change_min_token_balance_for_proposing(new_value);
    }

    #[endpoint(changeMaxActionsPerProposal)]
    fn change_max_actions_per_proposal(&self, new_value: usize) {
        self.require_caller_self();
        self.try_change_max_actions_per_proposal(new_value);
    }

    #[endpoint(changeVotingDelayInBlocks)]
    fn change_voting_delay_in_blocks(&self, new_value: u64) {
        self.require_caller_self();
        self.try_change_voting_delay_in_blocks(new_value);
    }

    #[endpoint(changeVotingPeriodInBlocks)]
    fn change_voting_period_in_blocks(&self, new_value: u64) {
        self.require_caller_self();
        self.try_change_voting_period_in_blocks(new_value);
    }

    #[endpoint(changeLockTimeAfterVotingEndsInBlocks)]
    fn change_lock_time_after_voting_ends_in_blocks(&self, new_value: u64) {
        self.require_caller_self();
        self.try_change_lock_time_after_voting_ends_in_blocks(new_value);
    }

    fn init_governance_module(&self, governance_token_id: &TokenIdentifier) {
        require!(governance_token_id.is_valid_esdt_identifier(), "invalid edst");

        self.governance_token_id().set_if_empty(&governance_token_id);

        self.try_change_quorum(BigUint::from(DEFAULT_VOTING_QUORUM));
        self.try_change_min_token_balance_for_proposing(BigUint::from(DEFAULT_PROPOSAL_MIN_TOKENS));
        self.try_change_max_actions_per_proposal(DEFAULT_VOTING_MAX_ACTIONS);
        self.try_change_voting_delay_in_blocks(DEFAULT_VOTING_DELAY);
        self.try_change_voting_period_in_blocks(DEFAULT_VOTING_PERIOD);
        self.try_change_lock_time_after_voting_ends_in_blocks(DEFAULT_VOTING_LOCKTIME);
    }

    fn require_caller_self(&self) {
        let caller = self.blockchain().get_caller();
        let sc_address = self.blockchain().get_sc_address();

        require!(caller == sc_address, "action not allowed");
    }

    fn try_change_quorum(&self, new_value: BigUint) {
        require!(new_value != 0, "invalid quorum");
        self.quorum().set(&new_value);
    }

    fn try_change_min_token_balance_for_proposing(&self, new_value: BigUint) {
        require!(new_value != 0, "min token balance for proposing can not be 0");
        self.min_token_balance_for_proposing().set(&new_value);
    }

    fn try_change_max_actions_per_proposal(&self, new_value: usize) {
        require!(new_value != 0, "max actions per proposal can not be 0");
        self.max_actions_per_proposal().set(&new_value);
    }

    fn try_change_voting_delay_in_blocks(&self, new_value: u64) {
        require!(new_value != 0, "voting delay in blocks can not be 0");
        self.voting_delay_in_blocks().set(&new_value);
    }

    fn try_change_voting_period_in_blocks(&self, new_value: u64) {
        require!(new_value != 0, "voting period (in blocks) can not be 0");
        self.voting_period_in_blocks().set(&new_value);
    }

    fn try_change_lock_time_after_voting_ends_in_blocks(&self, new_value: u64) {
        require!(new_value != 0, "lock time after voting ends (in blocks) can not be 0");
        self.lock_time_after_voting_ends_in_blocks().set(&new_value);
    }
}
