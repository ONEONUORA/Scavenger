#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};
use crate::contract::{ScavengerContract, ScavengerContractClient};
use crate::types::{Role, WasteType};

pub struct TestEnv<'a> {
    pub env: Env,
    pub client: ScavengerContractClient<'a>,
    pub admin: Address,
    pub token: Address,
    pub charity: Address,
}

impl<'a> TestEnv<'a> {
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let token = env.register_stellar_asset_contract(admin.clone());
        let charity = Address::generate(&env);

        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token, &charity, &30, &20);

        Self {
            env,
            client,
            admin,
            token,
            charity,
        }
    }

    pub fn generate_address(&self) -> Address {
        Address::generate(&self.env)
    }

    pub fn create_string(&self, s: &str) -> String {
        String::from_str(&self.env, s)
    }

    pub fn register_participant(&self, address: &Address, role: Role, name: &str) {
        let name_str = self.create_string(name);
        self.client.register_participant(address, &role, &name_str, &1000, &2000);
    }

    pub fn create_incentive(&self, rewarder: &Address, waste_type: WasteType, reward: u64, budget: u64) -> u64 {
        let incentive = self.client.create_incentive(rewarder, &waste_type, &reward, &budget);
        incentive.id
    }

    pub fn fund_contract(&self, amount: i128) {
        let token_client = soroban_sdk::token::StellarAssetClient::new(&self.env, &self.token);
        token_client.mint(&self.client.address, &amount);
    }

    pub fn get_token_client(&self) -> soroban_sdk::token::Client<'a> {
        soroban_sdk::token::Client::new(&self.env, &self.token)
    }
}
