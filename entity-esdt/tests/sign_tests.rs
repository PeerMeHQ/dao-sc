use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::*;
use entity_esdt::permission::*;
use setup::*;

mod setup;

#[test]
fn it_signs_a_proposal() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let signer_address = &setup.user_address;
    let mut proposal_id: u64 = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role(managed_buffer!(b"builder"));
        sc.assign_role(managed_address!(&signer_address), managed_buffer!(b"builder"));
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&signer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_tx(&signer_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);

        assert_eq!(1, sc.proposal_signers(proposal_id, &managed_buffer!(b"builder")).len());
    })
    .assert_ok();
}
