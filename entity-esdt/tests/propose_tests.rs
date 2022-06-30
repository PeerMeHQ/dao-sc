use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::proposal::{ProposalModule, Action};
use entity_esdt::governance::vote::*;
use entity_esdt::governance::*;
use entity_esdt::permission::{PermissionModule, ROLE_BUILTIN_LEADER};
use setup::*;

mod setup;

#[test]
fn it_creates_a_proposal() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(
            managed_buffer!(b"id"),
            managed_buffer!(b"content hash"),
            managed_buffer!(b"content signature"),
            managed_buffer!(b""),
            MultiValueManagedVec::new(),
        );
    })
    .assert_ok();

    setup.blockchain.execute_query(&setup.contract, |sc| {
        let proposal = sc.proposals(1).get();

        assert_eq!(1, proposal.id);
        assert_eq!(managed_address!(&owner_address), proposal.proposer);
        assert_eq!(managed_buffer!(b"content hash"), proposal.content_hash);
        assert_eq!(managed_buffer!(b""), proposal.actions_hash);
        assert_eq!(false, proposal.was_executed);
        assert_eq!(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL), proposal.votes_for);
        assert_eq!(managed_biguint!(0), proposal.votes_against);

        assert_eq!(2, sc.next_proposal_id().get());
    })
    .assert_ok();
}

#[test]
fn it_creates_a_proposal_with_actions() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let proposer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"any")]);

        sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b"content hash"), managed_buffer!(b"content signature"), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.execute_query(&setup.contract, |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        let expected = sc.calculate_actions_hash(&ManagedVec::from(actions));

        let proposal = sc.proposals(1).get();

        assert_eq!(expected, proposal.actions_hash);
    })
    .assert_ok();
}

#[test]
fn it_sends_a_vote_nft_to_the_voter() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let owner_address = setup.owner_address.clone();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(
            managed_buffer!(b"id"),
            managed_buffer!(b"content hash"),
            managed_buffer!(b"content signature"),
            managed_buffer!(b""),
            MultiValueManagedVec::new(),
        );
    })
    .assert_ok();

    setup.blockchain.execute_in_managed_environment(|| {
        setup.blockchain.check_nft_balance(
            &setup.owner_address,
            VOTE_NFT_TOKEN_ID,
            1,
            &rust_biguint!(1),
            Some(&VoteNFTAttributes::<DebugApi> {
                proposal_id: 1,
                vote_type: VoteType::For,
                vote_weight: managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL),
                voter: managed_address!(&owner_address),
                payment: EsdtTokenPayment::new(managed_token_id!(ENTITY_TOKEN_ID), 0, managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL)),
            }),
        );
    });
}

#[test]
fn it_fails_if_bad_token() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_esdt_transfer(&setup.user_address, &setup.contract, ENTITY_FAKE_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(
            managed_buffer!(b"id"),
            managed_buffer!(b""),
            managed_buffer!(b""),
            managed_buffer!(b""),
            MultiValueManagedVec::new(),
        );
    })
    .assert_user_error("invalid token");
}

#[test]
fn it_fails_if_bad_vote_weight_amount() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_esdt_transfer(&setup.user_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL - 1), |sc| {
        sc.propose_endpoint(
            managed_buffer!(b"id"),
            managed_buffer!(b""),
            managed_buffer!(b""),
            managed_buffer!(b""),
            MultiValueManagedVec::new(),
        );
    })
    .assert_user_error("insufficient vote weight");
}

#[test]
fn it_fails_if_trusted_host_id_is_already_known() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup.blockchain.execute_esdt_transfer(&setup.user_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(
            managed_buffer!(b"thesame"),
            managed_buffer!(b""),
            managed_buffer!(b""),
            managed_buffer!(b""),
            MultiValueManagedVec::new(),
        );
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.user_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
        sc.propose_endpoint(
            managed_buffer!(b"thesame"),
            managed_buffer!(b""),
            managed_buffer!(b""),
            managed_buffer!(b""),
            MultiValueManagedVec::new(),
        );
    })
    .assert_user_error("proposal already registered");
}
