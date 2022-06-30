elrond_wasm::imports!();

use crate::config;
use crate::features::{self, FEATURE_NAME_LEADER};

#[elrond_wasm::module]
pub trait FactoryModule: config::ConfigModule + features::FeaturesModule {
    fn create_entity(&self, token_id: &TokenIdentifier, initial_tokens: &BigUint, features: &ManagedVec<ManagedBuffer>) -> ManagedAddress {
        require!(!self.trusted_host_address().is_empty(), "trusted host address needs to be configured");

        let trusted_host_address = self.trusted_host_address().get();
        let template_contract = self.get_template_address();

        let leader_arg = if features.contains(&ManagedBuffer::from(FEATURE_NAME_LEADER)) {
            OptionalValue::Some(self.blockchain().get_caller())
        } else {
            OptionalValue::None
        };

        let (address, _) = self
            .entity_contract_proxy(ManagedAddress::zero())
            .init(trusted_host_address, OptionalValue::Some(token_id.clone()), OptionalValue::Some(initial_tokens.clone()), leader_arg)
            .deploy_from_source::<()>(&template_contract, self.get_deploy_code_metadata());

        require!(!address.is_zero(), "address is zero");

        address
    }

    fn upgrade_entity(&self, address: ManagedAddress) {
        require!(!self.trusted_host_address().is_empty(), "trusted host address needs to be configured");

        let trusted_host_address = self.trusted_host_address().get();
        let template_contract = self.get_template_address();

        self.entity_contract_proxy(address)
            .init(trusted_host_address, OptionalValue::<TokenIdentifier>::None, OptionalValue::<BigUint>::None, OptionalValue::<ManagedAddress>::None)
            .upgrade_from_source(&template_contract, self.get_deploy_code_metadata());
    }

    fn get_deploy_code_metadata(&self) -> CodeMetadata {
        CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE | CodeMetadata::PAYABLE_BY_SC
    }

    #[proxy]
    fn entity_contract_proxy(&self, to: ManagedAddress) -> entity_esdt::Proxy<Self::Api>;
}
