use entity::permission::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_creates_a_role() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role_endpoint(managed_buffer!(b"testrole"));

            assert!(sc.roles().contains(&managed_buffer!(b"testrole")));
        })
        .assert_ok();
}

#[test]
fn it_fails_to_create_a_role_that_already_exists() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role_endpoint(managed_buffer!(b"testrole"));

            sc.create_role_endpoint(managed_buffer!(b"testrole"));
        })
        .assert_user_error("role already exists");
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role_endpoint(managed_buffer!(b"testrole"));
        })
        .assert_user_error("action not allowed by user");
}
