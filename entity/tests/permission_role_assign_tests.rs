use entity::config::*;
use entity::permission::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_assigns_a_role() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role_endpoint(managed_buffer!(b"testrole"));

            sc.assign_role_endpoint(managed_buffer!(b"testrole"), managed_address!(user_address));

            let user_id = sc.users().get_user_id(&managed_address!(user_address));

            assert!(sc.user_roles(user_id).contains(&managed_buffer!(b"testrole")));
            assert_eq!(1, sc.roles_member_amount(&managed_buffer!(b"testrole")).get());
        })
        .assert_ok();
}

#[test]
fn it_creates_the_role_when_it_does_not_exist() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role_endpoint(managed_buffer!(b"testrole"), managed_address!(user_address));

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
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role_endpoint(managed_buffer!(b"testrole"));
            sc.assign_role_endpoint(managed_buffer!(b"testrole"), managed_address!(user_address));

            // same user again
            sc.assign_role_endpoint(managed_buffer!(b"testrole"), managed_address!(user_address));

            assert_eq!(1, sc.roles_member_amount(&managed_buffer!(b"testrole")).get());
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role_endpoint(managed_buffer!(b"testrole"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role_endpoint(managed_buffer!(b"testrole"), managed_address!(user_address));
        })
        .assert_user_error("action not allowed by user");
}
