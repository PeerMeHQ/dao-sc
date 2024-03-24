#![no_std]

multiversx_sc::imports!();

pub mod config;
pub mod credits;
pub mod dex;
pub mod events;
pub mod factory;
pub mod organization;

#[multiversx_sc::contract]
pub trait Manager:
    config::ConfigModule + factory::FactoryModule + credits::CreditsModule + events::EventsModule + dex::DexModule + organization::OrganizationModule
{
    #[init]
    fn init(&self, entity_template_address: ManagedAddress, trusted_host_address: ManagedAddress, native_token: TokenIdentifier, cost_entity_creation: BigUint) {
        self.entity_templ_address().set(&entity_template_address);
        self.trusted_host_address().set(&trusted_host_address);
        self.native_token().set(&native_token);
        self.cost_creation_amount().set(&cost_entity_creation);
    }

    #[endpoint]
    fn upgrade(&self, native_token: TokenIdentifier) {
        self.native_token().set(&native_token);
    }

    #[endpoint(addAdmin)]
    fn add_admin_endpoint(&self, address: ManagedAddress) {
        self.require_caller_is_admin();
        self.admins().insert(address);
    }

    #[endpoint(removeAdmin)]
    fn remove_admin_endpoint(&self, address: ManagedAddress) {
        self.require_caller_is_admin();
        self.admins().swap_remove(&address);
    }

    #[endpoint(forwardToken)]
    fn forward_token_endpoint(&self, token: TokenIdentifier, amount: BigUint, address: ManagedAddress) {
        self.require_caller_is_admin();
        self.send().direct_esdt(&address, &token, 0, &amount);
    }

    #[payable("*")]
    #[endpoint(executeTicket)]
    fn execute_ticket_endpoint(&self, ticket_type: ManagedBuffer, ticket_id: ManagedBuffer) {
        require!(!ticket_type.is_empty(), "ticket type is required");
        require!(!ticket_id.is_empty(), "ticket id is required");
    }

    #[payable("EGLD")]
    #[endpoint(createEntity)]
    fn create_entity_endpoint(&self, features: MultiValueManagedVec<ManagedBuffer>) -> ManagedAddress {
        let payment_amount = self.call_value().egld_value().clone_value();
        require!(payment_amount >= self.cost_creation_amount().get(), "invalid cost amount");

        let caller = self.blockchain().get_caller();
        let entity_address = self.create_entity();

        self.wrap_egld(payment_amount.clone());
        let swapped_payment = self.swap_wegld_to_stable_tokens(payment_amount.clone());

        self.entities().insert(entity_address.clone());
        self.set_features(&entity_address, features.into_vec());
        self.recalculate_daily_cost(&entity_address);
        self.boost_by_user(caller, entity_address.clone(), swapped_payment.amount.clone());
        self.forward_payment_to_org(EgldOrEsdtTokenPayment::from(swapped_payment));

        entity_address
    }

    #[endpoint(upgradeEntity)]
    fn upgrade_entity_endpoint(&self, entity_address: ManagedAddress) {
        self.require_entity_exists(&entity_address);
        self.recalculate_daily_cost(&entity_address);
        self.upgrade_entity(entity_address);
    }
}
