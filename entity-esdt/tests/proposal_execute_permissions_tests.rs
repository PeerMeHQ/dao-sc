use elrond_wasm::{types::*};
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::proposal::*;
use entity_esdt::governance::*;
use entity_esdt::permission::*;
use setup::*;

mod setup;

#[test]
fn it_executes_a_proposal_with_truthfully_announced_permissions() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let proposer_address = &setup.user_address;
    let executor_address = setup.blockchain.create_user_account(&rust_biguint!(5));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));

        sc.create_permission(managed_buffer!(b"announced1"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint1"));
        sc.create_permission(managed_buffer!(b"announced2"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint2"));

        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"announced1"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"announced2"),  PolicyMethod::Weight, BigUint::from(1u64), 12);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();
        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint1"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });
        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint2"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"announced1"), managed_buffer!(b"announced2")]);

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_tx(&executor_address, &setup.contract, &rust_biguint!(0), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();
        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint1"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });
        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint2"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));

        assert!(sc.proposals(proposal_id).get().was_executed);
    })
    .assert_ok();
}

#[test]
fn it_fails_to_executes_a_proposal_with_untruthfully_announced_permissions() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let proposer_address = &setup.user_address;
    let executor_address = setup.blockchain.create_user_account(&rust_biguint!(5));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"announced"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint1"));
        sc.create_permission(managed_buffer!(b"unannounced"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint2"));

        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"announced"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"unannounced"),  PolicyMethod::Weight, BigUint::from(1u64), 12);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();
        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint1"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });
        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint2"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"announced")]); // not announcing the 'unannounced' permission

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_tx(&executor_address, &setup.contract, &rust_biguint!(0), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();
        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint1"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });
        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint2"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));

        assert!(!sc.proposals(proposal_id).get().was_executed);
    })
    .assert_user_error("untruthful permissions announced");
}
