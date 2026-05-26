#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, token};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderStatus {
    AwaitingFunding = 0,
    Funded = 1,
    Delivered = 2,
    Cancelled = 3,
}

#[contracttype]
pub enum StorageKey {
    OfwSender,
    StoreOwner,
    Distributor,
    TokenAddress,
    OrderCost,
    Status,
}

#[contract]
pub struct TindahanConnectContract;

#[contractimpl]
impl TindahanConnectContract {
    /// Initializes a secure supply-chain inventory allocation connection mapping.
    pub fn setup_order(
        env: Env,
        ofw: Address,
        store_owner: Address,
        distributor: Address,
        token_addr: Address,
        cost: i128,
    ) {
        if env.storage().instance().has(&StorageKey::Status) {
            panic!("Inventory order structure already initialized");
        }
        if cost <= 0 {
            panic!("Order inventory cost must be positive");
        }

        env.storage().instance().set(&StorageKey::OfwSender, &ofw);
        env.storage().instance().set(&StorageKey::StoreOwner, &store_owner);
        env.storage().instance().set(&StorageKey::Distributor, &distributor);
        env.storage().instance().set(&StorageKey::TokenAddress, &token_addr);
        env.storage().instance().set(&StorageKey::OrderCost, &cost);
        env.storage().instance().set(&StorageKey::Status, &OrderStatus::AwaitingFunding);
    }

    /// Deposits the required digital dollars from the OFW account to secure the physical stock delivery.
    pub fn fund_order(env: Env) {
        let current_status: OrderStatus = env.storage().instance().get(&StorageKey::Status).unwrap();
        if current_status != OrderStatus::AwaitingFunding {
            panic!("Order is not in a fundable state");
        }

        let ofw: Address = env.storage().instance().get(&StorageKey::OfwSender).unwrap();
        ofw.require_auth();

        let token_addr: Address = env.storage().instance().get(&StorageKey::TokenAddress).unwrap();
        let cost: i128 = env.storage().instance().get(&StorageKey::OrderCost).unwrap();
        
        let token_client = token::Client::new(&env, &token_addr);
        
        // Lock inventory allocation budget directly into the smart contract custody layer
        token_client.transfer(&ofw, &env.current_contract_address(), &cost);

        env.storage().instance().set(&StorageKey::Status, &OrderStatus::Funded);
    }

    /// Releases payment to the distributor after physical wholesale verification at the store.
    pub fn fulfill_delivery(env: Env) {
        let current_status: OrderStatus = env.storage().instance().get(&StorageKey::Status).unwrap();
        if current_status != OrderStatus::Funded {
            panic!("Order target funds are not locked inside contract state");
        }

        let distributor: Address = env.storage().instance().get(&StorageKey::Distributor).unwrap();
        distributor.require_auth();

        let token_addr: Address = env.storage().instance().get(&StorageKey::TokenAddress).unwrap();
        let cost: i128 = env.storage().instance().get(&StorageKey::OrderCost).unwrap();

        let token_client = token::Client::new(&env, &token_addr);
        
        // Route payment directly to supplier following physical confirmation
        token_client.transfer(&env.current_contract_address(), &distributor, &cost);

        env.storage().instance().set(&StorageKey::Status, &OrderStatus::Delivered);
    }

    /// Allows the funding OFW to pull back capital if the inventory logistics line stalls.
    pub fn cancel_order(env: Env) {
        let ofw: Address = env.storage().instance().get(&StorageKey::OfwSender).unwrap();
        ofw.require_auth();

        let current_status: OrderStatus = env.storage().instance().get(&StorageKey::Status).unwrap();
        if current_status != OrderStatus::Funded {
            panic!("No active locked funds available to cancel");
        }

        let token_addr: Address = env.storage().instance().get(&StorageKey::TokenAddress).unwrap();
        let cost: i128 = env.storage().instance().get(&StorageKey::OrderCost).unwrap();

        let token_client = token::Client::new(&env, &token_addr);
        
        // Return capital securely back to the OFW source account
        token_client.transfer(&env.current_contract_address(), &ofw, &cost);

        env.storage().instance().set(&StorageKey::Status, &OrderStatus::Cancelled);
    }

    /// Reads the active physical lifecycle tracking state of the inventory link.
    pub fn get_order_status(env: Env) -> OrderStatus {
        env.storage().instance().get(&StorageKey::Status).unwrap_or(OrderStatus::AwaitingFunding)
    }
}

#[cfg(test)]
mod tests;
