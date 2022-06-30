elrond_wasm::imports!();

use elrond_wasm_debug::testing_framework::*;
use elrond_wasm_debug::*;
use entity_esdt::config::*;
use entity_esdt::*;

pub const ENTITY_TOKEN_ID: &[u8] = b"SUPER-abcdef";
pub const ENTITY_TOKEN_SUPPLY: u64 = 1_000;
pub const VOTE_NFT_TOKEN_ID: &[u8] = b"SUPERVOTE-abcdef";
pub const ENTITY_FAKE_TOKEN_ID: &[u8] = b"FAKE-abcdef";
pub const MIN_WEIGHT_FOR_PROPOSAL: u64 = 2;
pub const QURUM: u64 = 50;
pub const WASM_PATH: &'static str = "output/entity.wasm";

#[allow(dead_code)]
pub struct EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> entity_esdt::ContractObj<DebugApi>,
{
    pub blockchain: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_address: Address,
    pub trusted_host_address: Address,
    pub contract: ContractObjWrapper<entity_esdt::ContractObj<DebugApi>, ObjBuilder>,
}

impl<ObjBuilder> EntitySetup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> entity_esdt::ContractObj<DebugApi>,
{
    pub fn new(builder: ObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut blockchain = BlockchainStateWrapper::new();
        let owner_address = blockchain.create_user_account(&rust_zero);
        let user_address = blockchain.create_user_account(&rust_biguint!(1000));
        let trusted_host_address = blockchain.create_user_account(&rust_zero);
        let contract = blockchain.create_sc_account(&rust_biguint!(100), Some(&owner_address), builder, WASM_PATH);

        blockchain.set_esdt_balance(&owner_address, ENTITY_TOKEN_ID, &rust_biguint!(ENTITY_TOKEN_SUPPLY));
        blockchain.set_esdt_balance(&user_address, ENTITY_TOKEN_ID, &rust_biguint!(ENTITY_TOKEN_SUPPLY));
        blockchain.set_esdt_balance(&user_address, ENTITY_FAKE_TOKEN_ID, &rust_biguint!(ENTITY_TOKEN_SUPPLY));

        blockchain
            .execute_tx(&owner_address, &contract, &rust_zero, |sc| {
                sc.init(
                    managed_address!(&owner_address),
                    OptionalValue::Some(managed_token_id!(ENTITY_TOKEN_ID)),
                    OptionalValue::Some(managed_biguint!(ENTITY_TOKEN_SUPPLY)),
                    OptionalValue::None,
                );

                sc.quorum().set(managed_biguint!(QURUM));
                sc.min_proposal_vote_weight().set(managed_biguint!(MIN_WEIGHT_FOR_PROPOSAL));
                sc.voting_period_in_minutes().set(VOTING_PERIOD_MINUTES_DEFAULT);
                sc.vote_nft_token().set_token_id(&managed_token_id!(VOTE_NFT_TOKEN_ID));
            })
            .assert_ok();

        let vote_nft_roles = [EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn, EsdtLocalRole::NftUpdateAttributes];
        blockchain.set_esdt_local_roles(contract.address_ref(), VOTE_NFT_TOKEN_ID, &vote_nft_roles[..]);

        Self {
            blockchain,
            owner_address,
            user_address,
            trusted_host_address,
            contract,
        }
    }
}

#[test]
fn it_initializes_the_contract() {
    let mut setup = EntitySetup::new(entity_esdt::contract_obj);

    setup
        .blockchain
        .execute_query(&setup.contract, |sc| {
            assert_eq!(managed_token_id!(ENTITY_TOKEN_ID), sc.token().get_token_id());
        })
        .assert_ok();
}
