use elrond_wasm_debug::*;
use entity::config::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_assigns_a_role() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));

            // TODO: switch to endpoint, currently a bug in wasm-rs lib when SC calls itself
            sc.assign_role(managed_address!(user_address), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let user_id = sc.users().get_user_id(&managed_address!(user_address));

            assert!(sc.user_roles(user_id).contains(&managed_buffer!(b"testrole")));
            assert_eq!(1, sc.roles_member_amount(&managed_buffer!(b"testrole")).get());
        })
        .assert_ok();
}

#[test]
fn it_only_increases_role_member_count_once_per_assigned_user() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));
            sc.assign_role(managed_address!(user_address), managed_buffer!(b"testrole"));

            // same user again
            // TODO: switch to endpoint, currently a bug in wasm-rs lib when SC calls itself
            sc.assign_role(managed_address!(user_address), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(1, sc.roles_member_amount(&managed_buffer!(b"testrole")).get());
        })
        .assert_ok();
}

#[test]
fn it_must_call_itself() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));

            sc.assign_role_endpoint(managed_buffer!(b"testrole"), managed_address!(user_address));
        })
        .assert_user_error("action not allowed by user");
}