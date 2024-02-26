use std::vec;

use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use multiversx_sc::types::*;
use multiversx_sc_scenario::*;
use setup::*;

mod setup;

#[test]
fn it_matches_a_permission_based_on_value_only() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"valueOnlyPerm"),
                managed_biguint!(3),
                ManagedAddress::zero(),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"valueOnlyPerm"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(1),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions.clone()));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"valueOnlyPerm")]);

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );

            let proposal = sc.proposals(proposal_id).get();

            let (allowed, permissions) = sc.get_user_permissions_for_actions(&proposal.proposer, &ManagedVec::from(actions), true);

            assert!(allowed);
            assert_eq!(1, permissions.len());
        })
        .assert_ok();
}

#[test]
fn it_matches_a_permission_based_on_destination_only() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"addressOnlyPerm"),
                managed_biguint!(1),
                managed_address!(&action_receiver),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"addressOnlyPerm"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(1),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions.clone()));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"addressOnlyPerm")]);

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );

            let proposal = sc.proposals(proposal_id).get();

            let (allowed, permissions) = sc.get_user_permissions_for_actions(&proposal.proposer, &ManagedVec::from(actions), true);

            assert!(allowed);
            assert_eq!(1, permissions.len());
        })
        .assert_ok();
}

#[test]
fn it_matches_a_permission_based_on_endpoint_only() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"endpointOnlyPerm"),
                managed_biguint!(0),
                ManagedAddress::zero(),
                managed_buffer!(b"someEndpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"endpointOnlyPerm"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"someEndpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions.clone()));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"endpointOnlyPerm")]);

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );

            let proposal = sc.proposals(proposal_id).get();

            let (allowed, permissions) = sc.get_user_permissions_for_actions(&proposal.proposer, &ManagedVec::from(actions), true);

            assert!(allowed);
            assert_eq!(1, permissions.len());
        })
        .assert_ok();
}

#[test]
fn it_matches_a_permission_based_on_arguments_only() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"argumentsOnlyPerm"),
                managed_biguint!(0),
                ManagedAddress::zero(),
                ManagedBuffer::new(),
                ManagedVec::from(vec![managed_buffer!(b"arg1"), managed_buffer!(b"arg2")]),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"argumentsOnlyPerm"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1"), managed_buffer!(b"arg2")]),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions.clone()));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"argumentsOnlyPerm")]);

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );

            let proposal = sc.proposals(proposal_id).get();

            let (allowed, permissions) = sc.get_user_permissions_for_actions(&proposal.proposer, &ManagedVec::from(actions), true);

            assert!(allowed);
            assert_eq!(1, permissions.len());
        })
        .assert_ok();
}

#[test]
fn it_matches_a_permission_based_on_payments_only() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"paymentOnlyPerm"),
                managed_biguint!(0),
                ManagedAddress::zero(),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::from(vec![
                    EsdtTokenPayment::new(managed_token_id!(b"ONE-123456"), 0, managed_biguint!(10)),
                    EsdtTokenPayment::new(managed_token_id!(b"TWO-123456"), 0, managed_biguint!(10)),
                ]),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"paymentOnlyPerm"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(b"ONE-123456"), 0, managed_biguint!(5))]),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions.clone()));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"paymentOnlyPerm")]);

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );

            let proposal = sc.proposals(proposal_id).get();

            let (allowed, permissions) = sc.get_user_permissions_for_actions(&proposal.proposer, &ManagedVec::from(actions), true);

            assert!(allowed);
            assert_eq!(1, permissions.len());
        })
        .assert_ok();
}

/*
 * Mixed matches
 * Test matching multiple parameters at once
 */

#[test]
fn it_matches_a_permission_based_on_destination_and_endpoint() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));
            sc.create_permission(
                managed_buffer!(b"addressAndEndpoint"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                managed_buffer!(b"myendpoint"),
                ManagedVec::new(),
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"addressAndEndpoint"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    // propose
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions.clone()));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"addressAndEndpoint")]);

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );

            let proposal = sc.proposals(proposal_id).get();

            let (allowed, permissions) = sc.get_user_permissions_for_actions(&proposal.proposer, &ManagedVec::from(actions), true);

            assert!(allowed);
            assert_eq!(1, permissions.len());
        })
        .assert_ok();
}

#[test]
fn it_matches_a_permission_based_on_destination_and_endpoint_and_one_argument() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));

            sc.create_permission(
                managed_buffer!(b"addressAndEndpoint"),
                managed_biguint!(0),
                managed_address!(&action_receiver),
                managed_buffer!(b"myendpoint"),
                ManagedVec::from(vec![managed_buffer!(b"arg1")]),
                ManagedVec::new(),
            );

            sc.create_policy(
                managed_buffer!(b"builder"),
                managed_buffer!(b"addressAndEndpoint"),
                PolicyMethod::All,
                BigUint::from(0u64),
                10,
            );
        })
        .assert_ok();

    // propose
    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1"), managed_buffer!(b"arg2")]),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::new(),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions.clone()));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"addressAndEndpoint")]);

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );

            let proposal = sc.proposals(proposal_id).get();

            let (actual, permissions) = sc.get_user_permissions_for_actions(&proposal.proposer, &ManagedVec::from(actions), true);

            assert!(actual);
            assert_eq!(1, permissions.len());
        })
        .assert_ok();
}

#[test]
fn it_does_not_match_one_of_many_payments_that_exceeds_permission_max_amount() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));

            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(0),
                ManagedAddress::zero(),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::from(vec![
                    EsdtTokenPayment::new(managed_token_id!(b"ONE-123456"), 0, managed_biguint!(10)),
                    EsdtTokenPayment::new(managed_token_id!(b"TWO-123456"), 0, managed_biguint!(20)),
                ]),
            );
            sc.create_policy(managed_buffer!(b"builder"), managed_buffer!(b"perm"), PolicyMethod::All, BigUint::from(0u64), 10);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![
                    EsdtTokenPayment::new(managed_token_id!(b"ONE-123456"), 0, managed_biguint!(15)), // exceeds permission
                    EsdtTokenPayment::new(managed_token_id!(b"TWO-123456"), 0, managed_biguint!(20)),
                ]),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions.clone()));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );

            let proposal = sc.proposals(proposal_id).get();

            let (allowed, permissions) = sc.get_user_permissions_for_actions(&proposal.proposer, &ManagedVec::from(actions), true);

            assert!(!allowed);
            assert_eq!(0, permissions.len());
        })
        .assert_ok();
}

#[test]
fn it_does_not_match_a_payment_when_there_is_no_permission_for_it() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    // configure permissions
    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.create_role(managed_buffer!(b"builder"));
            sc.assign_role(managed_address!(&proposer_address), managed_buffer!(b"builder"));

            sc.create_permission(
                managed_buffer!(b"perm"),
                managed_biguint!(0),
                ManagedAddress::zero(),
                ManagedBuffer::new(),
                ManagedVec::new(),
                ManagedVec::from(vec![
                    // ONE token payment is not declared but trying to spend it in below action
                    EsdtTokenPayment::new(managed_token_id!(b"TWO-123456"), 0, managed_biguint!(20)),
                ]),
            );
            sc.create_policy(managed_buffer!(b"builder"), managed_buffer!(b"perm"), PolicyMethod::All, BigUint::from(0u64), 10);
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_GOV_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();
            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                endpoint: ManagedBuffer::new(),
                arguments: ManagedVec::new(),
                gas_limit: 5_000_000u64,
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![
                    EsdtTokenPayment::new(managed_token_id!(b"ONE-123456"), 0, managed_biguint!(1)), // no permission for this payment
                    EsdtTokenPayment::new(managed_token_id!(b"TWO-123456"), 0, managed_biguint!(20)),
                ]),
            });

            let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions.clone()));
            let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"perm")]);

            let proposal_id = sc.propose_endpoint(
                managed_buffer!(b"id"),
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                actions_hash,
                POLL_DEFAULT_ID,
                actions_permissions,
            );

            let proposal = sc.proposals(proposal_id).get();

            let (allowed, permissions) = sc.get_user_permissions_for_actions(&proposal.proposer, &ManagedVec::from(actions), true);

            assert!(!allowed);
            assert_eq!(0, permissions.len());
        })
        .assert_ok();
}
