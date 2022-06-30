use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::governance::*;
use entity_esdt::governance::proposal::*;
use entity_esdt::permission::*;
use setup::*;

mod setup;

#[test]
fn it_sets_the_longest_policy_voting_period_for_the_proposal() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);
    let sc_address = setup.contract.address_ref();
    let proposer_address = &setup.user_address;
    let longest_voting_period_minutes: usize = 180;

    setup.blockchain.execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
        sc.create_role(managed_buffer!(b"testrole"));

        sc.create_permission(managed_buffer!(b"testperm1"), managed_address!(sc_address), managed_buffer!(b"testendpoint"));
        sc.create_permission(managed_buffer!(b"testperm2"), managed_address!(sc_address), managed_buffer!(b"testendpoint"));
        sc.create_permission(managed_buffer!(b"testperm3"), managed_address!(sc_address), managed_buffer!(b"testendpoint"));

        sc.create_policy(managed_buffer!(b"testrole"), managed_buffer!(b"testperm1"), PolicyMethod::Weight, managed_biguint!(2u64), 60);
        sc.create_policy(managed_buffer!(b"testrole"), managed_buffer!(b"testperm2"), PolicyMethod::Weight, managed_biguint!(5u64), longest_voting_period_minutes);
        sc.create_policy(managed_buffer!(b"testrole"), managed_buffer!(b"testperm3"), PolicyMethod::Weight, managed_biguint!(8u64), 120);

        sc.assign_role(managed_address!(proposer_address), managed_buffer!(b"testrole"));
    }).assert_ok();

    setup.blockchain.execute_esdt_transfer(&proposer_address, &setup.contract, ENTITY_TOKEN_ID, 0, &rust_biguint!(MIN_WEIGHT_FOR_PROPOSAL), |sc| {
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
        let actions_permissions = MultiValueManagedVec::from(vec![managed_buffer!(b"testperm1"), managed_buffer!(b"testperm2"), managed_buffer!(b"testperm3")]);

        sc.propose_endpoint(managed_buffer!(b"id"), managed_buffer!(b"content hash"), managed_buffer!(b"content signature"), actions_hash, actions_permissions);
    })
    .assert_ok();

    setup.blockchain.execute_query(&setup.contract, |sc| {
        let proposal = sc.proposals(1).get();

        assert_eq!(0, proposal.starts_at);
        assert_eq!(10_800, longest_voting_period_minutes as u64 * 60);
    })
    .assert_ok();
}
