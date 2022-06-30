use elrond_wasm::{types::*};
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::proposal::*;
use entity_esdt::governance::*;
use entity_esdt::permission::*;
use setup::*;

mod setup;

#[test]
fn it_marks_a_proposal_as_executed() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let proposer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"));
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
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

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

        assert!(sc.proposals(proposal_id).get().was_executed);
    })
    .assert_ok();
}

#[test]
fn it_fails_if_attempted_to_execute_again() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"));
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
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

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

        sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions)); // and again
    })
    .assert_user_error("proposal is not executable");
}

#[test]
fn it_fails_if_the_proposal_is_still_active() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let proposer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"));
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
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

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
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

        sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions)); // and again
    })
    .assert_user_error("proposal is not executable");
}


#[test]
fn it_executes_actions_of_a_proposal() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1000));

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"));
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(ENTITY_TOKEN_SUPPLY), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(5),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

        sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b"a"), managed_buffer!(b"b"), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(5),
        });

        sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
    })
    .assert_ok();

    setup.blockchain.check_egld_balance(&action_receiver, &rust_biguint!(5));
}

#[test]
fn it_fails_if_actions_to_execute_are_incongruent_to_actions_proposed() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1000));

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"));
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(ENTITY_TOKEN_SUPPLY), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(5),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

        sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b"a"), managed_buffer!(b"b"), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            endpoint: managed_buffer!(b"yourendpoint"), // has changed from myendpoint to yourendpoint -> fail
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(5),
        });

        sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
    })
    .assert_user_error("actions have been corrupted");
}

#[test]
fn it_executes_a_contract_call_action() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.blockchain.set_esdt_balance(setup.contract.address_ref(), b"ACTION-123456", &rust_biguint!(1000));

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"));
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(ENTITY_TOKEN_SUPPLY), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            token_id: managed_token_id_wrapped!(b"ACTION-123456"),
            token_nonce: 0,
            amount: managed_biguint!(5),
            gas_limit: 5_000_000u64,
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::from(vec![managed_buffer!(b"arg1"), managed_buffer!(b"arg2")]),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

        sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b"a"), managed_buffer!(b"b"), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            token_id: managed_token_id_wrapped!(b"ACTION-123456"),
            token_nonce: 0,
            amount: managed_biguint!(5),
            gas_limit: 5_000_000u64,
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::from(vec![managed_buffer!(b"arg1"), managed_buffer!(b"arg2")]),
        });

        sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
    })
    .assert_ok();

    setup.blockchain.check_esdt_balance(&action_receiver, b"ACTION-123456", &rust_biguint!(5));
}

#[test]
fn it_fails_to_spend_vote_tokens() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = &setup.user_address;
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    // set available balance to 5
    setup.blockchain.set_esdt_balance(setup.contract.address_ref(), ENTITY_TOKEN_ID, &rust_biguint!(5));

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
        sc.create_permission(managed_buffer!(b"perm"), managed_address!(&action_receiver), managed_buffer!(b"myendpoint"));
        sc.create_policy(managed_buffer!(ROLE_BUILTIN_LEADER), managed_buffer!(b"perm"), PolicyMethod::Quorum, BigUint::from(1u64), 10);
    }).assert_ok();

    // but try to spend 6 with a proposal action
    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            token_id: managed_token_id_wrapped!(ENTITY_TOKEN_ID),
            token_nonce: 0,
            amount: managed_biguint!(6),
            gas_limit: 5_000_000u64,
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b"a"), managed_buffer!(b"b"), actions_hash, actions_permissions);
    })
    .assert_ok();

    // add to the sc token balance: vote for with 100 tokens
    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(100), |sc| {
            sc.vote_for_endpoint(proposal_id);
        })
        .assert_ok();

    // add to the sc token balance: vote against with 100 tokens
    setup.blockchain.execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(20), |sc| {
            sc.vote_against_endpoint(proposal_id);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    // but it should FAIL because vote tokens should NOT be spendable
    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();

        actions.push(Action::<DebugApi> {
            destination: managed_address!(&action_receiver),
            token_id: managed_token_id_wrapped!(ENTITY_TOKEN_ID),
            token_nonce: 0,
            amount: managed_biguint!(6),
            gas_limit: 5_000_000u64,
            endpoint: managed_buffer!(b"myendpoint"),
            arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
        });

        sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
    })
    .assert_user_error("not enough governance tokens available");
}
