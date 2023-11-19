// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           29
// Async Callback (empty):               1
// Total number of exported functions:  31

#![no_std]

// Configuration that works with rustc < 1.73.0.
// TODO: Recommended rustc version: 1.73.0 or newer.
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    manager
    (
        init => init
        upgrade => upgrade
        addAdmin => add_admin_endpoint
        removeAdmin => remove_admin_endpoint
        forwardToken => forward_token_endpoint
        executeTicket => execute_ticket_endpoint
        createEntity => create_entity_endpoint
        upgradeEntity => upgrade_entity_endpoint
        setFeatures => set_features_endpoint
        setEntityCreationCost => set_entity_creation_cost_endpoint
        setDailyBaseCost => set_daily_base_cost_endpoint
        setDailyFeatureCost => set_daily_feature_cost_endpoint
        getAdmins => admins
        getEntities => entities
        getEntityTemplateAddress => entity_templ_address
        getTrustedHostAddress => trusted_host_address
        getCostTokenId => cost_token_id
        getEntityCreationCost => cost_creation_amount
        getBaseDailyCost => cost_base_daily_amount
        getFeatureDailyCost => cost_feature_daily_amount
        getFeatures => features
        initCreditsModule => init_credits_module
        setCreditsBonusFactor => set_credits_bonus_factor_endpoint
        boost => boost_endpoint
        boostWithSwap => boost_with_swap_endpoint
        registerExternalBoost => register_external_boost_endpoint
        getCredits => get_credits_view
        getCreditsInfo => get_credits_info_view
        initDexModule => init_dex_module
        initOrgModule => init_organization_module
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
