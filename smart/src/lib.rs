#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
pub enum DataKey {
    Admin,
    Count,
    Record(Address, Symbol),
}

#[contract]
pub struct SmartContract;

#[contractimpl]
impl SmartContract {
    /// Inicializa el contrato definiendo al administrador
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Count, &0u32);
    }

    /// Obtiene la dirección del administrador
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Guarda un registro asociado a un usuario y una clave
    pub fn save_record(env: Env, user: Address, key: Symbol, value: Symbol) -> u32 {
        user.require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::Record(user.clone(), key.clone()), &value);

        let mut count: u32 = env.storage().instance().get(&DataKey::Count).unwrap_or(0);
        count += 1;
        env.storage().instance().set(&DataKey::Count, &count);

        count
    }

    /// Lee un registro guardado previamente
    pub fn get_record(env: Env, user: Address, key: Symbol) -> Option<Symbol> {
        env.storage()
            .persistent()
            .get(&DataKey::Record(user, key))
    }

    /// Retorna el contador total de registros
    pub fn get_count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Count).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        let contract_id = env.register(SmartContract, (admin.clone(),));
        let client = SmartContractClient::new(&env, &contract_id);

        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.get_count(), 0);

        let key = symbol_short!("doc1");
        let val = symbol_short!("aprobado");

        let new_count = client.save_record(&user, &key, &val);
        assert_eq!(new_count, 1);
        assert_eq!(client.get_count(), 1);
        assert_eq!(client.get_record(&user, &key), Some(val));
    }
}
