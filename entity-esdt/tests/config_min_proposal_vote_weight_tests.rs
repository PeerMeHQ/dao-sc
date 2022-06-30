use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::*;
use setup::*;

mod setup;

#[test]
fn it_changes_min_proposal_vote_weight() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_tx(setup.contract.address_ref(), &setup.contract, &rust_biguint!(0), |sc| {
        sc.change_min_proposal_vote_weight_endpoint(managed_biguint!(1000));

        assert_eq!(sc.min_proposal_vote_weight().get(), managed_biguint!(1000));
    })
    .assert_ok();
}

#[test]
fn it_fails_if_caller_not_self() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.change_min_proposal_vote_weight_endpoint(managed_biguint!(1000));
    })
    .assert_user_error("action not allowed by user");
}
