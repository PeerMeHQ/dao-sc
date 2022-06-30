elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::config;

pub const ROLE_BUILTIN_LEADER: &[u8] = b"leader";

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct PermissionDetails<M: ManagedTypeApi> {
    pub destination: ManagedAddress<M>,
    pub endpoint: ManagedBuffer<M>,
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
}

impl PolicyMethod {
    pub fn to_name(&self) -> &[u8] {
        match self {
            PolicyMethod::Weight => b"weight",
            PolicyMethod::One => b"one",
            PolicyMethod::All => b"all",
            PolicyMethod::Quorum => b"quorum",
        }
    }
}

#[elrond_wasm::module]
pub trait PermissionModule: config::ConfigModule {
    fn init_permission_module(&self, opt_leader: OptionalValue<ManagedAddress>) {
        let has_initialized = !self.roles().is_empty() || !self.permissions().is_empty();

        if has_initialized {
            return;
        }

        self.create_role(ManagedBuffer::from(ROLE_BUILTIN_LEADER));

        if let OptionalValue::Some(leader) = opt_leader {
            self.assign_role(leader, ManagedBuffer::from(ROLE_BUILTIN_LEADER));
        }
    }

    #[endpoint(createRole)]
    fn create_role_endpoint(&self, role_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_role(role_name);
    }

    #[endpoint(assignRole)]
    fn assign_role_endpoint(&self, address: ManagedAddress, role_name: ManagedBuffer) {
        self.require_caller_self();
        self.assign_role(address, role_name);
    }

    #[endpoint(createPermission)]
    fn create_permission_endpoint(&self, permission_name: ManagedBuffer, destination: ManagedAddress, endpoint: ManagedBuffer) {
        self.require_caller_self();
        self.create_permission(permission_name, destination, endpoint);
    }

    #[endpoint(createPolicyWeighted)]
    fn create_policy_weighted_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer, quorum: BigUint, voting_period_minutes: usize) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::Weight, quorum, voting_period_minutes);
    }

    #[endpoint(createPolicyForOne)]
    fn create_policy_one_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::One, BigUint::from(1u64), 0);
    }

    #[endpoint(createPolicyForAll)]
    fn create_policy_all_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::All, BigUint::zero(), self.voting_period_in_minutes().get());
    }

    #[endpoint(createPolicyQuorum)]
    fn create_policy_quorum_endpoint(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer, quorum: usize) {
        self.require_caller_self();
        self.create_policy(role_name, permission_name, PolicyMethod::Quorum, BigUint::from(quorum), self.voting_period_in_minutes().get());
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
    fn get_permissions_view(&self) -> MultiValueEncoded<MultiValue3<ManagedBuffer, ManagedAddress, ManagedBuffer>> {
        let mut permissions = MultiValueEncoded::new();

        for permission_name in self.permissions().iter() {
            let permission_details = self.permission_details(&permission_name).get();
            permissions.push((permission_name, permission_details.destination, permission_details.endpoint).into());
        }

        permissions
    }

    #[view(getPolicies)]
    fn get_policies_view(&self, role_name: ManagedBuffer) -> MultiValueEncoded<MultiValue4<ManagedBuffer, ManagedBuffer, BigUint, usize>> {
        let mut policies = MultiValueEncoded::new();

        for (permission_name, policy) in self.policies(&role_name).iter() {
            policies.push((permission_name, ManagedBuffer::from(policy.method.to_name()), policy.quorum, policy.voting_period_minutes).into());
        }

        policies
    }

    fn create_role(&self, role_name: ManagedBuffer) {
        let created = self.roles().insert(role_name);

        require!(created, "role already exists");
    }

    fn assign_role(&self, address: ManagedAddress, role_name: ManagedBuffer) {
        require!(self.roles().contains(&role_name), "role does not exist");

        self.roles_member_amount(&role_name).update(|current| *current += 1);

        let user_id = self.users().get_or_create_user(&address);
        let added = self.user_roles(user_id).insert(role_name);

        require!(added, "user already has role");
    }

    fn create_permission(&self, permission_name: ManagedBuffer, destination: ManagedAddress, endpoint: ManagedBuffer) {
        let created = self.permissions().insert(permission_name.clone());

        require!(created, "permission already exists");

        self.permission_details(&permission_name).set(PermissionDetails {
            destination,
            endpoint,
        });
    }

    fn create_policy(&self, role_name: ManagedBuffer, permission_name: ManagedBuffer, method: PolicyMethod, quorum: BigUint, voting_period_minutes: usize) {
        require!(self.roles().contains(&role_name), "role does not exist");
        require!(self.permissions().contains(&permission_name), "permission does not exist");

        require!(!self.policies(&role_name).contains_key(&permission_name), "policy already exists");

        self.policies(&role_name).insert(permission_name, Policy {
            method,
            quorum,
            voting_period_minutes,
        });
    }

    fn has_role(&self, address: ManagedAddress, role_name: ManagedBuffer) -> bool {
        let user_id = self.users().get_user_id(&address);

        if user_id == 0 {
            return false;
        }

        self.user_roles(user_id).contains(&role_name)
    }

    fn has_token_weighted_policy(&self, policies: &ManagedVec<Policy<Self::Api>>) -> bool {
        policies.iter()
            .find(|p| p.method == PolicyMethod::Weight)
            .is_some()
    }

    fn does_leader_role_exist(&self) -> bool {
        self.roles().contains(&ManagedBuffer::from(ROLE_BUILTIN_LEADER))
    }

    #[view(getRoles)]
    #[storage_mapper("roles")]
    fn roles(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;

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
