use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::vote::*;
use entity_esdt::governance::*;
use setup::*;

mod setup;

#[test]
fn it_votes_for_a_proposal() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_for_endpoint(1);

        let proposal = sc.proposals(1).get();

        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 25), proposal.votes_for);
        assert_eq!(managed_biguint!(0), proposal.votes_against);
        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 25), sc.protected_vote_tokens(&managed_token_id!(ENTITY_TOKEN_ID)).get());
    })
    .assert_ok();

    // same vote again to assert it adds up
    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_for_endpoint(1);

        let proposal = sc.proposals(1).get();

        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 50), proposal.votes_for);
        assert_eq!(managed_biguint!(0), proposal.votes_against);
        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 50), sc.protected_vote_tokens(&managed_token_id!(ENTITY_TOKEN_ID)).get());
    })
    .assert_ok();
}

#[test]
fn it_votes_against_a_proposal() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_against_endpoint(1);

        let proposal = sc.proposals(1).get();

        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL), proposal.votes_for);
        assert_eq!(managed_biguint!(25), proposal.votes_against);
        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 25), sc.protected_vote_tokens(&managed_token_id!(ENTITY_TOKEN_ID)).get());
    })
    .assert_ok();

    // same vote again to assert it adds up
    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_against_endpoint(1);

        let proposal = sc.proposals(1).get();

        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL), proposal.votes_for);
        assert_eq!(managed_biguint!(50), proposal.votes_against);
        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL + 50), sc.protected_vote_tokens(&managed_token_id!(ENTITY_TOKEN_ID)).get());
    })
    .assert_ok();
}

#[test]
fn it_sends_a_vote_nft_to_the_voter() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let user_address = setup.user_address.clone();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.user_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_for_endpoint(1);
    })
    .assert_ok();

    // assert voter received vote nft
    setup.blockchain.execute_in_managed_environment(|| {
        setup.blockchain.check_nft_balance(
            &setup.user_address,
            VOTE_NFT_TOKEN_ID,
            2, // nonce 2, the first nft was minted with proposing
            &rust_biguint!(1),
            Some(&VoteNFTAttributes::<DebugApi> {
                proposal_id: 1,
                vote_type: VoteType::For,
                vote_weight: managed_biguint!(25),
                voter: managed_address!(&user_address),
                payment: EsdtTokenPayment::new(managed_token_id!(ENTITY_TOKEN_ID), 0, managed_biguint!(25)),
            }),
        );
    });
}

#[test]
fn it_fails_if_proposal_voting_period_has_ended() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(25), |sc| {
        sc.vote_against_endpoint(1);
    })
    .assert_user_error("proposal is not active");
}
