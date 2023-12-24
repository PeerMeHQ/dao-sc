use entity::config::ConfigModule;
use entity::permission::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_removes_a_role() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role_endpoint(managed_buffer!(b"testrole"));

            sc.remove_role_endpoint(managed_buffer!(b"testrole"));

            assert!(!sc.roles().contains(&managed_buffer!(b"testrole")));
        })
        .assert_ok();
}

#[test]
fn it_fails_to_remove_a_role_that_does_not_exists() {
    let mut setup = EntitySetup::new(entity::contract_obj);

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.remove_role_endpoint(managed_buffer!(b"testrole"));
        })
        .assert_user_error("role does not exist");
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.remove_role_endpoint(managed_buffer!(b"testrole"));
        })
        .assert_user_error("action not allowed by user");
}

#[test]
fn it_unassigns_all_users_from_the_role() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let owner_address = &setup.owner_address.clone();
    let user_one = setup.blockchain.create_user_account(&rust_biguint!(0));
    let user_two = setup.blockchain.create_user_account(&rust_biguint!(0));
    let user_three = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&user_one), managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&user_two), managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&user_three), managed_buffer!(b"builder"));
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.remove_role_endpoint(managed_buffer!(b"builder"));

            assert!(!sc.roles().contains(&managed_buffer!(b"builder")));
            assert_eq!(0, sc.roles_member_amount(&managed_buffer!(b"builder")).get());

            let leader_user_id = sc.users().get_user_id(&managed_address!(&owner_address));
            assert!(!sc.user_roles(leader_user_id).contains(&managed_buffer!(b"builder")));

            let user_one_id = sc.users().get_user_id(&managed_address!(&user_one));
            assert!(!sc.user_roles(user_one_id).contains(&managed_buffer!(b"builder")));

            let user_two_id = sc.users().get_user_id(&managed_address!(&user_one));
            assert!(!sc.user_roles(user_two_id).contains(&managed_buffer!(b"builder")));

            let user_three_id = sc.users().get_user_id(&managed_address!(&user_one));
            assert!(!sc.user_roles(user_three_id).contains(&managed_buffer!(b"builder")));
        })
        .assert_ok();
}
