use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::token::StellarAssetContractClient as TokenAdminClient;

fn setup_connect_test_bed(env: &Env) -> (Address, Address, Address, Address, TokenClient, TindahanConnectContractClient) {
    let ofw = Address::generate(env);
    let store_owner = Address::generate(env);
    let distributor = Address::generate(env);
    
    let token_admin_id = env.register_stellar_asset_contract(Address::generate(env));
    let token_admin = TokenAdminClient::new(env, &token_admin_id);
    let token_client = TokenClient::new(env, &token_admin_id);
    
    // Seed OFW wallet with structural digital dollars
    token_admin.mint(&ofw, &4000_i128);

    let contract_id = env.register_contract(None, TindahanConnectContract);
    let contract_client = TindahanConnectContractClient::new(env, &contract_id);

    (ofw, store_owner, distributor, token_admin_id, token_client, contract_client)
}

#[test]
fn test_1_happy_path_successful_inventory_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let (ofw, store_owner, distributor, token_id, token_client, contract) = setup_connect_test_bed(&env);

    contract.setup_order(&ofw, &store_owner, &distributor, &token_id, &500_i128);
    assert_eq!(contract.get_order_status(), OrderStatus::AwaitingFunding);

    contract.fund_order();
    assert_eq!(contract.get_order_status(), OrderStatus::Funded);
    assert_eq!(token_client.balance(&env.current_contract_address()), 500_i128);

    contract.fulfill_delivery();
    assert_eq!(contract.get_order_status(), OrderStatus::Delivered);
    assert_eq!(token_client.balance(&distributor), 500_i128);
    assert_eq!(token_client.balance(&env.current_contract_address()), 0_i128);
}

#[test]
#[should_panic(expected = "Order target funds are not locked inside contract state")]
fn test_2_edge_case_unauthorized_premature_fulfillment_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (ofw, store_owner, distributor, token_id, _, contract) = setup_connect_test_bed(&env);

    contract.setup_order(&ofw, &store_owner, &distributor, &token_id, &300_i128);
    // Distributor attempts to bypass step and call delivery verification prematurely
    contract.fulfill_delivery();
}

#[test]
fn test_3_state_verification_order_cancellation() {
    let env = Env::default();
    env.mock_all_auths();
    let (ofw, store_owner, distributor, token_id, token_client, contract) = setup_connect_test_bed(&env);

    contract.setup_order(&ofw, &store_owner, &distributor, &token_id, &1000_i128);
    contract.fund_order();
    
    contract.cancel_order();
    assert_eq!(contract.get_order_status(), OrderStatus::Cancelled);
    assert_eq!(token_client.balance(&ofw), 4000_i128);
}

#[test]
#[should_panic(expected = "Inventory order structure already initialized")]
fn test_4_edge_case_duplicate_initialization_protection() {
    let env = Env::default();
    env.mock_all_auths();
    let (ofw, store_owner, distributor, token_id, _, contract) = setup_connect_test_bed(&env);

    contract.setup_order(&ofw, &store_owner, &distributor, &token_id, &200_i128);
    // Maliciously executing initialization logic twice to overwrite target accounts
    contract.setup_order(&ofw, &store_owner, &distributor, &token_id, &400_i128);
}

#[test]
#[should_panic(expected = "Order is not in a fundable state")]
fn test_5_edge_case_double_funding_attempt_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (ofw, store_owner, distributor, token_id, _, contract) = setup_connect_test_bed(&env);

    contract.setup_order(&ofw, &store_owner, &distributor, &token_id, &150_i128);
    contract.fund_order();
    // Attempting to fund the same specific order code context twice sequentially
    contract.fund_order();
}
