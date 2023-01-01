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

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct Action<M: ManagedTypeApi> {
    pub destination: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
    pub value: BigUint<M>,
    pub payments: ManagedVec<M, EsdtTokenPayment<M>>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
    pub gas_limit: u64,
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

        let voting_period_minutes = policies
            .iter()
            .map(|p| p.voting_period_minutes)
            .max()
            .unwrap_or_else(|| self.voting_period_in_minutes().get());

        let starts_at = self.blockchain().get_block_timestamp();
        let ends_at = starts_at + voting_period_minutes as u64 * 60;
        let sanitized_permissions = permissions.into_iter().filter(|perm| !perm.is_empty()).collect();

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
            permissions: sanitized_permissions,
        };

        self.proposals(proposal_id).set(&proposal);
        self.next_proposal_id().set(proposal_id + 1);

        proposal
    }

    fn get_proposal_status(&self, proposal: &Proposal<Self::Api>) -> ProposalStatus {
        if proposal.was_executed {
            return ProposalStatus::Executed;
        }

        let has_gov_token = !self.gov_token_id().is_empty();
        let has_actions = !proposal.actions_hash.is_empty() || !proposal.permissions.is_empty();
        let is_leaderless = self.is_leaderless();

        let (has_permission, has_token_weighted_policy) = if !is_leaderless && has_actions {
            self.has_fulfilled_permissions(&proposal)
        } else {
            (false, false)
        };

        // early succeed if signer majority & no token weighted policy
        if has_permission && !has_token_weighted_policy {
            return ProposalStatus::Succeeded;
        }

        if self.is_proposal_active(&proposal) {
            return ProposalStatus::Active;
        }

        if (is_leaderless || !has_actions) && has_gov_token {
            return match self.has_sufficient_votes(&proposal, &self.quorum().get()) {
                true => ProposalStatus::Succeeded,
                false => ProposalStatus::Defeated,
            };
        }

        if has_permission {
            return ProposalStatus::Succeeded;
        }

        ProposalStatus::Defeated
    }

    fn has_fulfilled_permissions(&self, proposal: &Proposal<Self::Api>) -> (bool, bool) {
        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let proposer_roles = self.user_roles(proposer_id);

        // require signer majority if no permissions announced
        if proposal.permissions.is_empty() {
            let has_signer_majority = proposer_roles
                .iter()
                .map(|role| self.has_signer_majority_for_role(&proposal, &role))
                .all(|res| res == true);

            return (has_signer_majority, false);
        }

        let mut fulfilled_all = true;
        let mut has_token_weighted_policy = false;

        for permission in proposal.permissions.into_iter() {
            let fulfilled_perm = proposer_roles
                .iter()
                .map(|role| {
                    if let Some(policy) = self.policies(&role).get(&permission) {
                        match policy.method {
                            PolicyMethod::Weight => {
                                has_token_weighted_policy = true;
                                self.has_sufficient_votes(&proposal, &policy.quorum)
                            }
                            PolicyMethod::One => self.proposal_signers(proposal.id, &role).contains(&proposer_id),
                            PolicyMethod::All => self.proposal_signers(proposal.id, &role).len() >= self.roles_member_amount(&role).get(),
                            PolicyMethod::Quorum => BigUint::from(self.proposal_signers(proposal.id, &role).len()) >= policy.quorum,
                        }
                    } else {
                        self.has_signer_majority_for_role(&proposal, &role)
                    }
                })
                .all(|fulfilled| fulfilled == true);

            if !fulfilled_perm {
                fulfilled_all = false;
            }
        }

        (fulfilled_all, has_token_weighted_policy)
    }

    fn execute_actions(&self, actions: &ManagedVec<Action<Self::Api>>) {
        let gov_token_id = self.gov_token_id().get();

        for action in actions.iter() {
            let mut call = self
                .send()
                .contract_call::<()>(action.destination, action.endpoint)
                .with_arguments_raw(ManagedArgBuffer::from(action.arguments))
                .with_gas_limit(action.gas_limit);

            for payment in action.payments.iter() {
                if payment.token_identifier == gov_token_id {
                    self.require_gov_tokens_available(&payment.amount, payment.token_nonce);
                }

                call = call.add_esdt_token_transfer(payment.token_identifier, payment.token_nonce, payment.amount);
            }

            if action.value > 0 {
                call = call.with_egld_transfer(action.value);
            }

            call.transfer_execute();
        }
    }

    fn can_propose(&self, proposer: &ManagedAddress, actions_hash: &ManagedBuffer, permissions: &ManagedVec<ManagedBuffer>) -> (bool, ManagedVec<Policy<Self::Api>>) {
        let proposer_id = self.users().get_user_id(proposer);
        let proposer_roles = self.user_roles(proposer_id);
        let mut policies = ManagedVec::new();

        // no actions -> always allowed
        if actions_hash.is_empty() && permissions.is_empty() {
            return (true, policies);
        }

        if self.is_leaderless() {
            return (true, policies);
        }

        let mut allowed = false;

        for role in proposer_roles.iter() {
            if role == ManagedBuffer::from(ROLE_BUILTIN_LEADER) {
                allowed = true;
            }

            for permission in permissions.into_iter() {
                if let Some(policy) = self.policies(&role).get(&permission) {
                    policies.push(policy);
                    allowed = true;
                }
            }
        }

        (allowed, policies)
    }

    fn calculate_actions_hash(&self, actions: &ManagedVec<Action<Self::Api>>) -> ManagedBuffer<Self::Api> {
        let mut serialized = ManagedBuffer::new();

        for action in actions.iter() {
            serialized.append(&sc_format!("{}{}{}", action.destination.as_managed_buffer(), action.endpoint, action.value));

            for payment in action.payments.iter() {
                serialized.append(&sc_format!("{}{}{}", payment.token_identifier, payment.token_nonce, payment.amount));
            }

            for arg in action.arguments.into_iter() {
                serialized.append(&arg);
            }
        }

        self.crypto().keccak256(&serialized).as_managed_buffer().clone()
    }

    fn get_actual_permissions(&self, proposal: &Proposal<Self::Api>, actions: &ManagedVec<Action<Self::Api>>) -> ManagedVec<ManagedBuffer> {
        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let proposer_roles = self.user_roles(proposer_id);
        let mut actual_permissions = ManagedVec::new();

        for action in actions.into_iter() {
            let mut has_permission_for_action = false;

            for role in proposer_roles.iter() {
                for permission in self.policies(&role).keys() {
                    if actual_permissions.contains(&permission) {
                        continue;
                    }

                    let permission_details = self.permission_details(&permission).get();

                    if self.does_permission_apply_to_action(&permission_details, &action) {
                        actual_permissions.push(permission);
                        has_permission_for_action = true;
                    }
                }

                // leader does not need permission for all actions defined
                if role == ManagedBuffer::from(ROLE_BUILTIN_LEADER) {
                    has_permission_for_action = true;
                }
            }

            require!(has_permission_for_action, "no permission for action");
        }

        actual_permissions
    }

    fn does_permission_apply_to_action(&self, permission_details: &PermissionDetails<Self::Api>, action: &Action<Self::Api>) -> bool {
        let mut is_pure_value_perm = true;

        // check value/EGLD mismatch
        if permission_details.value != 0 && action.value > permission_details.value {
            return false;
        }

        // check destination mismatch
        if !permission_details.destination.is_zero() && action.destination != permission_details.destination {
            return false;
        }

        // check endpoint mismatch
        if !permission_details.endpoint.is_empty() {
            is_pure_value_perm = false;

            if action.endpoint != permission_details.endpoint {
                return false;
            }
        }

        // check arguments mismatch. ignored if permission contains no arguments.
        // the permission can scope the argument sequence down as far as needed:
        //      - passes: arg1, arg2 (permission) -> arg1, arg2, arg3 (action)
        //      - fails: arg1, arg2 (permission) -> arg1, arg3 (action)
        //      - fails: arg1, arg2 (permission) -> arg1 (action)
        if !permission_details.arguments.is_empty() {
            is_pure_value_perm = false;

            for (i, perm_arg) in permission_details.arguments.into_iter().enumerate() {
                if let Option::Some(arg_at_index) = action.arguments.try_get(i).as_deref() {
                    let applies = arg_at_index == &perm_arg;

                    if applies {
                        continue;
                    }
                }

                return false;
            }
        }

        // check payments mismatch. ignored if permission contains no payments.
        // returns false, if a payment is not in the permissions or exceeds payment amount.
        if !permission_details.payments.is_empty() {
            is_pure_value_perm = false;

            let applies = action.payments.into_iter().all(|payment| {
                if let Some(guard) = permission_details.payments.into_iter().find(|p| p.token_identifier == payment.token_identifier) {
                    payment.amount <= guard.amount
                } else {
                    false
                }
            });

            if !applies {
                return false;
            }
        }

        // ensure that pure value transfer permissions are
        // not applied to smart contract calls with 0 value.
        if is_pure_value_perm && action.value == 0 {
            return false;
        }

        true
    }

    fn is_proposal_active(&self, proposal: &Proposal<Self::Api>) -> bool {
        let current_time = self.blockchain().get_block_timestamp();

        current_time >= proposal.starts_at && current_time < proposal.ends_at
    }

    fn has_sufficient_votes(&self, proposal: &Proposal<Self::Api>, quorum: &BigUint) -> bool {
        let total_votes = &proposal.votes_for + &proposal.votes_against;

        if total_votes < 1 {
            return false;
        }

        let vote_for_percent = &proposal.votes_for * &BigUint::from(100u64) / &total_votes;
        let vote_for_percent_to_pass = BigUint::from(50u64);

        vote_for_percent >= vote_for_percent_to_pass && &proposal.votes_for >= quorum
    }

    fn has_signer_majority_for_role(&self, proposal: &Proposal<Self::Api>, role: &ManagedBuffer) -> bool {
        self.proposal_signers(proposal.id, &role).len() >= self.get_signer_majority_for_role(&role)
    }

    fn get_signer_majority_for_role(&self, role: &ManagedBuffer) -> usize {
        self.roles_member_amount(&role).get() / 2 + 1
    }

    fn require_proposed_via_trusted_host(
        &self,
        trusted_host_id: &ManagedBuffer,
        content_hash: &ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: &ManagedBuffer,
        permissions: &ManagedVec<ManagedBuffer>,
    ) {
        let proposer = self.blockchain().get_caller();
        let entity_address = self.blockchain().get_sc_address();
        let trusted_host_signature = ManagedByteArray::try_from(content_sig).unwrap_or_default();

        let mut trusted_host_signable = sc_format!(
            "{}{}{}{}{}",
            proposer.as_managed_buffer(),
            entity_address.as_managed_buffer(),
            trusted_host_id,
            content_hash,
            actions_hash
        );

        for perm in permissions.into_iter() {
            trusted_host_signable.append(&perm);
        }

        self.require_signed_by_trusted_host(&trusted_host_signable, &trusted_host_signature);
    }

    fn proposal_exists(&self, proposal_id: u64) -> bool {
        !self.proposals(proposal_id).is_empty()
    }
}
