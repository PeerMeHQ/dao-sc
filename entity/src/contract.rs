multiversx_sc::imports!();

use crate::config;
use crate::governance;
use crate::governance::events;
use crate::governance::proposal::ProposalStatus;
use crate::permission;
use crate::permission::ROLE_BUILTIN_DEVELOPER;
use crate::plug;

#[multiversx_sc::module]
pub trait ContractModule:
    config::ConfigModule + governance::proposal::ProposalModule + permission::PermissionModule + events::GovEventsModule + plug::PlugModule
{
    #[endpoint(lockContractStage)]
    fn lock_contract_stage_endpoint(&self, address: ManagedAddress) {
        self.require_caller_self();

        let locked = self.stage_lock(&address).get();
        require!(!locked, "contract stage is locked already");

        let has_code = !self.stage(&address).is_empty();
        require!(has_code, "contract stage is empty");

        self.stage_lock(&address).set(true);
    }

    #[endpoint(unlockContractStage)]
    fn unlock_contract_stage_endpoint(&self, address: ManagedAddress) {
        self.require_caller_self();
        require!(self.is_stage_locked(&address), "contract stage is unlocked already");

        self.clear_stage(&address);
    }

    #[endpoint(stageContract)]
    fn stage_contract_endpoint(&self, address: ManagedAddress, code: ManagedBuffer) {
        self.require_caller_is_developer();

        self.stage_contract(&address, &code);
    }

    #[endpoint(stageContractAndPropose)]
    fn stage_contract_and_propose_endpoint(
        &self,
        address: ManagedAddress,
        code: ManagedBuffer,
        trusted_host_id: ManagedBuffer,
        content_hash: ManagedBuffer,
        content_sig: ManagedBuffer,
        actions_hash: ManagedBuffer,
        permissions: MultiValueManagedVec<ManagedBuffer>,
    ) -> u64 {
        self.require_caller_is_developer();

        let active_proposal_id = self.stage_current_proposal(&address).get();

        if active_proposal_id > 0 {
            self.cancel_stage_current_proposal(&address, active_proposal_id);
        }

        self.stage_contract(&address, &code);

        let proposer = self.blockchain().get_caller();

        let proposal = self.create_proposal(
            proposer,
            trusted_host_id,
            content_hash,
            content_sig,
            actions_hash,
            0,
            BigUint::zero(),
            permissions.into_vec(),
        );

        self.stage_current_proposal(&address).set(proposal.id);

        proposal.id
    }

    #[endpoint(activateContract)]
    fn activate_contract_endpoint(&self, unique_id: ManagedBuffer, address: ManagedAddress, code_metadata: CodeMetadata, args: MultiValueEncoded<ManagedBuffer>) {
        self.require_caller_self();
        require!(!self.stage(&address).is_empty(), "contract not staged");
        require!(!self.unique_ids().contains(&unique_id), "unique id already exists");

        let args_buffer = args.to_arg_buffer();
        let code = self.stage(&address).take();
        let gas = self.blockchain().get_gas_left();
        let value = BigUint::zero();

        self.unique_ids().insert(unique_id);
        self.stage_lock(&address).clear();

        if address.is_zero() {
            self.send_raw().deploy_contract(gas, &value, &code, code_metadata, &args_buffer);
        } else {
            self.send_raw().upgrade_contract(&address, gas, &value, &code, code_metadata, &args_buffer);
        }
    }

    #[view(getContractStage)]
    fn get_contract_stage_view(&self, address: ManagedAddress) -> MultiValue2<bool, ManagedBuffer> {
        let is_locked = self.is_stage_locked(&address);
        let code = self.stage(&address).get();

        (is_locked, code).into()
    }

    fn stage_contract(&self, address: &ManagedAddress, code: &ManagedBuffer) {
        require!(!self.is_stage_locked(&address), "contract stage is locked");
        require!(self.blockchain().is_smart_contract(&address), "address must be contract");
        require!(&self.blockchain().get_sc_address() != address, "address must not be self");
        require!(!code.is_empty(), "code must not be empty");

        self.stage(&address).set(code);
        self.stage_lock(&address).set(true);
    }

    fn require_caller_is_developer(&self) {
        let caller = self.blockchain().get_caller();
        let dev_role = ManagedBuffer::from(ROLE_BUILTIN_DEVELOPER);
        let has_dev_role = self.has_role(&caller, &dev_role);

        require!(has_dev_role, "caller must be developer");
    }

    fn cancel_stage_current_proposal(&self, address: &ManagedAddress, active_proposal_id: u64) {
        let active_proposal = self.proposals(active_proposal_id).get();

        if self.get_proposal_status(&active_proposal) != ProposalStatus::Active {
            return;
        }

        self.cancel_proposal(active_proposal);
        self.clear_stage(&address);
    }

    fn clear_stage(&self, address: &ManagedAddress) {
        self.stage_lock(&address).clear();
        self.stage(&address).clear();
    }

    fn is_stage_locked(&self, address: &ManagedAddress) -> bool {
        !self.stage_lock(&address).is_empty()
    }

    #[storage_mapper("contract:stage")]
    fn stage(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("contract:stage_lock")]
    fn stage_lock(&self, address: &ManagedAddress) -> SingleValueMapper<bool>;

    #[storage_mapper("contract:stage_current_proposal")]
    fn stage_current_proposal(&self, address: &ManagedAddress) -> SingleValueMapper<u64>;

    #[storage_mapper("contract:unique_ids")]
    fn unique_ids(&self) -> UnorderedSetMapper<ManagedBuffer<Self::Api>>;
}
