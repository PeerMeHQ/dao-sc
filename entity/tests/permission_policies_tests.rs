use entity::config::*;
use entity::permission::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_fails_create_weighted_policy_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_policy_weighted_endpoint(managed_buffer!(b"testrole"), managed_buffer!(b"testperm"), managed_biguint!(0), 0);
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_fails_create_one_policy_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_policy_one_endpoint(managed_buffer!(b"testrole"), managed_buffer!(b"testperm"));
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_fails_create_all_policy_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_policy_all_endpoint(managed_buffer!(b"testrole"), managed_buffer!(b"testperm"));
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_fails_create_quorum_policy_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_policy_quorum_endpoint(managed_buffer!(b"testrole"), managed_buffer!(b"testperm"), 2);
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_fails_create_majority_policy_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_policy_majority_endpoint(managed_buffer!(b"testrole"), managed_buffer!(b"testperm"));
        })
        .assert_user_error("action not allowed by user");
}
