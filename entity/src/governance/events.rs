elrond_wasm::imports!();

use super::proposal::Proposal;
use super::vote::{VoteNFTAttributes, VoteType};


#[elrond_wasm::module]
pub trait GovEventsModule {
    fn emit_propose_event(&self, proposal: Proposal<Self::Api>, payment: EsdtTokenPayment<Self::Api>, weight: BigUint) {
        self.propose_event(
            self.blockchain().get_caller(),
            proposal,
            payment,
            weight,
            self.blockchain().get_block_timestamp(),
            self.blockchain().get_block_nonce(),
        );
    }

    fn emit_vote_event(&self, proposal: Proposal<Self::Api>, vote_type: VoteType, payment: EsdtTokenPayment<Self::Api>, weight: BigUint) {
        match vote_type {
            VoteType::For => {
                self.upvote_event(
                    self.blockchain().get_caller(),
                    proposal,
                    payment,
                    weight,
                    self.blockchain().get_block_timestamp(),
                    self.blockchain().get_block_nonce(),
                );
            }
            VoteType::Against => {
                self.downvote_event(
                    self.blockchain().get_caller(),
                    proposal,
                    payment,
                    weight,
                    self.blockchain().get_block_timestamp(),
                    self.blockchain().get_block_nonce(),
                );
            }
        }
    }

    fn emit_execute_event(&self, proposal: Proposal<Self::Api>) {
        self.execute_event(
            self.blockchain().get_caller(),
            proposal,
            self.blockchain().get_block_timestamp(),
            self.blockchain().get_block_nonce(),
        );
    }

    fn emit_redeem_event(&self, proposal: Proposal<Self::Api>, payment: EsdtTokenPayment<Self::Api>, vote_attr: VoteNFTAttributes<Self::Api>) {
        self.redeem_event(
            self.blockchain().get_caller(),
            proposal,
            payment,
            vote_attr,
            self.blockchain().get_block_timestamp(),
            self.blockchain().get_block_nonce(),
        );
    }

    #[event("propose")]
    fn propose_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] proposal: Proposal<Self::Api>,
        #[indexed] payment: EsdtTokenPayment<Self::Api>,
        #[indexed] weight: BigUint,
        #[indexed] timestamp: u64,
        #[indexed] epoch: u64,
    );

    #[event("upvote")]
    fn upvote_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] proposal: Proposal<Self::Api>,
        #[indexed] payment: EsdtTokenPayment<Self::Api>,
        #[indexed] weight: BigUint,
        #[indexed] timestamp: u64,
        #[indexed] epoch: u64,
    );

    #[event("downvote")]
    fn downvote_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] proposal: Proposal<Self::Api>,
        #[indexed] payment: EsdtTokenPayment<Self::Api>,
        #[indexed] weight: BigUint,
        #[indexed] timestamp: u64,
        #[indexed] epoch: u64,
    );

    #[event("execute")]
    fn execute_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] proposal: Proposal<Self::Api>,
        #[indexed] timestamp: u64,
        #[indexed] epoch: u64,
    );

    #[event("redeem")]
    fn redeem_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] proposal: Proposal<Self::Api>,
        #[indexed] payment: EsdtTokenPayment<Self::Api>,
        #[indexed] vote_attr: VoteNFTAttributes<Self::Api>,
        #[indexed] timestamp: u64,
        #[indexed] epoch: u64,
    );
}
