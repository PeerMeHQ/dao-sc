multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("entityCreated")]
    fn entity_created_event(&self, #[indexed] entity: ManagedAddress);

    #[event("entityUpgraded")]
    fn entity_upgraded_event(&self, #[indexed] entity: ManagedAddress);

    #[event("boost")]
    fn boost_event(
        &self,
        #[indexed] booster: ManagedAddress,
        #[indexed] entity: ManagedAddress,
        #[indexed] amount: BigUint,
        #[indexed] virtual_amount: BigUint,
        #[indexed] bonus_factor: u8,
    );

    #[event("voucher_redeemed")]
    fn voucher_redeemed_event(&self, #[indexed] caller: ManagedAddress, #[indexed] nonce: u64);
}
