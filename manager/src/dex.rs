use crate::config;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait DexModule: config::ConfigModule {
    #[only_owner]
    #[endpoint(initDexModule)]
    fn init_dex_module(
        &self,
        wegld_token: TokenIdentifier,
        native_token_pair_contract: ManagedAddress,
        wrap_egld_contract: ManagedAddress,
        stable_token: TokenIdentifier,
        stable_token_pair_contract: ManagedAddress,
    ) {
        self.wegld_token().set(wegld_token);
        self.native_token_pair_contract().set(native_token_pair_contract);
        self.wrap_egld_contract().set(wrap_egld_contract);
        self.stable_token().set(stable_token);
        self.stable_token_pair_contract().set(stable_token_pair_contract);
    }

    fn wrap_egld(&self, amount: BigUint) -> EsdtTokenPayment {
        let wegld_token_id = self.wegld_token().get();
        let wrap_egld_contract = self.wrap_egld_contract().get();

        self.wrap_egld_proxy(wrap_egld_contract)
            .wrap_egld()
            .with_egld_transfer(amount.clone())
            .execute_on_dest_context::<()>();

        EsdtTokenPayment::new(wegld_token_id, 0, amount)
    }

    fn swap_tokens_to_wegld(&self, payment_token: TokenIdentifier, payment_amount: BigUint, swap_contract: ManagedAddress) -> EsdtTokenPayment {
        let wegld_token_id = self.wegld_token().get();

        let swapped_wegld: dex_pair_proxy::SwapTokensFixedInputResultType<Self::Api> = self
            .dex_pair_contract_proxy(swap_contract)
            .swap_tokens_fixed_input(wegld_token_id, BigUint::from(1u32))
            .with_esdt_transfer(EsdtTokenPayment::new(payment_token, 0, payment_amount))
            .execute_on_dest_context();

        swapped_wegld
    }

    fn swap_wegld_to_native_tokens(&self, amount: BigUint) -> EsdtTokenPayment {
        let cost_token_id = self.native_token().get();
        let wegld_token_id = self.wegld_token().get();
        let native_token_pair_contract = self.native_token_pair_contract().get();

        let swap_output: dex_pair_proxy::SwapTokensFixedInputResultType<Self::Api> = self
            .dex_pair_contract_proxy(native_token_pair_contract)
            .swap_tokens_fixed_input(cost_token_id.clone(), BigUint::from(1u32))
            .with_esdt_transfer(EsdtTokenPayment::new(wegld_token_id, 0, amount))
            .execute_on_dest_context();

        require!(swap_output.token_identifier == cost_token_id, "swapped invalid cost token");

        swap_output
    }

    fn swap_wegld_to_stable_tokens(&self, amount: BigUint) -> EsdtTokenPayment {
        let wegld_token = self.wegld_token().get();
        let stable_token = self.stable_token().get();
        let stable_token_pair_contract = self.stable_token_pair_contract().get();

        let swap_output: dex_pair_proxy::SwapTokensFixedInputResultType<Self::Api> = self
            .dex_pair_contract_proxy(stable_token_pair_contract)
            .swap_tokens_fixed_input(stable_token.clone(), BigUint::from(1u32))
            .with_esdt_transfer(EsdtTokenPayment::new(wegld_token, 0, amount))
            .execute_on_dest_context();

        require!(swap_output.token_identifier == stable_token, "swapped invalid cost token");

        swap_output
    }

    #[storage_mapper("dex:wegld_token_id")]
    fn wegld_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("dex:cost_token_wegld_swap_contract")]
    fn native_token_pair_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("dex:wrap_egld_contract")]
    fn wrap_egld_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getStableToken)]
    #[storage_mapper("dex:stable_token")]
    fn stable_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getStablePairContract)]
    #[storage_mapper("dex:stable_token_pair_contract")]
    fn stable_token_pair_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn dex_pair_contract_proxy(&self, to: ManagedAddress) -> dex_pair_proxy::Proxy<Self::Api>;

    #[proxy]
    fn wrap_egld_proxy(&self, to: ManagedAddress) -> dex_wrap_proxy::Proxy<Self::Api>;
}

mod dex_pair_proxy {
    multiversx_sc::imports!();
    multiversx_sc::derive_imports!();

    pub type SwapTokensFixedInputResultType<M> = EsdtTokenPayment<M>;

    #[multiversx_sc::proxy]
    pub trait DexRouterContractProxy {
        #[payable("*")]
        #[endpoint(swapTokensFixedInput)]
        fn swap_tokens_fixed_input(&self, token_out: TokenIdentifier, amount_out_min: BigUint) -> SwapTokensFixedInputResultType<Self::Api>;
    }
}

mod dex_wrap_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait DexWrapContractProxy {
        #[payable("EGLD")]
        #[endpoint(wrapEgld)]
        fn wrap_egld(&self);
    }
}
