use manager::config::*;
use manager::credits::*;
use multiversx_sc_scenario::*;

mod setup;

#[test]
fn it_registers_external_boosts_from_the_trusted_host() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let booster = setup.user_address.clone();
    let entity_address = setup.contract_entity_template.address_ref();

    setup
        .blockchain
        .execute_tx(&setup.trusted_host_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.entities().insert(managed_address!(&entity_address));

            sc.register_external_boost_endpoint(managed_address!(&booster), managed_address!(&entity_address), managed_biguint!(25));

            let actual = sc.credits_entries(&managed_address!(&entity_address)).get();

            assert_eq!(managed_biguint!(25), actual.total_amount);
            assert_eq!(managed_biguint!(25), actual.period_amount);
            assert_eq!(managed_biguint!(25), sc.credits_total_deposits_amount().get());
        })
        .assert_ok();
}

#[test]
fn it_registers_external_boosts_from_the_owner() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let booster = setup.user_address.clone();
    let entity_address = setup.contract_entity_template.address_ref();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.entities().insert(managed_address!(&entity_address));

            sc.register_external_boost_endpoint(managed_address!(&booster), managed_address!(&entity_address), managed_biguint!(25));

            let actual = sc.credits_entries(&managed_address!(&entity_address)).get();

            assert_eq!(managed_biguint!(25), actual.total_amount);
            assert_eq!(managed_biguint!(25), actual.period_amount);
            assert_eq!(managed_biguint!(25), sc.credits_total_deposits_amount().get());
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_is_user() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let booster = setup.user_address.clone();
    let entity_address = setup.contract_entity_template.address_ref();

    setup
        .blockchain
        .execute_tx(&setup.user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.register_external_boost_endpoint(managed_address!(&booster), managed_address!(&entity_address), managed_biguint!(25));
        })
        .assert_user_error("caller must be admin");
}

#[test]
fn it_fails_when_the_entity_does_not_exist() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let booster = setup.user_address.clone();
    let entity_address = setup.contract_entity_template.address_ref();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.register_external_boost_endpoint(managed_address!(&booster), managed_address!(&entity_address), managed_biguint!(25));
        })
        .assert_user_error("entity does not exist");
}
