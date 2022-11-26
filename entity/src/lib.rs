#![no_std]

elrond_wasm::imports!();

pub mod config;
pub mod governance;
pub mod permission;

#[elrond_wasm::contract]
pub trait Entity:
    config::ConfigModule
    + permission::PermissionModule
    + governance::GovernanceModule
    + governance::events::GovEventsModule
    + governance::proposal::ProposalModule
    + governance::vote::VoteModule
    + governance::token::TokenModule
{
    #[init]
    fn init(&self, trusted_host_address: ManagedAddress, opt_leader: OptionalValue<ManagedAddress>) {
        self.trusted_host_address().set(&trusted_host_address);
        self.init_governance_module();

        if let OptionalValue::Some(leader) = opt_leader {
            self.init_permission_module(leader);
        }
    }

    #[endpoint(eject)]
    fn eject_endpoint(&self, trusted_host_address: ManagedAddress) {
        self.require_caller_self();

        self.trusted_host_address().set(&trusted_host_address);
    }

    #[payable("EGLD")]
    #[endpoint(registerDns)]
    fn register_dns(&self, dns_address: ManagedAddress, name: ManagedBuffer) {
        self.require_caller_self();
        let payment = self.call_value().egld_value();

        self.dns_proxy(dns_address).register(&name).with_egld_transfer(payment).async_call().call_and_exit()
    }

    #[view(getVersion)]
    fn version_view(&self) -> &'static [u8] {
        env!("CARGO_PKG_VERSION").as_bytes()
    }

    #[proxy]
    fn dns_proxy(&self, to: ManagedAddress) -> dns_proxy::Proxy<Self::Api>;
}

mod dns_proxy {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait Dns {
        #[payable("EGLD")]
        #[endpoint]
        fn register(&self, name: &ManagedBuffer);
    }
}
