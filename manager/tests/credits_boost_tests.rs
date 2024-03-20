use manager::config::*;
use manager::credits::*;
use multiversx_sc::codec::multi_types::OptionalValue;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_increases_deposited_amounts_in_the_storage() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_address = setup.contract_entity_template.address_ref();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, STABLE_TOKEN, 0, &rust_biguint!(50), |sc| {
            sc.entities().insert(managed_address!(&entity_address));

            sc.boost_endpoint(managed_address!(&entity_address), OptionalValue::None);

            let actual = sc.credits_entries(&managed_address!(&entity_address)).get();

            assert_eq!(managed_biguint!(50_00_000000000000), actual.total_amount);
            assert_eq!(managed_biguint!(50_00_000000000000), actual.period_amount);
            assert_eq!(managed_biguint!(50_00_000000000000), sc.credits_total_deposits_amount().get());
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, STABLE_TOKEN, 0, &rust_biguint!(25), |sc| {
            sc.boost_endpoint(managed_address!(&entity_address), OptionalValue::None);

            let actual = sc.credits_entries(&managed_address!(&entity_address)).get();

            assert_eq!(managed_biguint!(75_00_000000000000), actual.total_amount);
            assert_eq!(managed_biguint!(75_00_000000000000), actual.period_amount);
            assert_eq!(managed_biguint!(75_00_000000000000), sc.credits_total_deposits_amount().get());
        })
        .assert_ok();
}

#[test]
fn it_fails_when_the_entity_does_not_exist() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_address = setup.contract_entity_template.address_ref();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, STABLE_TOKEN, 0, &rust_biguint!(25), |sc| {
            sc.boost_endpoint(managed_address!(&entity_address), OptionalValue::None);
        })
        .assert_user_error("entity does not exist");
}
