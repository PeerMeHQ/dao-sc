use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::*;
use setup::*;

mod setup;

#[test]
fn it_changes_the_governance_token() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
        sc.change_gov_token_endpoint(managed_token_id!(b"GOV-123456"));

        assert_eq!(sc.governance_token_id().get(), managed_token_id!(b"GOV-123456"));
    })
    .assert_ok();
}

#[test]
fn it_fails_if_caller_not_self() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.change_gov_token_endpoint(managed_token_id!(b"GOV-123456"));
    })
    .assert_user_error("action not allowed by user");
}
