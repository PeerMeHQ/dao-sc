use entity::permission::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_creates_a_permission() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let sc_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_permission_endpoint(
                managed_buffer!(b"testperm"),
                managed_biguint!(0),
                managed_address!(sc_address),
                managed_buffer!(b"endpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );

            assert!(sc.permissions().contains(&managed_buffer!(b"testperm")));

            let actual_permission_details = sc.permission_details(&managed_buffer!(b"testperm")).get();

            assert_eq!(managed_address!(sc_address), actual_permission_details.destination);
            assert_eq!(managed_buffer!(b"endpoint"), actual_permission_details.endpoint);
        })
        .assert_ok();
}

#[test]
fn it_fails_when_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = &setup.user_address;
    let sc_address = setup.contract.address_ref();

    setup
        .blockchain
        .execute_tx(&user_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_permission_endpoint(
                managed_buffer!(b"testperm"),
                managed_biguint!(0),
                managed_address!(sc_address),
                managed_buffer!(b"endpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
        })
        .assert_user_error("action not allowed by user");
}
