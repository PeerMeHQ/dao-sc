use elrond_wasm_debug::*;
use entity::*;
use setup::*;

mod setup;

#[test]
fn it_ejects_an_entity() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = setup.user_address.clone();

    setup
        .blockchain
        .execute_tx(&setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
            sc.eject_endpoint(managed_address!(&user_address));
        })
        .assert_ok();
}

#[test]
fn it_fails_if_caller_not_self() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let user_address = setup.user_address.clone();

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.eject_endpoint(managed_address!(&user_address));
        })
        .assert_user_error("action not allowed by user");
}
