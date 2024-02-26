use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use multiversx_sc::codec::multi_types::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_executes_actions_when_proposed_by_non_leader() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let board_member_one = setup.blockchain.create_user_account(&rust_biguint!(0));
    let board_member_two = setup.blockchain.create_user_account(&rust_biguint!(0));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let board_role = b"board";

    setup.configure_gov_token(true);

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1000));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&board_member_one), managed_buffer!(board_role));
            sc.assign_role(managed_address!(&board_member_two), managed_buffer!(board_role));
            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(5),
                managed_address!(&action_receiver),
                managed_buffer!(b""),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(managed_buffer!(board_role), managed_buffer!(b"perm"), PolicyMethod::Majority, BigUint::from(0u64), 10);
        })
        .assert_ok();

    // propose
    setup
        .blockchain
        .execute_tx(&board_member_one, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(5),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

            sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"a"),
                managed_buffer!(b"b"),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    // sign by member
    setup
        .blockchain
        .execute_tx(&board_member_two, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sign_endpoint(1, OptionalValue::None);
        })
        .assert_ok();

    // execute
    setup
        .blockchain
        .execute_tx(&board_member_two, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(5),
                payments: ManagedVec::new(),
            });

            sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    setup.blockchain.check_egld_balance(&action_receiver, &rust_biguint!(5));
}

#[test]
fn it_executes_actions_with_advanced_permissions_when_proposed_by_non_leader() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let board_member_one = setup.blockchain.create_user_account(&rust_biguint!(0));
    let board_member_two = setup.blockchain.create_user_account(&rust_biguint!(0));
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let board_role = b"board";

    setup.configure_gov_token(true);

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(1000));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&board_member_one), managed_buffer!(board_role));
            sc.assign_role(managed_address!(&board_member_two), managed_buffer!(board_role));
            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                managed_buffer!(b"addCategory"),
                ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                ManagedVec::new(),
            );
            sc.create_policy(managed_buffer!(board_role), managed_buffer!(b"perm"), PolicyMethod::Majority, BigUint::from(0u64), 10);
        })
        .assert_ok();

    // propose
    setup
        .blockchain
        .execute_tx(&board_member_one, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"addCategory"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(5),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

            sc.propose_endpoint(
                managed_buffer!(b"id"),
                managed_buffer!(b"a"),
                managed_buffer!(b"b"),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    // sign by member
    setup
        .blockchain
        .execute_tx(&board_member_two, &setup.contract, &rust_biguint!(0), |sc| {
            sc.sign_endpoint(1, OptionalValue::None);
        })
        .assert_ok();

    // execute
    setup
        .blockchain
        .execute_tx(&board_member_two, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"addCategory"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(5),
                payments: ManagedVec::new(),
            });

            sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    setup.blockchain.check_egld_balance(&action_receiver, &rust_biguint!(5));
}

#[test]
fn it_fails_when_a_required_argument_is_missing() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let executor_address = setup.blockchain.create_user_account(&rust_biguint!(5));
    let mut proposal_id = 0;

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));

            sc.create_permission(
                managed_buffer!(b"callSc"),
                managed_biguint!(10),
                managed_address!(&action_receiver),
                ManagedBuffer::new(),
                ManagedVec::from(vec![managed_buffer!(b"testarg1"), managed_buffer!(b"testarg2")]),
                ManagedVec::new(),
            );

            sc.create_policy(managed_buffer!(b"builder"), managed_buffer!(b"callSc"), PolicyMethod::Quorum, BigUint::from(1u64), 1);
        })
        .assert_ok();

    // propose
    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::from(vec![managed_buffer!(b"testarg1")]),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"callSc")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    // execute the proposal by someone else (not proposer)
    setup
        .blockchain
        .execute_tx(&executor_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::from(vec![managed_buffer!(b"testarg1")]),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_user_error("no permission for action");
}

#[test]
fn it_fails_when_payment_value_is_higher_than_defined_by_permission() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let executor_address = setup.blockchain.create_user_account(&rust_biguint!(5));
    let mut proposal_id = 0;

    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(100));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"sendEGLD"),
                managed_biguint!(10),
                managed_address!(&action_receiver),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"sendEGLD"),
                PolicyMethod::Quorum,
                BigUint::from(1u64),
                1,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(11),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"sendEGLD")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup
        .blockchain
        .execute_tx(&executor_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(11),
                payments: ManagedVec::new(),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_user_error("no permission for action");
}

#[test]
fn it_fails_when_token_payment_amount_is_higher_than_defined_by_permission() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let executor_address = setup.blockchain.create_user_account(&rust_biguint!(5));
    let mut proposal_id = 0;

    setup.blockchain.set_esdt_balance(setup.contract.address_ref(), b"SUPER-123456", &rust_biguint!(100));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));

            sc.create_permission(
                managed_buffer!(b"sendSuper"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(b"SUPER-123456"), 0, managed_biguint!(10))]),
            );

            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"sendSuper"),
                PolicyMethod::Quorum,
                BigUint::from(1u64),
                1,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(b"SUPER-123456"), 0, managed_biguint!(11))]),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"sendSuper")]);

            proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_tx(&executor_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(b"SUPER-123456"), 0, managed_biguint!(11))]),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_user_error("no permission for action");
}
