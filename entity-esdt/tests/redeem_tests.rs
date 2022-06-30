use elrond_wasm::types::MultiValueManagedVec;
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::GovernanceModule;
use setup::*;

mod setup;

#[test]
fn it_redeems_vote_nfts() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(
            managed_buffer!(b"id"),
            managed_buffer!(b"content hash"),
            managed_buffer!(b"content signature"),
            managed_buffer!(b""),
            MultiValueManagedVec::new(),
        );
    }).assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, VOTE_NFT_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
        sc.redeem_endpoint();
    }).assert_ok();

    setup.blockchain.check_esdt_balance(&setup.owner_address, ENTITY_TOKEN_ID, &rust_biguint!(ENTITY_TOKEN_SUPPLY));
}

#[test]
fn it_fails_if_voting_period_has_not_ended() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup
        .blockchain
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract,
            ENTITY_TOKEN_ID,
            0,
            &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
            |sc| {
                sc.propose_endpoint(
                    managed_buffer!(b"id"),
                    managed_buffer!(b"content hash"),
                    managed_buffer!(b"content signature"),
                    managed_buffer!(b""),
                    MultiValueManagedVec::new(),
                );
            },
        )
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, VOTE_NFT_TOKEN_ID, 1, &rust_biguint!(1), |sc| {
            sc.redeem_endpoint();
        })
        .assert_user_error("proposal is still active");
}
