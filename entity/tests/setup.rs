multiversx_sc::imports!();

use entity::config::*;
use entity::governance::*;
use entity::permission::PermissionModule;
use entity::permission::ROLE_BUILTIN_LEADER;
use entity::plug::*;
use entity::*;
use multiversx_sc_scenario::testing_framework::BlockchainStateWrapper;
use multiversx_sc_scenario::testing_framework::ContractObjWrapper;
use multiversx_sc_scenario::*;

pub const ENTITY_GOV_TOKEN_ID: &[u8] = b"SUPER-abcdef";
pub const ENTITY_GOV_TOKEN_SUPPLY: u64 = 1_000;
pub const ENTITY_FAKE_TOKEN_ID: &[u8] = b"FAKE-abcdef";
pub const MIN_PROPOSE_WEIGHT: u64 = 2;
pub const POLL_DEFAULT_ID: u8 = 0;
pub const QURUM: u64 = 50;
pub const WASM_PATH: &'static str = "output/entity.wasm";
pub const PLUG_EXAMPLE_WASM_PATH: &'static str = "tests/external/plug-example.wasm";

#[allow(dead_code)]
pub struct EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> entity::ContractObj<DebugApi>,
{
    pub blockchain: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_address: Address,
    pub trusted_host_address: Address,
    pub contract: ContractObjWrapper<entity::ContractObj<DebugApi>, ObjBuilder>,
}

impl<ObjBuilder> EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> entity::ContractObj<DebugApi>,
{
    pub fn new(builder: ObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut blockchain = BlockchainStateWrapper::new();
        let owner_address = blockchain.create_user_account(&rust_zero);
        let user_address = blockchain.create_user_account(&rust_biguint!(1000));
        let trusted_host_address = blockchain.create_user_account(&rust_zero);
        let contract = blockchain.create_sc_account(&rust_biguint!(100), Some(&owner_address), builder, WASM_PATH);

        blockchain.set_esdt_balance(&owner_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY));
        blockchain.set_esdt_balance(&user_address, ENTITY_GOV_TOKEN_ID, &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY));
        blockchain.set_esdt_balance(&user_address, ENTITY_FAKE_TOKEN_ID, &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY));

        blockchain
            .execute_tx(&owner_address, &contract, &rust_zero, |sc| {
                sc.init(managed_address!(&trusted_host_address), managed_address!(&owner_address));

                // disable trusted host for tests
                sc.trusted_host_address().clear();
            })
            .assert_ok();

        Self {
            blockchain,
            owner_address,
            user_address,
            trusted_host_address,
            contract,
        }
    }

    pub fn configure_gov_token(&mut self, lock_vote_tokens: bool) {
        self.blockchain
            .execute_tx(&self.owner_address, &self.contract, &rust_biguint!(0), |sc| {
                sc.configure_governance_token(managed_token_id!(ENTITY_GOV_TOKEN_ID), managed_biguint!(MIN_PROPOSE_WEIGHT), lock_vote_tokens);

                // override defaults
                sc.quorum().set(managed_biguint!(QURUM));
                sc.min_propose_weight().set(managed_biguint!(MIN_PROPOSE_WEIGHT));

                // assert
                assert_eq!(lock_vote_tokens, sc.lock_vote_tokens(&managed_token_id!(ENTITY_GOV_TOKEN_ID)).get());
            })
            .assert_ok();
    }

    pub fn configure_leaderless(&mut self) {
        let owner_address = self.owner_address.clone();

        self.blockchain
            .execute_tx(&self.owner_address, &self.contract, &rust_biguint!(0), |sc| {
                sc.unassign_role(managed_address!(&owner_address), managed_buffer!(ROLE_BUILTIN_LEADER));
            })
            .assert_ok();
    }

    pub fn configure_trusted_host(&mut self) {
        let trusted_host_address = self.trusted_host_address.clone();

        self.blockchain
            .execute_tx(&self.owner_address, &self.contract, &rust_biguint!(0), |sc| {
                sc.trusted_host_address().set(managed_address!(&trusted_host_address));
            })
            .assert_ok();
    }

    pub fn configure_plug(&mut self, quorum: u64, min_propose_weight: u64) {
        let plug_contract = self
            .blockchain
            .create_sc_account(&rust_biguint!(0), Some(&self.owner_address), fakes::contract_obj, PLUG_EXAMPLE_WASM_PATH);

        self.blockchain
            .execute_tx(&self.owner_address, &self.contract, &rust_biguint!(0), |sc| {
                sc.plug_contract().set(managed_address!(&plug_contract.address_ref()));
                sc.try_change_quorum(managed_biguint!(quorum));
                sc.try_change_min_propose_weight(managed_biguint!(min_propose_weight));
            })
            .assert_ok();
    }
}

#[test]
fn it_initializes_the_contract() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_query(&setup.contract, |_| {
            //
        })
        .assert_ok();
}

mod fakes {
    multiversx_sc::imports!();

    #[multiversx_sc::contract]
    pub trait FakePlug {
        #[init]
        fn init(&self) {}

        #[view(getDaoVoteWeight)]
        fn get_dao_vote_weight_view(&self, _address: ManagedAddress, _token: OptionalValue<TokenIdentifier>) -> BigUint {
            BigUint::from(100u64)
        }

        #[view(getDaoMembers)]
        fn get_dao_members_view(&self, _token: OptionalValue<TokenIdentifier>) -> MultiValueEncoded<MultiValue2<ManagedAddress, BigUint>> {
            MultiValueEncoded::new()
        }
    }
}
