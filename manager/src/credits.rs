use crate::config;
use crate::dex;
use crate::events;
use crate::organization;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const REWARD_ADJUSTMENT_DECIMAL_EXPONENT: u32 = 12; // difference between stable token decimals (6) and reward token decimals (18)
const CREDITS_STABLE_AMOUNT_FACTOR: u32 = 100; // 1 credit = 1 cent

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct CreditEntry<M: ManagedTypeApi> {
    pub total_amount: BigUint<M>,
    pub period_amount: BigUint<M>,
    pub daily_cost: BigUint<M>,
    pub last_period_change: u64,
}

#[multiversx_sc::module]
pub trait CreditsModule: config::ConfigModule + dex::DexModule + organization::OrganizationModule + events::EventsModule {
    #[only_owner]
    #[endpoint(initCreditsModule)]
    fn init_credits_module(&self, boost_reward_token: TokenIdentifier, bonus_factor: u8) {
        require!(bonus_factor > 0, "bonus factor can not be zero");

        self.credits_reward_token().set(boost_reward_token);
        self.credits_bonus_factor().set(bonus_factor);
    }

    #[endpoint(setCreditsBonusFactor)]
    fn set_credits_bonus_factor_endpoint(&self, bonus_factor: u8) {
        self.require_caller_is_admin();
        require!(bonus_factor > 0, "bonus factor can not be zero");

        self.credits_bonus_factor().set(bonus_factor);
    }

    #[endpoint(setCreditsCostBase)]
    fn set_credits_cost_base_endpoint(&self, amount: BigUint) {
        self.require_caller_is_admin();
        require!(amount > 0, "can not be zero");
        self.credits_cost_base_amount().set(amount);
    }

    #[endpoint(setCreditsCostExtraPercent)]
    fn set_credits_cost_extra_percent_endpoint(&self, address: ManagedAddress, percent: u32) {
        self.require_caller_is_admin();
        require!(percent > 0 && percent <= 100_00, "invalid percent");
        self.credits_cost_extra_percent(&address).set(percent);
        self.recalculate_daily_cost(&address);
    }

    #[endpoint(setCreditsCostFeature)]
    fn set_credits_cost_feature_amount_endpoint(&self, feature: ManagedBuffer, amount: BigUint) {
        self.require_caller_is_admin();
        require!(amount > 0, "can not be zero");
        self.credits_cost_feature_amount(&feature).set(amount);
    }

    #[payable("*")]
    #[endpoint(boost)]
    fn boost_endpoint(&self, entity: ManagedAddress, swap_contract_opt: OptionalValue<ManagedAddress>) {
        let (payment_token, payment_amount) = self.call_value().egld_or_single_fungible_esdt();
        let caller = self.blockchain().get_caller();

        require!(payment_token.is_valid(), "invalid token id");
        require!(payment_amount > 0, "amount can not be zero");

        // payment in stable token : 1 cent = 1 credit
        if payment_token == self.stable_token().get() {
            let esdt_payment = self.call_value().single_esdt();
            let credits = self.convert_stable_amount_to_credits(&esdt_payment.amount);

            self.boost_by_user(caller, entity, credits);
            self.forward_distribution_to_org(esdt_payment);

            return;
        }

        // swap to stable for energy determination
        let wegld = if payment_token.is_egld() {
            self.wrap_egld(payment_amount)
        } else {
            self.swap_tokens_to_wegld(payment_token.unwrap_esdt(), payment_amount, swap_contract_opt.into_option().unwrap())
        };

        let swapped_payment = self.swap_wegld_to_stable_tokens(wegld.amount);
        let credits = self.convert_stable_amount_to_credits(&swapped_payment.amount);

        self.boost_by_user(caller, entity, credits);
        self.forward_distribution_to_org(swapped_payment);
    }

    #[endpoint(registerExternalBoost)]
    fn register_external_boost_endpoint(&self, booster: ManagedAddress, entity: ManagedAddress, amount: BigUint) {
        self.require_caller_is_admin();
        self.boost_by_user(booster, entity, amount);
    }

    #[endpoint(boostNoReward)]
    fn boost_no_reward_endpoint(&self, entity: ManagedAddress, amount: BigUint) {
        self.require_caller_is_admin();
        self.direct_boost(&entity, &amount);
    }

    #[endpoint(setFeatures)]
    fn set_features_endpoint(&self, features: MultiValueManagedVec<ManagedBuffer>) {
        let caller_entity = self.blockchain().get_caller();
        let features = features.into_vec();

        self.require_entity_exists(&caller_entity);
        self.set_features(&caller_entity, features);
        self.recalculate_daily_cost(&caller_entity);
    }

    #[endpoint(forceDisableFeature)]
    fn disable_feature_endpoint(&self, entity: ManagedAddress, feature: ManagedBuffer) {
        self.require_caller_is_admin();
        self.require_entity_exists(&entity);

        self.disable_feature(&entity, feature);
        self.recalculate_daily_cost(&entity);
    }

    #[view(getCredits)]
    fn get_credits_view(&self, entity_address: ManagedAddress) -> MultiValue2<BigUint, BigUint> {
        if self.credits_entries(&entity_address).is_empty() {
            return (BigUint::zero(), BigUint::zero()).into();
        }

        let entry = self.credits_entries(&entity_address).get();
        let available = self.calculate_available_credits(&entry);

        (available, entry.daily_cost).into()
    }

    #[view(getCreditsInfo)]
    fn get_credits_info_view(&self) -> u8 {
        let bonus_factor = self.credits_bonus_factor().get();

        bonus_factor
    }

    #[inline]
    fn convert_stable_amount_to_credits(&self, amount: &BigUint) -> BigUint {
        amount * &BigUint::from(CREDITS_STABLE_AMOUNT_FACTOR) * &BigUint::from(10u64).pow(REWARD_ADJUSTMENT_DECIMAL_EXPONENT)
    }

    fn boost_by_user(&self, booster: ManagedAddress, entity: ManagedAddress, amount: BigUint) {
        let bonus_factor = self.credits_bonus_factor().get();
        let virtual_amount = &amount * &BigUint::from(bonus_factor);

        self.direct_boost(&entity, &virtual_amount);
        self.credits_total_deposits_amount().update(|current| *current += &virtual_amount);
        self.mint_and_send_reward_tokens(&booster, &virtual_amount);
        self.boost_event(booster, entity, amount, virtual_amount, bonus_factor);
    }

    fn direct_boost(&self, entity: &ManagedAddress, amount: &BigUint) {
        self.require_entity_exists(&entity);
        let mut entry = self.get_or_create_entry(&entity);
        entry.total_amount += amount;
        entry.period_amount += amount;

        self.credits_entries(&entity).set(entry);
    }

    fn recalculate_daily_cost(&self, entity: &ManagedAddress) {
        let mut entry = self.get_or_create_entry(&entity);
        let mut daily_cost = self.credits_cost_base_amount().get();
        let base_extra_percent = self.credits_cost_extra_percent(&entity).get();

        entry.last_period_change = self.blockchain().get_block_timestamp();
        entry.period_amount = self.calculate_available_credits(&entry);

        for feature in self.features(&entity).iter() {
            daily_cost += self.credits_cost_feature_amount(&feature).get();
        }

        entry.daily_cost = if base_extra_percent > 0 {
            let extra_cost = &daily_cost * &BigUint::from(base_extra_percent) / &BigUint::from(100_00u64);
            daily_cost + extra_cost
        } else {
            daily_cost
        };

        self.credits_entries(&entity).set(entry);
    }

    fn calculate_available_credits(&self, entry: &CreditEntry<Self::Api>) -> BigUint {
        let seconds_in_period = self.blockchain().get_block_timestamp() - entry.last_period_change;
        let cost_per_second = &entry.daily_cost / &BigUint::from(86_400u64); // 1 day in seconds
        let credits_consumed = &cost_per_second * &BigUint::from(seconds_in_period);
        let available_credits = if &entry.period_amount > &credits_consumed {
            &entry.period_amount - &credits_consumed
        } else {
            BigUint::zero()
        };

        available_credits
    }

    fn get_or_create_entry(&self, entity_address: &ManagedAddress) -> CreditEntry<Self::Api> {
        if self.credits_entries(&entity_address).is_empty() {
            CreditEntry {
                total_amount: BigUint::zero(),
                period_amount: BigUint::zero(),
                daily_cost: self.credits_cost_base_amount().get(),
                last_period_change: self.blockchain().get_block_timestamp(),
            }
        } else {
            self.credits_entries(&entity_address).get()
        }
    }

    fn mint_and_send_reward_tokens(&self, address: &ManagedAddress, amount: &BigUint) {
        let reward_token = self.credits_reward_token().get();
        self.send().esdt_local_mint(&reward_token, 0, &amount);
        self.send().direct_esdt(&address, &reward_token, 0, &amount);
    }

    fn set_features(&self, entity_address: &ManagedAddress, features: ManagedVec<ManagedBuffer>) {
        self.features(&entity_address).clear();

        for feature in features.into_iter() {
            self.enable_feature(&entity_address, feature);
        }
    }

    fn enable_feature(&self, entity_address: &ManagedAddress, feature: ManagedBuffer) {
        self.features(&entity_address).insert(feature);
    }

    fn disable_feature(&self, entity_address: &ManagedAddress, feature: ManagedBuffer) {
        self.features(&entity_address).swap_remove(&feature);
    }

    #[storage_mapper("credits:entries")]
    fn credits_entries(&self, entity_address: &ManagedAddress) -> SingleValueMapper<CreditEntry<Self::Api>>;

    #[storage_mapper("credits:total_deposits")]
    fn credits_total_deposits_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getBaseDailyCost)]
    #[storage_mapper("credits:cost_base")]
    fn credits_cost_base_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getBaseExtraPercent)]
    #[storage_mapper("credits:cost_extra_percent")]
    fn credits_cost_extra_percent(&self, entity: &ManagedAddress) -> SingleValueMapper<u32>;

    #[view(getCreditsCostFeature)]
    #[storage_mapper("credits:cost_feature")]
    fn credits_cost_feature_amount(&self, feature: &ManagedBuffer) -> SingleValueMapper<BigUint>;

    #[storage_mapper("credits:boost_reward_token")]
    fn credits_reward_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("credits:bonus_factor")]
    fn credits_bonus_factor(&self) -> SingleValueMapper<u8>;

    #[view(getFeatures)]
    #[storage_mapper("credits:features")]
    fn features(&self, entity_address: &ManagedAddress) -> UnorderedSetMapper<ManagedBuffer>;
}
