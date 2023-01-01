use elrond_wasm::elrond_codec::multi_types::*;
use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::errors::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_sends_esdt_tokens() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    // set available balance to 5
    setup
        .blockchain
        .set_esdt_balance(setup.contract.address_ref(), b"TOKEN-123456", &rust_biguint!(5));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(b"TOKEN-123456"), 0, managed_biguint!(10))]),
            );
            sc.create_policy(
                managed_buffer!(ROLE_BUILTIN_LEADER),
                managed_buffer!(b"perm"),
                PolicyMethod::Weight,
                BigUint::from(1u64),
                10,
            );
        })
        .assert_ok();

    // send 3 tokens to action receiver
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(b"TOKEN-123456"), 0, managed_biguint!(3))]),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"a"),
                managed_buffer!(b"b"),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    // execute proposal
    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(b"TOKEN-123456"), 0, managed_biguint!(3))]),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    // assert action receiver balance
    setup.blockchain.check_esdt_balance(&action_receiver, b"TOKEN-123456", &rust_biguint!(3));
}

#[test]
fn it_fails_to_spend_esdt_vote_tokens() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    // set available balance to 5
    setup
        .blockchain
        .set_esdt_balance(setup.contract.address_ref(), ENTITY_GOV_TOKEN_ID, &rust_biguint!(5));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                managed_buffer!(b"myendpoint"),
                ManagedVec::new(),
                ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(ENTITY_GOV_TOKEN_ID), 0, managed_biguint!(10))]),
            );
            sc.create_policy(
                managed_buffer!(ROLE_BUILTIN_LEADER),
                managed_buffer!(b"perm"),
                PolicyMethod::Weight,
                BigUint::from(1u64),
                10,
            );
        })
        .assert_ok();

    // but try to spend 6 with a proposal action
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(ENTITY_GOV_TOKEN_ID), 0, managed_biguint!(6))]),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"a"),
                managed_buffer!(b"b"),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    // add to the sc token balance: vote for with 100 tokens
    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(100), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    // add to the sc token balance: vote against with 100 tokens
    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(20), |sc| {
            sc.vote_against_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    // but it should FAIL because vote tokens should NOT be spendable
    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(ENTITY_GOV_TOKEN_ID), 0, managed_biguint!(6))]),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_user_error(&String::from_utf8(NOT_ENOUGH_GOV_TOKENS_AVAILABLE.to_vec()).unwrap());
}

#[test]
fn it_fails_to_spend_sft_vote_tokens() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    setup
        .blockchain
        .set_nft_balance(&setup.owner_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(500), &0);
    setup.blockchain.set_nft_balance(&proposer_address, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(500), &0);

    // set available balance to 5
    setup
        .blockchain
        .set_nft_balance(setup.contract.address_ref(), ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(5), &0);

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                managed_buffer!(b"myendpoint"),
                ManagedVec::new(),
                ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(ENTITY_GOV_TOKEN_ID), 1, managed_biguint!(10))]),
            );
            sc.create_policy(
                managed_buffer!(ROLE_BUILTIN_LEADER),
                managed_buffer!(b"perm"),
                PolicyMethod::Weight,
                BigUint::from(1u64),
                10,
            );
        })
        .assert_ok();

    // but try to spend 6 with a proposal action
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(ENTITY_GOV_TOKEN_ID), 1, managed_biguint!(6))]),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"a"),
                managed_buffer!(b"b"),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&setup.owner_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 1, &rust_biguint!(100), |sc| {
            sc.vote_for_endpoint(proposal_id, OptionalValue::None);
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    // but it should FAIL because vote tokens should NOT be spendable
    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(ENTITY_GOV_TOKEN_ID), 1, managed_biguint!(6))]),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_user_error(&String::from_utf8(NOT_ENOUGH_GOV_TOKENS_AVAILABLE.to_vec()).unwrap());
}
