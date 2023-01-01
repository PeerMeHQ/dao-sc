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
fn it_sends_egld() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    // set available balance to 5
    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(5));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(3),
                managed_address!(&action_receiver),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::new(),
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

    // send 3 egld to action receiver
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                value: managed_biguint!(3),
                payments: ManagedVec::new(),
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
                value: managed_biguint!(3),
                payments: ManagedVec::new(),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    // assert action receiver balance
    setup.blockchain.check_egld_balance(&action_receiver, &rust_biguint!(3));
}

#[test]
fn it_sends_egld_to_multiple_recipients() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = setup.user_address.clone();
    let action_receiver_one = setup.blockchain.create_user_account(&rust_biguint!(0));
    let action_receiver_two = setup.blockchain.create_user_account(&rust_biguint!(0));
    let action_receiver_three = setup.blockchain.create_user_account(&rust_biguint!(0));
    let mut proposal_id = 0;

    setup.configure_gov_token(true);

    // set available balance to 10
    setup.blockchain.set_egld_balance(setup.contract.address_ref(), &rust_biguint!(10));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(ROLE_BUILTIN_LEADER));
            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(3),
                ManagedAddress::zero(),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::new(),
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

    // send 3 egld to all action receivers
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver_one),
                gas_limit: 5_000_000u64,
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                value: managed_biguint!(3),
                payments: ManagedVec::new(),
            });

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver_two),
                gas_limit: 5_000_000u64,
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                value: managed_biguint!(3),
                payments: ManagedVec::new(),
            });

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver_three),
                gas_limit: 5_000_000u64,
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                value: managed_biguint!(3),
                payments: ManagedVec::new(),
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
                destination: managed_address!(&action_receiver_one),
                gas_limit: 5_000_000u64,
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                value: managed_biguint!(3),
                payments: ManagedVec::new(),
            });

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver_two),
                gas_limit: 5_000_000u64,
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                value: managed_biguint!(3),
                payments: ManagedVec::new(),
            });

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver_three),
                gas_limit: 5_000_000u64,
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                value: managed_biguint!(3),
                payments: ManagedVec::new(),
            });

            sc.execute_endpoint(proposal_id, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    // assert action receiver balances
    setup.blockchain.check_egld_balance(&action_receiver_one, &rust_biguint!(3));
    setup.blockchain.check_egld_balance(&action_receiver_two, &rust_biguint!(3));
    setup.blockchain.check_egld_balance(&action_receiver_three, &rust_biguint!(3));
}
