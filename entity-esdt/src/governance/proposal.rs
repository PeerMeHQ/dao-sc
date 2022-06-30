elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::api::KECCAK256_RESULT_LEN;

use crate::config;
use crate::permission;
use crate::permission::PermissionDetails;
use crate::permission::{Policy, PolicyMethod, ROLE_BUILTIN_LEADER};
use core::convert::TryFrom;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Proposal<M: ManagedTypeApi> {
    pub id: u64,
    pub proposer: ManagedAddress<M>,
    pub content_hash: ManagedBuffer<M>,
    pub actions_hash: ManagedBuffer<M>,
    pub starts_at: u64,
    pub ends_at: u64,
    pub was_executed: bool,
    pub votes_for: BigUint<M>,
    pub votes_against: BigUint<M>,
    pub permissions: ManagedVec<M, ManagedBuffer<M>>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct Action<M: ManagedTypeApi> {
    pub destination: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
    pub gas_limit: u64,
    pub token_id: EgldOrEsdtTokenIdentifier<M>,
    pub token_nonce: u64,
    pub amount: BigUint<M>,
}

pub type ActionAsMultiArg<M> =
    MultiValue8<ManagedAddress<M>, ManagedBuffer<M>, u64, EgldOrEsdtTokenIdentifier<M>, u64, BigUint<M>, usize, MultiValueManagedVec<M, ManagedBuffer<M>>>;

impl<M: ManagedTypeApi> Action<M> {
    pub fn into_multiarg(self) -> ActionAsMultiArg<M> {
        (
            self.destination,
            self.endpoint,
            self.gas_limit,
            self.token_id,
            self.token_nonce,
            self.amount,
            self.arguments.len(),
            MultiValueManagedVec::from(self.arguments),
        )
            .into()
    }
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub enum ProposalStatus {
    Pending,
    Active,
    Defeated,
    Succeeded,
    Executed,
}

#[elrond_wasm::module]
pub trait ProposalModule: config::ConfigModule + permission::PermissionModule {
    fn create_proposal(
        &self,
        content_hash: ManagedBuffer,
        actions_hash: ManagedBuffer,
        vote_weight: BigUint,
        permissions: ManagedVec<ManagedBuffer>,
        policies: &ManagedVec<Policy<Self::Api>>,
    ) -> Proposal<Self::Api> {
        let proposer = self.blockchain().get_caller();
        let proposal_id = self.next_proposal_id().get();

        if !actions_hash.is_empty() {
            require!(actions_hash.len() == KECCAK256_RESULT_LEN, "invalid actions hash");
        }

        let voting_period_minutes = policies.iter()
            .map(|p| p.voting_period_minutes)
            .max()
            .unwrap_or_else(|| self.voting_period_in_minutes().get());

        let starts_at = self.blockchain().get_block_timestamp();
        let ends_at = starts_at + voting_period_minutes as u64 * 60;

        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.clone(),
            content_hash,
            starts_at,
            ends_at,
            was_executed: false,
            actions_hash,
            votes_for: vote_weight.clone(),
            votes_against: BigUint::zero(),
            permissions,
        };

        self.proposals(proposal_id.clone()).set(&proposal);
        self.next_proposal_id().set(proposal_id + 1);

        proposal
    }

    fn get_proposal_status(&self, proposal: &Proposal<Self::Api>) -> ProposalStatus {
        let current_time = self.blockchain().get_block_timestamp();

        if proposal.was_executed {
            return ProposalStatus::Executed;
        }

        if current_time >= proposal.starts_at && current_time < proposal.ends_at {
            return ProposalStatus::Active;
        }

        if proposal.actions_hash.is_empty() || proposal.permissions.is_empty() {
            return match self.has_sufficient_votes(&proposal, &self.quorum().get()) {
                true => ProposalStatus::Succeeded,
                false => ProposalStatus::Defeated,
            };
        }

        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let mut fulfilled_all = true;

        for permission in proposal.permissions.into_iter() {
            let fulfilled_perm = self.user_roles(proposer_id).iter()
                .map(|role| if let Some(policy) = self.policies(&role).get(&permission) {
                    match policy.method {
                        PolicyMethod::Weight => self.has_sufficient_votes(&proposal, &policy.quorum),
                        PolicyMethod::One => self.proposal_signers(proposal.id, &role).contains(&proposer_id),
                        PolicyMethod::All => self.proposal_signers(proposal.id, &role).len() >= self.roles_member_amount(&role).get(),
                        PolicyMethod::Quorum => BigUint::from(self.proposal_signers(proposal.id, &role).len()) >= policy.quorum,
                    }
                } else {
                    true
                })
                .find(|fulfilled| !fulfilled)
                .is_none();

            if !fulfilled_perm {
                fulfilled_all = false;
            }
        }

        if fulfilled_all {
            return ProposalStatus::Succeeded;
        }

        ProposalStatus::Defeated
    }

    fn execute_actions(&self, actions: &ManagedVec<Action<Self::Api>>) {
        let gov_token_id = self.governance_token_id().get();

        for action in actions.iter() {
            let mut call = self
                .send()
                .contract_call::<()>(action.destination, action.endpoint)
                .with_arguments_raw(ManagedArgBuffer::from(action.arguments))
                .with_gas_limit(action.gas_limit);

            if action.amount > 0 {
                if action.token_id == gov_token_id {
                    self.require_governance_tokens_available(&action.amount);
                }

                call = call.with_egld_or_single_esdt_token_transfer(action.token_id, action.token_nonce, action.amount)
            }

            call.transfer_execute()
        }
    }

    fn can_propose(&self, proposer: &ManagedAddress, actions_hash: &ManagedBuffer, permissions: &ManagedVec<ManagedBuffer>) -> (ManagedVec<Policy<Self::Api>>, bool) {
        let proposer_id = self.users().get_user_id(proposer);
        let user_roles = self.user_roles(proposer_id);
        let mut policies = ManagedVec::new();

        if actions_hash.is_empty() && permissions.is_empty() {
            return (policies, true);
        }

        if self.does_leader_role_exist() && user_roles.is_empty() {
            return (policies, false);
        }

        let mut allowed = false;

        for role in user_roles.iter() {
            if role == ManagedBuffer::from(ROLE_BUILTIN_LEADER) {
                allowed = true;
            }

            for permission in permissions.into_iter() {
                let policy = self.policies(&role).get(&permission);

                if policy.is_some() {
                    policies.push(policy.unwrap());
                    allowed = true;
                }
            }
        }

        (policies, allowed)
    }

    fn calculate_actions_hash(&self, actions: &ManagedVec<Action<Self::Api>>) -> ManagedBuffer<Self::Api> {
        let mut serialized = ManagedBuffer::new();

        for action in actions.iter() {
            let formatted = sc_format!("{:x}{}{}{}{}", action.destination.as_managed_buffer(), action.amount, action.token_id, action.token_nonce, action.endpoint);

            serialized.append(&formatted);

            for arg in action.arguments.into_iter() {
                serialized.append(&sc_format!("{:x}", arg));
            }
        }

        self.crypto().keccak256(&serialized).as_managed_buffer().clone()
    }

    fn get_actual_permissions(&self, proposal: &Proposal<Self::Api>, actions: &ManagedVec<Action<Self::Api>>) -> ManagedVec<ManagedBuffer> {
        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let mut actual_permissions = ManagedVec::new();

        for role in self.user_roles(proposer_id).iter() {
            for permission in self.policies(&role).keys() {
                let permission_details = self.permission_details(&permission).get();

                for action in actions.into_iter() {
                    if self.does_permission_apply_to_action(&permission_details, &action) && !actual_permissions.contains(&permission) {
                        actual_permissions.push(permission.clone());
                    }
                }
            }
        }

        actual_permissions
    }

    fn does_permission_apply_to_action(&self, permission_details: &PermissionDetails<Self::Api>, action: &Action<Self::Api>) -> bool {
        action.destination == permission_details.destination && action.endpoint == permission_details.endpoint
    }

    fn has_sufficient_votes(&self, proposal: &Proposal<Self::Api>, quorum: &BigUint) -> bool {
        let total_votes = &proposal.votes_for + &proposal.votes_against;
        let vote_for_percent = &proposal.votes_for * &BigUint::from(100u64) / &total_votes;
        let vote_for_percent_to_pass = BigUint::from(50u64);

        vote_for_percent >= vote_for_percent_to_pass && &proposal.votes_for >= quorum
    }

    fn require_proposed_via_trusted_host(
        &self,
        trusted_host_id: &ManagedBuffer,
        content_hash: &ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: &ManagedBuffer,
        permissions: &ManagedVec<ManagedBuffer>
    ) {
        let proposer = self.blockchain().get_caller();
        let entity_token_id = self.token().get_token_id();
        let trusted_host_signature = ManagedByteArray::try_from(content_sig).unwrap();
        let mut trusted_host_signable = sc_format!("{:x}{:x}{:x}{:x}{:x}", proposer, entity_token_id, trusted_host_id, content_hash, actions_hash);

        for perm in permissions.into_iter() {
            trusted_host_signable.append(&sc_format!("{:x}", perm));
        }

        self.require_signed_by_trusted_host(&trusted_host_signable, &trusted_host_signature);
    }

    fn proposal_exists(&self, proposal_id: u64) -> bool {
        !self.proposals(proposal_id).is_empty()
    }
}
