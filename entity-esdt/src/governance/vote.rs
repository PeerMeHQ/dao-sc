elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use super::events;
use super::proposal;
use super::proposal::Proposal;
use super::proposal::ProposalStatus;
use crate::config;
use crate::permission;

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
pub trait VoteModule: config::ConfigModule + permission::PermissionModule + proposal::ProposalModule + events::GovEventsModule {
    fn vote(&self, proposal_id: u64, vote_type: VoteType) {
        self.require_payment_token_governance_token();

        let voter = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
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
        self.protected_vote_tokens(&payment.token_identifier).update(|current| *current += &payment.amount);
        self.emit_vote_event(proposal, vote_type, payment, vote_weight);
    }

    fn sign(&self, proposal_id: u64) {
        let proposal = self.proposals(proposal_id).get();
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, "proposal is not active");

        let signer = self.blockchain().get_caller();
        self.sign_for_all_roles(&signer, &proposal);
        self.emit_sign_event(proposal);
    }

    fn sign_for_all_roles(&self, signer: &ManagedAddress, proposal: &Proposal<Self::Api>) {
        let signer_id = self.users().get_or_create_user(&signer);
        let signer_roles = self.user_roles(signer_id);

        for role in signer_roles.iter() {
            self.proposal_signers(proposal.id, &role).insert(signer_id);
        }
    }

    fn redeem_vote_tokens(&self, payment: EsdtTokenPayment<Self::Api>) {
        let caller = self.blockchain().get_caller();
        let vote_nft_id = self.vote_nft_token().get_token_id();
        let attr: VoteNFTAttributes<Self::Api> = self.vote_nft_token().get_token_attributes(payment.token_nonce.clone());
        let proposal = self.proposals(attr.proposal_id).get();
        let status = self.get_proposal_status(&proposal);

        require!(payment.token_identifier == vote_nft_id, "invalid vote position");
        require!(status != ProposalStatus::Active, "proposal is still active");

        self.protected_vote_tokens(&attr.payment.token_identifier).update(|current| *current -= &attr.payment.amount);
        self.vote_nft_token().nft_burn(payment.token_nonce, &payment.amount);
        self.send().direct_esdt(&caller, &attr.payment.token_identifier, attr.payment.token_nonce, &attr.payment.amount);
        self.emit_redeem_event(proposal, payment, attr);
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
