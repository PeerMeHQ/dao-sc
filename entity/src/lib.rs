#![no_std]

multiversx_sc::imports!();

pub mod config;
pub mod contract;
pub mod governance;
pub mod permission;
pub mod plug;

#[multiversx_sc::contract]
pub trait Entity:
    config::ConfigModule
    + contract::ContractModule
    + permission::PermissionModule
    + plug::PlugModule
    + governance::GovernanceModule
    + governance::events::GovEventsModule
    + governance::proposal::ProposalModule
    + governance::token::TokenModule
{
    #[init]
    fn init(&self, trusted_host_address: ManagedAddress, leader: ManagedAddress) {
        self.trusted_host_address().set(&trusted_host_address);
        self.init_governance_module();
        self.init_permission_module(leader);
    }

    #[endpoint]
    fn upgrade(&self, trusted_host_address: ManagedAddress) {
        self.trusted_host_address().set(&trusted_host_address);
    }

    #[endpoint(changeVoteTokenLock)]
    fn change_vote_token_lock_endpoint(&self, token: TokenIdentifier, lock: bool) {
        self.require_caller_trusted_host();
        self.lock_vote_tokens(&token).set(lock);
    }

    #[payable("EGLD")]
    #[endpoint(registerDns)]
    fn register_dns(&self, dns_address: ManagedAddress, name: ManagedBuffer) {
        self.require_caller_self();
        let payment = self.call_value().egld_value().clone_value();

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
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait Dns {
        #[payable("EGLD")]
        #[endpoint]
        fn register(&self, name: &ManagedBuffer);
    }
}
