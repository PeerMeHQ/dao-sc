multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ConfigModule {
    fn require_entity_exists(&self, entity_address: &ManagedAddress) {
        require!(self.entities().contains(&entity_address), "entity does not exist");
    }

    fn require_caller_is_admin(&self) {
        let caller = self.blockchain().get_caller();
        let is_admin = self.admins().contains(&caller);
        let is_owner = self.blockchain().get_owner_address() == caller;
        let is_trusted_host = self.trusted_host_address().get() == caller;
        require!(is_admin || is_owner || is_trusted_host, "caller must be admin");
    }

    fn get_template_address(&self) -> ManagedAddress {
        require!(!self.entity_templ_address().is_empty(), "no template set");

        self.entity_templ_address().get()
    }

    #[endpoint(setEntityTemplateAddress)]
    fn set_entity_template_address_endpoint(&self, address: ManagedAddress) {
        self.require_caller_is_admin();
        require!(self.blockchain().is_smart_contract(&address), "must be contract");

        self.entity_templ_address().set(&address);
    }

    #[endpoint(setEntityCreationCost)]
    fn set_entity_creation_cost_endpoint(&self, amount: BigUint) {
        self.require_caller_is_admin();
        require!(amount > 0, "can not be zero");
        self.cost_creation_amount().set(amount);
    }

    #[view(getAdmins)]
    #[storage_mapper("admins")]
    fn admins(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getEntities)]
    #[storage_mapper("entities")]
    fn entities(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getEntityTemplateAddress)]
    #[storage_mapper("entity_templ_address")]
    fn entity_templ_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getTrustedHostAddress)]
    #[storage_mapper("trusted_host_addr")]
    fn trusted_host_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getNativeToken)]
    #[storage_mapper("native_token")]
    fn native_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getEntityCreationCost)]
    #[storage_mapper("cost_creation_amount")]
    fn cost_creation_amount(&self) -> SingleValueMapper<BigUint>;
}
