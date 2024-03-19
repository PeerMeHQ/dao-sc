use manager::config::*;
use manager::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;
mod setup;

#[test]
#[ignore]
fn it_creates_an_entity() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));

    setup.blockchain.set_esdt_balance(&caller, NATIVE_TOKEN, &rust_biguint!(5000));

    let manager_addr = setup.contract.address_ref().clone();
    let new_entity_wrapper = setup.blockchain.prepare_deploy_from_sc(&manager_addr, entity::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(&caller, &setup.contract, NATIVE_TOKEN, 0, &rust_biguint!(1000), |sc| {
            sc.create_entity_endpoint(MultiValueManagedVec::new());

            let new_entity_address = managed_address!(new_entity_wrapper.address_ref());

            assert!(sc.entities().contains(&new_entity_address));
        })
        .assert_ok();
}

#[test]
fn it_fails_when_wrong_cost_amount() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let caller = setup.blockchain.create_user_account(&rust_biguint!(1));
    let wrong_cost_amount = COST_AMOUNT_ENTITY_CREATION - 1u64;

    setup.blockchain.set_egld_balance(&caller, &rust_biguint!(5000));

    setup
        .blockchain
        .execute_tx(&caller, &setup.contract, &rust_biguint!(wrong_cost_amount), |sc| {
            sc.create_entity_endpoint(MultiValueManagedVec::new());
        })
        .assert_user_error("invalid cost amount");
}
