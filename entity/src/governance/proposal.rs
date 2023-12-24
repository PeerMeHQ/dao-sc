multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use multiversx_sc::api::KECCAK256_RESULT_LEN;

use super::events;
use crate::config;
use crate::permission;
use crate::permission::PermissionDetails;
use crate::permission::{Policy, PolicyMethod, ROLE_BUILTIN_LEADER};
use crate::plug;
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
    Canceled,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, Clone)]
pub enum VoteType {
    For = 1,
    Against = 2,
}

#[multiversx_sc::module]
pub trait ProposalModule: config::ConfigModule + permission::PermissionModule + events::GovEventsModule + plug::PlugModule {
    fn create_proposal(
        &self,
        proposer: ManagedAddress,
        trusted_host_id: ManagedBuffer,
        content_hash: ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: ManagedBuffer,
        option_id: u8,
        vote_weight: BigUint,
        permissions: ManagedVec<ManagedBuffer>,
    ) -> Proposal<Self::Api> {
        let proposal_id = self.next_proposal_id().get();

        self.require_proposed_via_trusted_host(&proposer, &trusted_host_id, &content_hash, content_sig, &actions_hash, &permissions);
        require!(!self.known_trusted_host_proposal_ids().contains(&trusted_host_id), "proposal already registered");

        let (allowed, policies) = self.can_propose(&proposer, &actions_hash, &permissions);
        require!(allowed, "action not allowed for user");

        let proposer_id = self.users().get_user_id(&proposer);
        let proposer_roles = self.user_roles(proposer_id);

        if proposer_roles.is_empty() || self.has_token_weighted_policy(&policies) {
            require!(vote_weight >= self.min_propose_weight().get(), "insufficient vote weight");
        }

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

        if !proposer_roles.is_empty() {
            self.sign_for_all_roles(&proposer, &proposal);
        }

        self.proposals(proposal_id).set(&proposal);
        self.next_proposal_id().set(proposal_id + 1);
        self.cast_poll_vote(proposal.id.clone(), option_id, vote_weight.clone());
        self.known_trusted_host_proposal_ids().insert(trusted_host_id);
        self.emit_propose_event(proposer, &proposal, vote_weight, option_id);

        proposal
    }

    fn cancel_proposal(&self, mut proposal: Proposal<Self::Api>) {
        let status = self.get_proposal_status(&proposal);

        require!(status == ProposalStatus::Active, "proposal is not active");

        proposal.ends_at = 0;
        self.proposals(proposal.id).set(&proposal);

        self.emit_cancel_event(&proposal);
    }

    fn get_proposal_status(&self, proposal: &Proposal<Self::Api>) -> ProposalStatus {
        if proposal.ends_at == 0 {
            return ProposalStatus::Canceled;
        }

        if proposal.was_executed {
            return ProposalStatus::Executed;
        }

        let has_gov_token = !self.gov_token_id().is_empty();
        let has_actions = !proposal.actions_hash.is_empty() || !proposal.permissions.is_empty();
        let is_leaderless = self.is_leaderless();

        let (has_policies, meets_policy_requirements, has_weighted_policy) = if has_actions {
            self.get_policy_requirements(&proposal, is_leaderless)
        } else {
            (false, false, false)
        };

        // early succeed if signer majority & no token weighted policy
        if meets_policy_requirements && !has_weighted_policy {
            return ProposalStatus::Succeeded;
        }

        if self.is_proposal_active(&proposal) {
            return ProposalStatus::Active;
        }

        let is_weight_based = has_gov_token || self.is_plugged();
        let is_leaderless_weighted = is_leaderless && !has_policies;

        if is_weight_based && (is_leaderless_weighted || !has_actions) {
            return match self.has_sufficient_votes(&proposal, &self.quorum().get()) {
                true => ProposalStatus::Succeeded,
                false => ProposalStatus::Defeated,
            };
        }

        if meets_policy_requirements {
            return ProposalStatus::Succeeded;
        }

        ProposalStatus::Defeated
    }

    fn get_policy_requirements(&self, proposal: &Proposal<Self::Api>, is_leaderless: bool) -> (bool, bool, bool) {
        let proposer_id = self.users().get_user_id(&proposal.proposer);
        let proposer_roles = self.user_roles(proposer_id);

        // Signer majority is required if permissions are not explicitly set and DAO is not leaderless.
        if !is_leaderless && proposal.permissions.is_empty() {
            let has_signer_majority = proposer_roles
                .iter()
                .map(|role| self.has_signer_majority_for_role(&proposal, &role))
                .all(|res| res == true);

            return (false, has_signer_majority, false);
        }

        if proposer_roles.is_empty() || proposal.permissions.is_empty() {
            return (false, false, false);
        }

        // Flags to check if all permissions are satisfied and if any token-weighted policies are applied.
        let mut is_satisfied_all = true;
        let mut has_policies = false;
        let mut has_weighted_policy = false;

        // Evaluating each permission against the proposer's roles and associated policies.
        for permission in proposal.permissions.iter() {
            let is_satisfied_perm = proposer_roles
                .iter()
                .map(|role| {
                    if let Some(policy) = self.policies(&role).get(&permission) {
                        has_policies = true;

                        match policy.method {
                            PolicyMethod::Weight => {
                                has_weighted_policy = true;
                                self.has_sufficient_votes(&proposal, &policy.quorum)
                            }
                            PolicyMethod::One => self.proposal_signers(proposal.id, &role).contains(&proposer_id),
                            PolicyMethod::All => self.proposal_signers(proposal.id, &role).len() >= self.roles_member_amount(&role).get(),
                            PolicyMethod::Quorum => BigUint::from(self.proposal_signers(proposal.id, &role).len()) >= policy.quorum,
                            PolicyMethod::Majority => self.has_signer_majority_for_role(&proposal, &role),
                        }
                    } else {
                        // If no specific policy is set, fallback to checking for a signer majority.
                        self.has_signer_majority_for_role(&proposal, &role)
                    }
                })
                .all(|satisfied| satisfied == true);

            if !is_satisfied_perm {
                is_satisfied_all = false;
            }
        }

        (has_policies, is_satisfied_all, has_weighted_policy)
    }

    fn execute_actions(&self, actions: &ManagedVec<Action<Self::Api>>) {
        let gov_token_id = self.gov_token_id().get();

        for action in actions.iter() {
            let mut call = self.send().contract_call::<()>(action.destination, action.endpoint).with_gas_limit(action.gas_limit);

            for arg in &action.arguments {
                call.push_raw_argument(arg);
            }

            if action.value > 0 {
                call.with_egld_transfer(action.value).transfer_execute();
                break;
            }

            for payment in action.payments.iter() {
                if payment.token_identifier == gov_token_id {
                    self.require_gov_tokens_available(&payment.amount, payment.token_nonce);
                }
            }

            call.with_multi_token_transfer(action.payments).transfer_execute();
        }
    }

    fn vote(&self, voter: ManagedAddress, proposal_id: u64, vote_type: VoteType, weight: BigUint, option_id: u8) {
        require!(weight > 0, "vote weight must be greater than 0");
        require!(!self.proposals(proposal_id).is_empty(), "proposal does not exist");

        let mut proposal = self.proposals(proposal_id).get();
        let min_vote_weight = self.min_vote_weight().get();

        require!(weight >= min_vote_weight, "not enought vote weight");
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, "proposal is not active");

        match vote_type {
            VoteType::For => proposal.votes_for += &weight,
            VoteType::Against => proposal.votes_against += &weight,
        }

        self.proposals(proposal_id).set(&proposal);
        self.cast_poll_vote(proposal_id, option_id, weight.clone());
        self.emit_vote_event(voter, &proposal, vote_type, weight, option_id);
    }

    fn sign(&self, proposal_id: u64, option_id: u8) {
        let proposal = self.proposals(proposal_id).get();
        require!(self.get_proposal_status(&proposal) == ProposalStatus::Active, "proposal is not active");

        let signer = self.blockchain().get_caller();

        self.sign_for_all_roles(&signer, &proposal);
        self.cast_poll_vote(proposal.id, option_id, BigUint::from(1u8));
        self.emit_sign_event(signer, &proposal, option_id);
    }

    fn sign_for_all_roles(&self, signer: &ManagedAddress, proposal: &Proposal<Self::Api>) {
        let signer_id = self.users().get_or_create_user(&signer);
        let signer_roles = self.user_roles(signer_id);

        for role in signer_roles.iter() {
            self.proposal_signers(proposal.id, &role).insert(signer_id);
        }
    }

    fn cast_poll_vote(&self, proposal_id: u64, option_id: u8, weight: BigUint) {
        if option_id == 0 || weight == 0 {
            return;
        }

        self.proposal_poll(proposal_id, option_id).update(|current| *current += weight);
    }

    fn withdraw_proposal_votes(&self, proposal_id: u64) {
        let mut voter_ids_mapper = self.withdrawable_voters(proposal_id);
        let safe_voter_ids = voter_ids_mapper.iter().collect::<ManagedVec<usize>>();

        for voter_id in safe_voter_ids.iter() {
            let voter_address = self.users().get_user_address_unchecked(voter_id);
            let withdrawn = self.withdraw_votes(&voter_address, proposal_id);

            if withdrawn.is_ok() {
                voter_ids_mapper.swap_remove(&voter_id);
            }
        }
    }

    fn withdraw_user_votes(&self, voter: &ManagedAddress) {
        let mut proposal_ids_mapper = self.withdrawable_proposal_ids(&voter);
        let safe_proposal_ids = proposal_ids_mapper.iter().collect::<ManagedVec<u64>>();

        for proposal_id in safe_proposal_ids.iter() {
            let withdrawn = self.withdraw_votes(&voter, proposal_id);

            if withdrawn.is_ok() {
                proposal_ids_mapper.swap_remove(&proposal_id);
            }
        }
    }

    fn withdraw_votes(&self, voter: &ManagedAddress, proposal_id: u64) -> SCResult<(), ()> {
        if self.proposals(proposal_id).is_empty() {
            return Ok(());
        }

        let proposal = self.proposals(proposal_id).get();
        let status = self.get_proposal_status(&proposal);

        if status == ProposalStatus::Active || status == ProposalStatus::Pending {
            return Err(());
        }

        let mut returnables: ManagedVec<EsdtTokenPayment> = ManagedVec::new();

        // keep for backwards compatibility
        let gov_token_id = self.gov_token_id().get();
        let votes_mapper = self.votes(proposal_id, &voter);
        let votes = votes_mapper.get();

        if votes > 0 {
            self.guarded_vote_tokens(&gov_token_id, 0).update(|current| *current -= &votes);
            votes_mapper.clear();
            returnables.push(EsdtTokenPayment::new(gov_token_id, 0, votes));
        }
        // *end backwards compatibility

        for vote in self.withdrawable_votes(proposal_id, &voter).iter() {
            self.guarded_vote_tokens(&vote.token_identifier, vote.token_nonce)
                .update(|current| *current -= &vote.amount);

            returnables.push(vote);
        }

        self.withdrawable_votes(proposal_id, &voter).clear();
        self.emit_withdraw_event(&proposal);

        if !returnables.is_empty() {
            self.send().direct_multi(&voter, &returnables);
        }

        return Ok(());
    }

    fn can_propose(&self, proposer: &ManagedAddress, actions_hash: &ManagedBuffer, permissions: &ManagedVec<ManagedBuffer>) -> (bool, ManagedVec<Policy<Self::Api>>) {
        // no actions -> always allowed
        if actions_hash.is_empty() && permissions.is_empty() {
            return (true, ManagedVec::new());
        }

        if self.is_leaderless() {
            return (true, ManagedVec::new());
        }

        self.get_user_policies_for_permissions(proposer, permissions)
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

    fn get_user_permissions_for_actions(
        &self,
        address: &ManagedAddress,
        actions: &ManagedVec<Action<Self::Api>>,
        has_member_approval: bool,
    ) -> (bool, ManagedVec<ManagedBuffer>) {
        let proposer_id = self.users().get_user_id(&address);
        let proposer_roles = self.user_roles(proposer_id);
        let leader_role = ManagedBuffer::from(ROLE_BUILTIN_LEADER);
        let leader_count = self.roles_member_amount(&leader_role).get();
        let mut applied_permissions = ManagedVec::new();

        for action in actions.iter() {
            let mut has_permission_for_action = false;

            for role in proposer_roles.iter() {
                for (permission, policy) in self.policies(&role).iter() {
                    if applied_permissions.contains(&permission) {
                        continue;
                    }

                    let permission_details = self.permission_details(&permission).get();

                    if self.does_permission_apply_to_action(&permission_details, &action) {
                        applied_permissions.push(permission);
                        has_permission_for_action = has_member_approval || policy.method == PolicyMethod::One;
                    }
                }

                // If proposer is a leader and there's only one leader, grant all permissions.
                if role == leader_role && leader_count == 1 {
                    has_permission_for_action = true;
                }
            }

            // If the DAO is leaderless or there's more than one leader and has_approval is true, grant permission.
            if (leader_count == 0 || leader_count > 1) && has_member_approval {
                has_permission_for_action = true;
            }

            // If after all checks, the action still does not have permission, return false.
            if !has_permission_for_action {
                return (false, applied_permissions);
            }
        }

        (true, applied_permissions)
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
        let signer_count = self.proposal_signers(proposal.id, &role).len();

        signer_count > 0 && signer_count >= self.get_signer_majority_for_role(&role)
    }

    fn get_signer_majority_for_role(&self, role: &ManagedBuffer) -> usize {
        self.roles_member_amount(&role).get() / 2 + 1
    }

    fn require_proposed_via_trusted_host(
        &self,
        proposer: &ManagedAddress,
        trusted_host_id: &ManagedBuffer,
        content_hash: &ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: &ManagedBuffer,
        permissions: &ManagedVec<ManagedBuffer>,
    ) {
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
