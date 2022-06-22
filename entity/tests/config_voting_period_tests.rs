use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::*;
use setup::*;

mod setup;

#[test]
fn it_changes_the_voting_period_on_unsealed_entity() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sealed().set(SEALED_NOT_SET);

            sc.change_voting_period_in_minutes_endpoint(60);

            assert_eq!(sc.voting_period_in_minutes().get(), 60);
        })
        .assert_ok();
}

#[test]
fn it_changes_the_voting_period_if_entity_is_sealed_but_contract_calls_itself() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.sealed().set(SEALED_ON);

            sc.change_voting_period_in_minutes_endpoint(60);

            assert_eq!(sc.voting_period_in_minutes().get(), 60);
        })
        .assert_ok();
}

#[test]
fn it_fails_if_the_entity_is_sealed() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sealed().set(SEALED_ON);

            sc.change_voting_period_in_minutes_endpoint(60);
        })
        .assert_user_error("action not allowed by user");
}
