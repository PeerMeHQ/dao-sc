use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::proposal::*;
use entity_esdt::governance::*;
use entity_esdt::permission::*;
use setup::*;

mod setup;

#[test]
fn it_returns_active_for_a_newly_created_proposal() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let proposer_address = &setup.user_address;
    let mut proposal_id = 0;

    setup.blockchain.set_block_timestamp(0);

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Active, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_returns_defeated_if_for_votes_quorum_not_met() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let mut proposal_id = 0;

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM - 10), |sc| {
        proposal_id= sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(9), |sc| {
        sc.vote_for_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_returns_defeated_if_quorum_met_but_votes_against_is_more_than_for() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let mut proposal_id = 0;

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(10), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        sc.vote_for_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM * 2), |sc| {
        sc.vote_against_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_applies_the_default_quorum_to_proposals_with_actions_that_do_not_require_any_permissions() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let proposer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(10), |sc| {
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

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(1); // go back in time

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        sc.vote_for_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_returns_succeeded_if_for_votes_quorum_met_and_more_for_than_against_votes() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let mut proposal_id = 0;

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), managed_buffer!(b""), MultiValueManagedVec::new());
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(20), |sc| {
        sc.vote_for_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(10), |sc| {
        sc.vote_against_endpoint(proposal_id);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(proposal_id));
    })
    .assert_ok();
}

#[test]
fn it_returns_executed_for_an_executed_proposal() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let proposer_address = &setup.owner_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"));
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
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
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
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

        sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
    })
    .assert_ok();

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Executed, sc.get_proposal_status_view(1));
    })
    .assert_ok();
}
