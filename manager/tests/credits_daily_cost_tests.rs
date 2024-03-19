use manager::config::*;
use manager::credits::*;
use manager::features::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_recalculates_to_daily_base_cost_when_no_features_set() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_address = setup.contract_entity_template.address_ref();

    setup.blockchain.set_esdt_balance(&entity_address, NATIVE_TOKEN, &rust_biguint!(100));

    setup
        .blockchain
        .execute_esdt_transfer(&entity_address, &setup.contract, NATIVE_TOKEN, 0, &rust_biguint!(50), |sc| {
            sc.entities().insert(managed_address!(&entity_address));

            sc.credits_cost_base_amount().set(managed_biguint!(20));

            sc.recalculate_daily_cost(&managed_address!(&entity_address));

            let actual = sc.credits_entries(&managed_address!(&entity_address)).get();

            assert_eq!(managed_biguint!(20), actual.daily_cost);
        })
        .assert_ok();
}

#[test]
fn it_recalculates_to_daily_base_cost_when_extra_base_percent_set() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_address = setup.contract_entity_template.address_ref();

    setup
        .blockchain
        .execute_tx(&entity_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.entities().insert(managed_address!(&entity_address));

            sc.credits_cost_base_amount().set(managed_biguint!(100));
            sc.credits_cost_extra_percent(&managed_address!(&entity_address)).set(20_00); // 20%

            sc.recalculate_daily_cost(&managed_address!(&entity_address));

            let actual = sc.credits_entries(&managed_address!(&entity_address)).get();

            assert_eq!(managed_biguint!(120), actual.daily_cost);
        })
        .assert_ok();
}

#[test]
fn it_recalculates_daily_cost_with_features() {
    let mut setup = setup::setup_manager(manager::contract_obj);
    let entity_address = setup.contract_entity_template.address_ref();

    setup.blockchain.set_esdt_balance(&entity_address, NATIVE_TOKEN, &rust_biguint!(100));

    setup
        .blockchain
        .execute_esdt_transfer(&entity_address, &setup.contract, NATIVE_TOKEN, 0, &rust_biguint!(50), |sc| {
            sc.entities().insert(managed_address!(&entity_address));

            sc.credits_cost_base_amount().set(managed_biguint!(20));

            sc.credits_cost_feature_amount(&managed_buffer!(b"feature1")).set(managed_biguint!(5));
            sc.credits_cost_feature_amount(&managed_buffer!(b"feature2")).set(managed_biguint!(10));

            sc.enable_feature(&managed_address!(&entity_address), managed_buffer!(b"feature1"));
            sc.enable_feature(&managed_address!(&entity_address), managed_buffer!(b"feature2"));

            sc.recalculate_daily_cost(&managed_address!(&entity_address));

            let actual = sc.credits_entries(&managed_address!(&entity_address)).get();

            assert_eq!(managed_biguint!(35), actual.daily_cost);
        })
        .assert_ok();
}
