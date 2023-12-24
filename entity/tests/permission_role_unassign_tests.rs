use entity::config::*;
use entity::permission::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_unassigns_a_role() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));

            // TODO: switch to endpoint, currently a bug in wasm-rs lib when SC calls itself
            sc.assign_role(managed_address!(user_address), managed_buffer!(b"testrole"));

            sc.unassign_role(managed_address!(user_address), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            let user_id = sc.users().get_user_id(&managed_address!(user_address));

            assert!(sc.user_roles(user_id).is_empty());
            assert_eq!(0, sc.roles_member_amount(&managed_buffer!(b"testrole")).get());
        })
        .assert_ok();
}

#[test]
fn it_removes_the_leader_role_when_last_leader_is_unassigned() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup.configure_gov_token(true);

    setup
        .blockchain
        .execute_tx(&owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.unassign_role(managed_address!(&owner_address), managed_buffer!(ROLE_BUILTIN_LEADER));

            assert!(sc.roles().is_empty());
            assert_eq!(0, sc.roles_member_amount(&managed_buffer!(ROLE_BUILTIN_LEADER)).get());
        })
        .assert_ok();
}

#[test]
fn it_only_decreases_role_member_count_once_per_unassigned_user() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));
            sc.unassign_role(managed_address!(user_address), managed_buffer!(b"testrole"));

            // same user again
            // TODO: switch to endpoint, currently a bug in wasm-rs lib when SC calls itself
            sc.unassign_role(managed_address!(user_address), managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(0, sc.roles_member_amount(&managed_buffer!(b"testrole")).get());
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"testrole"));

            sc.unassign_role_endpoint(managed_buffer!(b"testrole"), managed_address!(user_address));
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_fails_to_unassign_last_leader_when_not_gov_token_is_set() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup
        .blockchain
        .execute_tx(&setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.unassign_role_endpoint(managed_buffer!(ROLE_BUILTIN_LEADER), managed_address!(&owner_address));
        })
        .assert_user_error("can not remove last leader: gov token or plug required");

    setup.configure_gov_token(false);

    setup
        .blockchain
        .execute_tx(&setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.unassign_role_endpoint(managed_buffer!(ROLE_BUILTIN_LEADER), managed_address!(&owner_address));
        })
        .assert_ok();
}

#[test]
fn it_fails_to_unassign_last_leader_when_not_plugged() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup
        .blockchain
        .execute_tx(&setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.unassign_role_endpoint(managed_buffer!(ROLE_BUILTIN_LEADER), managed_address!(&owner_address));
        })
        .assert_user_error("can not remove last leader: gov token or plug required");

    setup.configure_plug(1, 1);

    setup
        .blockchain
        .execute_tx(&setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.unassign_role_endpoint(managed_buffer!(ROLE_BUILTIN_LEADER), managed_address!(&owner_address));
        })
        .assert_ok();
}
