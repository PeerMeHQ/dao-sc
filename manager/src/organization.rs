use crate::config;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait OrganizationModule: config::ConfigModule {
    #[only_owner]
    #[endpoint(initOrgModule)]
    fn init_organization_module(&self, org_contract: ManagedAddress) {
        self.org_contract_address().set(org_contract);
    }

    fn forward_payment_to_org(&self, payment: EgldOrEsdtTokenPayment) {
        if self.org_contract_address().is_empty() {
            return;
        }

        self.org_contract_proxy(self.org_contract_address().get())
            .deposit_endpoint()
            .with_egld_or_single_esdt_transfer(EgldOrEsdtTokenPayment::from(payment))
            .execute_on_dest_context::<()>();
    }

    fn forward_distribution_to_org(&self, payment: EsdtTokenPayment) {
        if self.org_contract_address().is_empty() {
            return;
        }

        self.org_contract_proxy(self.org_contract_address().get())
            .distribute_endpoint()
            .with_esdt_transfer(payment)
            .execute_on_dest_context::<()>();
    }

    #[storage_mapper("org:organization_contract_address")]
    fn org_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn org_contract_proxy(&self, to: ManagedAddress) -> organization_proxy::Proxy<Self::Api>;
}

mod organization_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait OrganizationContractProxy {
        #[payable("*")]
        #[endpoint(deposit)]
        fn deposit_endpoint(&self);

        #[payable("*")]
        #[endpoint(distribute)]
        fn distribute_endpoint(&self);
    }
}
