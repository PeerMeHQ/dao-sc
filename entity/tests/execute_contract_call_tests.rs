use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::config::*;
use entity::governance::proposal::*;
use entity::governance::*;
use entity::permission::*;
use setup::*;

mod setup;

#[test]
fn it_calls_a_contract() {
    let mut setup = EntitySetup::new(entity::contract_obj);
    let voting_period_seconds = VOTING_PERIOD_MINUTES_DEFAULT as u64 * 60;
    let proposer_address = setup.user_address.clone();
    let action_receiver = setup.blockchain.create_user_account(&rust_biguint!(0));

    setup.configure_gov_token(true);

    setup
        .blockchain
        .set_esdt_balance(setup.contract.address_ref(), b"ACTION-123456", &rust_biguint!(1000));

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
                ManagedVec::new(),
            );
            sc.create_policy(
                managed_buffer!(ROLE_BUILTIN_LEADER),
                managed_buffer!(b"perm"),
                PolicyMethod::Quorum,
                BigUint::from(1u64),
                10,
            );
        })
        .assert_ok();

    setup
        .blockchain
        .execute_esdt_transfer(
            &proposer_address,
            &setup.contract,
            ENTITY_GOV_TOKEN_ID,
            0,
            &rust_biguint!(ENTITY_GOV_TOKEN_SUPPLY),
            |sc| {
                let mut actions = Vec::<Action<DebugApi>>::new();

                actions.push(Action::<DebugApi> {
                    destination: managed_address!(&action_receiver),
                    gas_limit: 5_000_000u64,
                    endpoint: managed_buffer!(b"myendpoint"),
                    arguments: ManagedVec::from(vec![managed_buffer!(b"arg1"), managed_buffer!(b"arg2")]),
                    value: managed_biguint!(0),
                    payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(b"ACTION-123456"), 0, managed_biguint!(5))]),
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
            },
        )
        .assert_ok();

    setup.blockchain.set_block_timestamp(voting_period_seconds + 1);

    setup
        .blockchain
        .execute_tx(&proposer_address, &setup.contract, &rust_biguint!(0), |sc| {
            let mut actions = Vec::<Action<DebugApi>>::new();

            actions.push(Action::<DebugApi> {
                destination: managed_address!(&action_receiver),
                gas_limit: 5_000_000u64,
                endpoint: managed_buffer!(b"myendpoint"),
                arguments: ManagedVec::from(vec![managed_buffer!(b"arg1"), managed_buffer!(b"arg2")]),
                value: managed_biguint!(0),
                payments: ManagedVec::from(vec![EsdtTokenPayment::new(managed_token_id!(b"ACTION-123456"), 0, managed_biguint!(5))]),
            });

            sc.execute_endpoint(1, MultiValueManagedVec::from(actions));
        })
        .assert_ok();

    setup.blockchain.check_esdt_balance(&action_receiver, b"ACTION-123456", &rust_biguint!(5));
}
