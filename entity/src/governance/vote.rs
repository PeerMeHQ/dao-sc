elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use super::events;
use super::proposal;
use super::proposal::ProposalStatus;
use crate::config;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, Clone)]
pub enum VoteType {
    For = 1,
    Against = 2,
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub struct VoteNFTAttributes<M: ManagedTypeApi> {
    pub proposal_id: u64,
    pub vote_type: VoteType,
    pub vote_weight: BigUint<M>,
    pub voter: ManagedAddress<M>,
    pub payment: EsdtTokenPayment<M>,
}

#[elrond_wasm::module]
pub trait VoteModule: config::ConfigModule + proposal::ProposalModule + events::GovEventsModule {
    fn vote(&self, proposal_id: u64, vote_type: VoteType) {
        self.require_sealed();
        self.require_payment_token_governance_token();

        let voter = self.blockchain().get_caller();
        let payment = self.call_value().payment();
        let vote_weight = payment.amount.clone();
        let mut proposal = self.proposals(proposal_id).get();

        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, "proposal is not active");
        require!(vote_weight != 0u64, "can not vote with zero");

        match vote_type {
            VoteType::For => proposal.votes_for += &vote_weight,
            VoteType::Against => proposal.votes_against += &vote_weight,
        }

        self.create_vote_nft_and_send(&voter, proposal_id, vote_type.clone(), vote_weight.clone(), payment.clone());
        self.proposals(proposal_id).set(&proposal);
        self.emit_vote_event(proposal, vote_type, payment, vote_weight);
    }

    fn create_vote_nft_and_send(
        &self,
        voter: &ManagedAddress,
        proposal_id: u64,
        vote_type: VoteType,
        vote_weight: BigUint,
        payment: EsdtTokenPayment<Self::Api>,
    ) {
        let attr = VoteNFTAttributes {
            proposal_id,
            vote_type,
            vote_weight,
            voter: voter.clone(),
            payment,
        };

        self.vote_nft_token().nft_create_and_send(&voter, BigUint::from(1u64), &attr);
    }
}
