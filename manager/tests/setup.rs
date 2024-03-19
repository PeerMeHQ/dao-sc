multiversx_sc::imports!();

use manager::config::*;
use manager::credits::*;
use manager::dex::*;
use manager::*;
use multiversx_sc_scenario::testing_framework::BlockchainStateWrapper;
use multiversx_sc_scenario::testing_framework::ContractObjWrapper;
use multiversx_sc_scenario::*;

pub const WEGLD_TOKEN: &[u8] = b"WEGLD-abcdef";
pub const NATIVE_TOKEN: &[u8] = b"SUPER-abcdef";
pub const STABLE_TOKEN: &[u8] = b"USD-abcdef";
pub const BOOST_REWARD_TOKEN_ID: &[u8] = b"SUPERPOWER-abcdef";
pub const COST_AMOUNT_ENTITY_CREATION: u64 = 500;

pub const WASM_PATH: &'static str = "output/manager.wasm";
pub const WASM_PATH_ENTITY_TEMPLATE: &'static str = "output/entity.wasm";

#[allow(dead_code)]
pub struct ManagerSetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> manager::ContractObj<DebugApi>,
{
    pub blockchain: BlockchainStateWrapper,
    pub owner_address: Address,
    pub trusted_host_address: Address,
    pub user_address: Address,
    pub contract: ContractObjWrapper<manager::ContractObj<DebugApi>, ObjBuilder>,
    pub contract_entity_template: ContractObjWrapper<manager::ContractObj<DebugApi>, ObjBuilder>,
}

pub fn setup_manager<ObjBuilder>(builder: ObjBuilder) -> ManagerSetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> manager::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain = BlockchainStateWrapper::new();
    let owner_address = blockchain.create_user_account(&rust_zero);
    let trusted_host_address = blockchain.create_user_account(&rust_biguint!(1));
    let user_address = blockchain.create_user_account(&rust_zero);
    let contract = blockchain.create_sc_account(&rust_zero, Some(&owner_address), builder, WASM_PATH);
    let contract_entity_template = blockchain.create_sc_account(&rust_zero, Some(&owner_address), builder, WASM_PATH_ENTITY_TEMPLATE);

    blockchain.set_esdt_balance(&owner_address, NATIVE_TOKEN, &rust_biguint!(10_000));
    blockchain.set_esdt_balance(&owner_address, STABLE_TOKEN, &rust_biguint!(10_000));
    blockchain.set_esdt_local_roles(contract.address_ref(), BOOST_REWARD_TOKEN_ID, &[EsdtLocalRole::Mint]);

    blockchain
        .execute_tx(&owner_address, &contract, &rust_zero, |sc| {
            sc.init(
                managed_address!(contract_entity_template.address_ref()),
                managed_address!(&trusted_host_address),
                managed_token_id!(NATIVE_TOKEN),
                managed_biguint!(COST_AMOUNT_ENTITY_CREATION),
            );

            sc.init_credits_module(managed_token_id!(BOOST_REWARD_TOKEN_ID), 1);

            sc.init_dex_module(
                managed_token_id!(WEGLD_TOKEN),
                managed_address!(contract.address_ref()), // dummy
                managed_address!(contract.address_ref()), // dummy
                managed_token_id!(STABLE_TOKEN),
                managed_address!(contract.address_ref()), // dummy
            )
        })
        .assert_ok();

    ManagerSetup {
        blockchain,
        owner_address,
        trusted_host_address,
        user_address,
        contract,
        contract_entity_template,
    }
}

#[test]
fn it_initializes_the_contract() {
    let mut setup = setup_manager(manager::contract_obj);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(managed_token_id!(NATIVE_TOKEN), sc.native_token().get());
        })
        .assert_ok();
}
