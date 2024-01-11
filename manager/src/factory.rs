multiversx_sc::imports!();

use crate::config;
use crate::events;

#[multiversx_sc::module]
pub trait FactoryModule: config::ConfigModule + events::EventsModule {
    fn create_entity(&self) -> ManagedAddress {
        require!(!self.trusted_host_address().is_empty(), "trusted host address needs to be configured");

        let trusted_host_address = self.trusted_host_address().get();
        let template_contract = self.get_template_address();
        let leader = self.blockchain().get_caller();

        let (address, _) = self
            .entity_contract_proxy(ManagedAddress::zero())
            .init(trusted_host_address, leader)
            .deploy_from_source::<()>(&template_contract, self.get_deploy_code_metadata());

        require!(!address.is_zero(), "address is zero");

        self.entity_created_event(address.clone());

        address
    }

    fn upgrade_entity(&self, address: ManagedAddress) {
        require!(!self.trusted_host_address().is_empty(), "trusted host address needs to be configured");

        let trusted_host_address = self.trusted_host_address().get();
        let template = self.get_template_address();
        let gas = self.blockchain().get_gas_left();
        let metadata = self.get_deploy_code_metadata();

        let mut args = ManagedArgBuffer::new();
        args.push_arg(trusted_host_address);

        self.send_raw()
            .upgrade_from_source_contract(&address, gas, &BigUint::zero(), &template, metadata, &args);

        self.entity_upgraded_event(address);
    }

    fn get_deploy_code_metadata(&self) -> CodeMetadata {
        CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE | CodeMetadata::PAYABLE_BY_SC
    }

    #[proxy]
    fn entity_contract_proxy(&self, to: ManagedAddress) -> entity::Proxy<Self::Api>;
}
