use manager::config::*;
use manager::credits::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_increases_deposited_amounts_in_the_storage() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_address = setup.contract_entity_template.address_ref();

    setup.blockchain.set_esdt_balance(&entity_address, COST_TOKEN_ID, &rust_biguint!(100));

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(50), |sc| {
            sc.entities().insert(managed_address!(&entity_address));

            sc.boost_endpoint(managed_address!(&entity_address));

            let actual = sc.credits_entries(&managed_address!(&entity_address)).get();

            assert_eq!(managed_biguint!(50), actual.total_amount);
            assert_eq!(managed_biguint!(50), actual.period_amount);
            assert_eq!(managed_biguint!(50), sc.credits_total_deposits_amount().get());
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.boost_endpoint(managed_address!(&entity_address));

            let actual = sc.credits_entries(&managed_address!(&entity_address)).get();

            assert_eq!(managed_biguint!(75), actual.total_amount);
            assert_eq!(managed_biguint!(75), actual.period_amount);
            assert_eq!(managed_biguint!(75), sc.credits_total_deposits_amount().get());
        })
        .assert_ok();
}

#[test]
fn it_fails_when_the_entity_does_not_exist() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_address = setup.contract_entity_template.address_ref();

    setup.blockchain.set_esdt_balance(&entity_address, COST_TOKEN_ID, &rust_biguint!(100));

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, COST_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
            sc.boost_endpoint(managed_address!(&entity_address));
        })
        .assert_user_error("entity does not exist");
}
