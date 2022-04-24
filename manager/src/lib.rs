#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();

mod config;
mod esdt;
mod factory;

#[elrond_wasm::contract]
pub trait Manager: config::ConfigModule + factory::FactoryModule + esdt::EsdtModule {
    #[init]
    fn init(&self, entity_template_address: ManagedAddress, cost_token: TokenIdentifier, cost_entity_creation: BigUint) {
        self.entity_templ_address().set_if_empty(&entity_template_address);
        self.cost_token_id().set_if_empty(&cost_token);
        self.cost_creation_amount().set_if_empty(&cost_entity_creation);
    }

    #[payable("EGLD")]
    #[endpoint(deposit)]
    fn deposit_endpoint(&self) {}

    #[payable("EGLD")]
    #[endpoint(createEntityToken)]
    fn create_entity_token_endpoint(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer, amount: BigUint) {
        let issue_cost = self.call_value().egld_value();
        let initial_caller = self.blockchain().get_caller();

        require!(amount > 0, "amount must be greater than zero");

        self.issue_token(&token_name, &token_ticker, &amount, &issue_cost)
            .with_callback(self.callbacks().token_issue_callback(&initial_caller))
            .call_and_exit()
    }

    #[payable("*")]
    #[endpoint(registerEntityToken)]
    fn register_entity_token_endpoint(&self, supply: BigUint) {
        let caller = self.blockchain().get_caller();
        let proof = self.call_value().payment();

        self.setup_token_id(&caller).set(&proof.token_identifier);
        self.setup_token_amount(&caller).set(&supply);

        self.send()
            .direct(&caller, &proof.token_identifier, proof.token_nonce, &proof.amount, &[]);
    }

    #[payable("*")]
    #[endpoint(createEntity)]
    fn create_entity_endpoint(&self, token_id: TokenIdentifier, #[var_args] features: MultiValueEncoded<MultiValue2<ManagedBuffer, ManagedBuffer>>) {
        let (cost_token_id, _, cost_amount) = self.call_value().payment_as_tuple();

        self.require_caller_is_setup_owner(&token_id);
        require!(cost_token_id == self.cost_token_id().get(), "invalid cost token");
        require!(cost_amount >= self.cost_creation_amount().get(), "invalid cost amount");

        let caller = self.blockchain().get_caller();
        let initial_tokens = self.setup_token_amount(&caller).get();

        require!(initial_tokens > 0, "setup token is not available");

        let entity_address = self.create_entity(&token_id, &initial_tokens);

        self.entities_map().insert(token_id.clone(), entity_address.clone());

        self.enable_entity_features(&entity_address, features);

        self.send().esdt_local_burn(&cost_token_id, 0, &cost_amount);

        self.set_entity_edst_roles(&token_id, &entity_address).call_and_exit()
    }

    #[endpoint(finalizeEntity)]
    fn finalize_entity_endpoint(&self, token_id: TokenIdentifier) {
        self.require_caller_is_setup_owner(&token_id);

        let caller = self.blockchain().get_caller();
        let entity_address = self.get_entity_address(&token_id);

        self.setup_token_id(&caller).clear();
        self.setup_token_amount(&caller).clear();

        self.transfer_entity_esdt_ownership(&token_id, &entity_address).call_and_exit()
    }

    #[payable("*")]
    #[callback]
    fn token_issue_callback(&self, initial_caller: &ManagedAddress, #[call_result] result: ManagedAsyncCallResult<()>) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                let payment = self.call_value().payment();
                self.setup_token_id(&initial_caller).set(&payment.token_identifier);
                self.setup_token_amount(&initial_caller).set(&payment.amount);
                self.send().direct(&initial_caller, &payment.token_identifier, 0, &payment.amount, &[]);
            }
            ManagedAsyncCallResult::Err(_) => self.send_back_egld(&initial_caller),
        }
    }

    #[endpoint(upgradeEntity)]
    fn upgrade_entity_endpoint(&self, token_id: TokenIdentifier) {
        self.upgrade_entity(self.get_entity_address(&token_id));
    }

    #[view(getEntityAddress)]
    fn get_entity_address_view(&self, token_id: TokenIdentifier) -> ManagedAddress {
        self.entities_map().get(&token_id).unwrap_or_default()
    }

    fn get_entity_address(&self, token_id: &TokenIdentifier) -> ManagedAddress {
        require!(self.entities_map().contains_key(&token_id), "entity does not exist");

        self.entities_map().get(&token_id).unwrap()
    }

    fn send_back_egld(&self, initial_caller: &ManagedAddress) {
        let egld_returned = self.call_value().egld_value();
        if egld_returned > 0u32 {
            self.send().direct_egld(&initial_caller, &egld_returned, &[]);
        }
    }

    fn require_caller_is_setup_owner(&self, token_id: &TokenIdentifier) {
        let caller = self.blockchain().get_caller();
        let temp_owner_token_id = self.setup_token_id(&caller).get();
        require!(&temp_owner_token_id == token_id, "token not in setup");
    }

    #[storage_mapper("entities")]
    fn entities_map(&self) -> MapMapper<TokenIdentifier, ManagedAddress>;

    #[view(getSetupToken)]
    #[storage_mapper("setup:token_id")]
    fn setup_token_id(&self, owner: &ManagedAddress) -> SingleValueMapper<TokenIdentifier>;

    #[view(getSetupTokenAmount)]
    #[storage_mapper("setup:token_amount")]
    fn setup_token_amount(&self, owner: &ManagedAddress) -> SingleValueMapper<BigUint>;
}
