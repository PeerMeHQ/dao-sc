use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::proposal::*;
use entity_esdt::governance::*;
use entity_esdt::permission::*;
use setup::*;

mod setup;

#[test]
fn it_returns_active_when_just_created() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let sc_address = setup.contract.address_ref();
    let proposer_address = &setup.user_address;
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role(managed_buffer!(b"testrole"));
        sc.create_permission(managed_buffer!(b"testperm"), managed_address!(sc_address), managed_buffer!(b"testendpoint"));
        sc.create_policy(managed_buffer!(b"testrole"), managed_buffer!(b"testperm"), PolicyMethod::Quorum, managed_biguint!(3), VOTING_PERIOD_MINUTES_DEFAULT);
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(b"testrole"));
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();
        actions.push(Action::<DebugApi> {
            destination: managed_address!(sc_address),
            endpoint: managed_buffer!(b"testendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm")]);

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    }).assert_ok();

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Active, sc.get_proposal_status_view(proposal_id));
    }).assert_ok();
}

#[test]
fn it_succeeds_if_one_of_one_permission_policies_reaches_signer_quorum() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let sc_address = setup.contract.address_ref();
    let proposer_address = &setup.user_address;
    let signer_one = setup.blockchain.create_user_account(&rust_biguint!(1));
    let signer_two = setup.blockchain.create_user_account(&rust_biguint!(1));
    let mut proposal_id = 0;
    let quorum = 3;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role(managed_buffer!(b"testrole"));
        sc.create_permission(managed_buffer!(b"testperm"), managed_address!(sc_address), managed_buffer!(b"testendpoint"));
        sc.create_policy(managed_buffer!(b"testrole"), managed_buffer!(b"testperm"), PolicyMethod::Quorum, managed_biguint!(quorum), VOTING_PERIOD_MINUTES_DEFAULT);
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(b"testrole"));
        sc.assign_role(managed_address!(&signer_one), managed_buffer!(b"testrole"));
        sc.assign_role(managed_address!(&signer_two), managed_buffer!(b"testrole"));
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();
        actions.push(Action::<DebugApi> {
            destination: managed_address!(sc_address),
            endpoint: managed_buffer!(b"testendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm")]);

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    }).assert_ok();

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);
    }).assert_ok();

    setup.blockchain.execute_tx(&signer_one, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);
    }).assert_ok();

    setup.blockchain.execute_tx(&signer_two, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);
    }).assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Succeeded, sc.get_proposal_status_view(1));
    }).assert_ok();
}

#[test]
fn it_returns_defeated_if_one_of_one_permission_policies_does_not_meet_quorum_after_voting_period_ended() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let sc_address = setup.contract.address_ref();
    let proposer_address = &setup.user_address;
    let mut proposal_id = 0;
    let quorum = 3;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role(managed_buffer!(b"testrole"));
        sc.create_permission(managed_buffer!(b"testperm"), managed_address!(sc_address), managed_buffer!(b"testendpoint"));
        sc.create_policy(managed_buffer!(b"testrole"), managed_buffer!(b"testperm"), PolicyMethod::Quorum, managed_biguint!(quorum), VOTING_PERIOD_MINUTES_DEFAULT);
        sc.assign_role(managed_address!(proposer_address), managed_buffer!(b"testrole"));
    }).assert_ok();

    // not reaching policy quorum
    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();
        actions.push(Action::<DebugApi> {
            destination: managed_address!(sc_address),
            endpoint: managed_buffer!(b"testendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm")]);

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    }).assert_ok();

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);
    }).assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(1));
    }).assert_ok();
}

#[test]
fn it_returns_defeated_if_one_of_two_permission_policies_does_not_meet_quorum_after_voting_period_ended() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let sc_address = setup.contract.address_ref();
    let proposer_address = &setup.user_address;
    let mut proposal_id = 0;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role(managed_buffer!(b"testrole"));

        sc.create_permission(managed_buffer!(b"testperm1"), managed_address!(sc_address), managed_buffer!(b"testendpoint"));
        sc.create_permission(managed_buffer!(b"testperm2"), managed_address!(sc_address), managed_buffer!(b"testendpoint"));

        sc.create_policy(managed_buffer!(b"testrole"), managed_buffer!(b"testperm1"), PolicyMethod::Quorum, managed_biguint!(3), VOTING_PERIOD_MINUTES_DEFAULT);
        sc.create_policy(managed_buffer!(b"testrole"), managed_buffer!(b"testperm2"), PolicyMethod::Weight, managed_biguint!(QURUM), VOTING_PERIOD_MINUTES_DEFAULT);

        sc.assign_role(managed_address!(proposer_address), managed_buffer!(b"testrole"));
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(QURUM + 1), |sc| {
        let mut actions = Vec::<Action<DebugApi>>::new();
        actions.push(Action::<DebugApi> {
            destination: managed_address!(sc_address),
            endpoint: managed_buffer!(b"testendpoint"),
            arguments: ManagedVec::new(),
            gas_limit: 5_000_000u64,
            token_id: managed_egld_token_id!(),
            token_nonce: 0,
            amount: managed_biguint!(0),
        });

        let actions_hash = sc.calculate_actions_hash(&ManagedVec::from(actions));
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm1"), managed_buffer!(b"testperm2")]);

        proposal_id = sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b""), managed_buffer!(b""), actions_hash, actions_permissions);
    }).assert_ok();

    setup.blockchain.execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.sign_endpoint(proposal_id);
    }).assert_ok();

    setup.blockchain.set_block_timestamp(VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60 + 1);

    setup.blockchain.execute_query(&setup.contract, |sc| {
        assert_eq!(ProposalStatus::Defeated, sc.get_proposal_status_view(proposal_id));
    }).assert_ok();
}
