multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{config, plug};

pub const ROLE_BUILTIN_LEADER: &[u8] = b"leader";
pub const ROLE_BUILTIN_DEVELOPER: &[u8] = b"developer";

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct PermissionDetails<M: ManagedTypeApi> {
    pub value: BigUint<M>,
    pub destination: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
    pub payments: ManagedVec<M, EsdtTokenPayment<M>>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct Policy<M: ManagedTypeApi> {
    pub method: PolicyMethod,
    pub quorum: BigUint<M>,
    pub voting_period_minutes: usize,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, Copy, PartialEq, Debug, ManagedVecItem)]
pub enum PolicyMethod {
    Weight,
    One,
    All,
    Quorum,
    Majority,
}

impl PolicyMethod {
    pub fn to_name(&self) -> &[u8] {
        match self {
            PolicyMethod::Weight => b"weight",
            PolicyMethod::One => b"one",
            PolicyMethod::All => b"all",
            PolicyMethod::Quorum => b"quorum",
            PolicyMethod::Majority => b"majority",
        }
    }
}

#[multiversx_sc::module]
pub trait PermissionModule: config::ConfigModule + plug::PlugModule {
    fn init_permission_module(&self, leader: ManagedAddress) {
        self.create_role(ManagedBuffer::from(ROLE_BUILTIN_LEADER));
        self.assign_role(leader, ManagedBuffer::from(ROLE_BUILTIN_LEADER));
    }

    /// Create a custom role.
    /// Can only be called by the contract itself.
    #[endpoint(createRole)]
    fn create_role_endpoint(&self, role_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_role(role_name);
    }

    /// Remove a custom role.
    /// Will also unassign all users belonging to that role.
    /// Can only be called by the contract itself.
    #[endpoint(removeRole)]
    fn remove_role_endpoint(&self, role_name: ManagedBuffer) {
        self.require_caller_self();
        self.remove_role(role_name);
    }

    /// Assign a custom role to the given user.
    /// Can only be called by the contract itself.
    #[endpoint(assignRole)]
    fn assign_role_endpoint(&self, role_name: ManagedBuffer, address: ManagedAddress) {
        self.require_caller_self();
        self.assign_role(address, role_name);
    }

    /// Unassign a custom role from the given user.
    /// Can only be called by the contract itself.
    #[endpoint(unassignRole)]
    fn unassign_role_endpoint(&self, role_name: ManagedBuffer, address: ManagedAddress) {
        self.require_caller_self();
        self.unassign_role(address, role_name);
    }

    /// Create a general permission.
    /// This permission can later be connected to custom roles through a policy.
    /// Can only be called by the contract itself.
    #[endpoint(createPermission)]
    fn create_permission_endpoint(
        &self,
        permission_name: ManagedBuffer,
        value: BigUint,
        destination: ManagedAddress,
        endpoint: ManagedBuffer,
        payments_multi: MultiValueManagedVec<EsdtTokenPaymentMultiValue>,
    ) {
        self.require_caller_self();

        let mut payments = ManagedVec::new();

        for payment in payments_multi.iter() {
            payments.push(payment.into_esdt_token_payment());
        }

        self.create_permission(permission_name, value, destination, endpoint, ManagedVec::new(), payments);
    }

    /// Create a policy that requires role members to vote based on the provided parameters in order to invoke the permission.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyWeighted)]
    fn create_policy_weighted_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer, quorum: BigUint, voting_period_minutes: usize) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::Weight, quorum, voting_period_minutes);
    }

    /// Create a policy that allows permissions to be invoked unilaterally.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyOne)]
    fn create_policy_one_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::One, BigUint::from(1u64), 0);
    }

    /// Create a policy that requires all role members to sign in order to invoke the permission.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyAll)]
    fn create_policy_all_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::All, BigUint::zero(), self.voting_period_in_minutes().get());
    }

    /// Create a policy that requires role members to reach a defined quorum in order to invoke the permission.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyQuorum)]
    fn create_policy_quorum_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer, quorum: usize) {
        self.require_caller_self();
        self.create_policy(
            role_name,
            permission_name,
            PolicyMethod::Quorum,
            BigUint::from(quorum),
            self.voting_period_in_minutes().get(),
        );
    }

    /// Create a policy that requires role members to reach a majority in order to invoke the permission.
    /// Can only be called by the contract itself.
    #[endpoint(createPolicyMajority)]
    fn create_policy_majority_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_policy(
            role_name,
            permission_name,
            PolicyMethod::Majority,
            BigUint::zero(),
            self.voting_period_in_minutes().get(),
        );
    }

    #[view(getUserRoles)]
    fn get_user_roles_view(&self, address: ManagedAddress) -> MultiValueEncoded<ManagedBuffer> {
        let user_id = self.users().get_user_id(&address);
        let mut roles = MultiValueEncoded::new();

        if user_id == 0 {
            return roles;
        }

        for role in self.user_roles(user_id).iter() {
            roles.push(role);
        }

        roles
    }

    #[view(getPermissions)]
    fn get_permissions_view(
        &self,
    ) -> MultiValueEncoded<
        MultiValue8<ManagedBuffer, BigUint, ManagedAddress, ManagedBuffer, usize, MultiValueEncoded<ManagedBuffer>, usize, MultiValueEncoded<EsdtTokenPaymentMultiValue>>,
    > {
        let mut permissions = MultiValueEncoded::new();

        for permission_name in self.permissions().iter() {
            let perm = self.permission_details(&permission_name).get();

            permissions.push(
                (
                    permission_name,
                    perm.value,
                    perm.destination,
                    perm.endpoint,
                    perm.arguments.len(),
                    MultiValueEncoded::from(perm.arguments),
                    perm.payments.len(),
                    MultiValueEncoded::from(perm.payments.into_multi_value()),
                )
                    .into(),
            );
        }

        permissions
    }

    #[view(getPolicies)]
    fn get_policies_view(&self, role_name: ManagedBuffer) -> MultiValueEncoded<MultiValue4<ManagedBuffer, ManagedBuffer, BigUint, usize>> {
        let mut policies = MultiValueEncoded::new();

        for (permission_name, policy) in self.policies(&role_name).iter() {
            policies.push(
                (
                    permission_name,
                    ManagedBuffer::from(policy.method.to_name()),
                    policy.quorum,
                    policy.voting_period_minutes,
                )
                    .into(),
            );
        }

        policies
    }

    fn create_role(&self, role_name: ManagedBuffer) {
        let created = self.roles().insert(role_name);

        require!(created, "role already exists")
    }

    fn remove_role(&self, role_name: ManagedBuffer) {
        require!(self.roles().contains(&role_name), "role does not exist");

        self.roles().swap_remove(&role_name);
        self.roles_member_amount(&role_name).set(0);

        for user_id in 1..=self.users().get_user_count() {
            self.user_roles(user_id).swap_remove(&role_name);
        }
    }

    fn assign_role(&self, address: ManagedAddress, role_name: ManagedBuffer) {
        if !self.roles().contains(&role_name) {
            self.create_role(role_name.clone());
        }

        let user_id = self.users().get_or_create_user(&address);

        if self.user_roles(user_id).insert(role_name.clone()) {
            self.roles_member_amount(&role_name).update(|current| *current += 1);
        }
    }

    fn unassign_role(&self, address: ManagedAddress, role_name: ManagedBuffer) {
        require!(self.roles().contains(&role_name), "role does not exist");

        let leader_role_name = ManagedBuffer::from(ROLE_BUILTIN_LEADER);
        let is_last_leader = role_name == leader_role_name && self.roles_member_amount(&role_name).get() == 1;

        if is_last_leader && !self.is_plugged() && self.gov_token_id().is_empty() {
            sc_panic!("can not remove last leader: gov token or plug required");
        }

        if is_last_leader {
            self.remove_role(leader_role_name);
        }

        let user_id = self.users().get_or_create_user(&address);

        if self.user_roles(user_id).swap_remove(&role_name) {
            self.roles_member_amount(&role_name).update(|current| *current -= 1);
        }
    }

    fn create_permission(
        &self,
        permission_name: ManagedBuffer,
        value: BigUint,
        destination: ManagedAddress,
        endpoint: ManagedBuffer,
        arguments: ManagedVec<ManagedBuffer>,
        payments: ManagedVec<EsdtTokenPayment>,
    ) {
        self.permissions().insert(permission_name.clone());

        self.permission_details(&permission_name).set(PermissionDetails {
            value,
            destination,
            endpoint,
            arguments,
            payments,
        });
    }

    fn get_user_policies_for_permissions(&self, address: &ManagedAddress, permissions: &ManagedVec<ManagedBuffer>) -> (bool, ManagedVec<Policy<Self::Api>>) {
        let proposer_id = self.users().get_user_id(address);

        if proposer_id == 0 {
            return (false, ManagedVec::new());
        }

        let proposer_roles = self.user_roles(proposer_id);
        let mut policies = ManagedVec::new();
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

    fn create_policy(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer, method: PolicyMethod, quorum: BigUint, voting_period_minutes: usize) {
        require!(self.roles().contains(&role_name), "role does not exist");
        require!(self.permissions().contains(&permission_name), "permission does not exist");
        require!(!self.policies(&role_name).contains_key(&permission_name), "policy already exists");

        self.policies(&role_name).insert(
            permission_name,
            Policy {
                method,
                quorum,
                voting_period_minutes,
            },
        );
    }

    fn has_role(&self, address: &ManagedAddress, role_name: &ManagedBuffer) -> bool {
        let user_id = self.users().get_user_id(&address);

        if user_id == 0 {
            return false;
        }

        self.user_roles(user_id).contains(&role_name)
    }

    fn has_token_weighted_policy(&self, policies: &ManagedVec<Policy<Self::Api>>) -> bool {
        policies.iter().find(|p| p.method == PolicyMethod::Weight).is_some()
    }

    fn is_leaderless(&self) -> bool {
        let leader_role = ManagedBuffer::from(ROLE_BUILTIN_LEADER);
        let is_leaderless = self.roles_member_amount(&leader_role).get() == 0;

        is_leaderless
    }

    fn has_leader_role(&self, address: &ManagedAddress) -> bool {
        let leader_role = ManagedBuffer::from(ROLE_BUILTIN_LEADER);

        self.has_role(&address, &leader_role)
    }

    fn require_caller_has_leader_role(&self) {
        let caller = self.blockchain().get_caller();
        require!(self.has_leader_role(&caller), "caller must be leader");
    }

    #[view(getRoles)]
    #[storage_mapper("roles")]
    fn roles(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[view(getRoleMemberAmount)]
    #[storage_mapper("roles_member_amount")]
    fn roles_member_amount(&self, role_name: &ManagedBuffer) -> SingleValueMapper<usize>;

    #[storage_mapper("user_roles")]
    fn user_roles(&self, user_id: usize) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[storage_mapper("permissions")]
    fn permissions(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

    #[storage_mapper("permission_details")]
    fn permission_details(&self, permission_name: &ManagedBuffer) -> SingleValueMapper<PermissionDetails<Self::Api>>;

    #[storage_mapper("policies")]
    fn policies(&self, role_name: &ManagedBuffer) -> MapMapper<ManagedBuffer<Self::Api>, Policy<Self::Api>>;
}
