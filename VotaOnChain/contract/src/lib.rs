#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, Map, Symbol, Vec,
};

#[contracttype]
pub enum DataKey {
    Admin,
    Votes,
    Options,
    HasVoted(Address),
}

#[contract]
pub struct VotingContract;

#[contractimpl]
impl VotingContract {
    /// Inicializa el contrato con el admin y las opciones de votación.
    /// Corre una sola vez, atómicamente, al desplegar — no se puede volver a
    /// invocar (a diferencia de un `init()` manual, cierra la ventana de
    /// front-running entre el deploy y la inicialización).
    pub fn __constructor(env: Env, admin: Address, options: Vec<Symbol>) {
        admin.require_auth();

        let votes: Map<Symbol, u32> = Map::new(&env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Votes, &votes);
        env.storage().persistent().set(&DataKey::Options, &options);
    }

    /// Retorna la dirección del admin del contrato.
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Registrar un voto. Retorna false si la wallet ya votó, true si el voto fue exitoso.
    pub fn vote(env: Env, voter: Address, option: Symbol) -> bool {
        voter.require_auth();

        let has_voted: bool = env
            .storage()
            .persistent()
            .get(&DataKey::HasVoted(voter.clone()))
            .unwrap_or(false);

        if has_voted {
            return false;
        }

        let mut votes: Map<Symbol, u32> = env
            .storage()
            .persistent()
            .get(&DataKey::Votes)
            .unwrap_or(Map::new(&env));

        let current = votes.get(option.clone()).unwrap_or(0);
        votes.set(option, current + 1);

        env.storage().persistent().set(&DataKey::Votes, &votes);
        env.storage()
            .persistent()
            .set(&DataKey::HasVoted(voter), &true);

        true
    }

    /// Retorna el mapa completo de votos: { opcion -> cantidad }
    pub fn get_votes(env: Env) -> Map<Symbol, u32> {
        env.storage()
            .persistent()
            .get(&DataKey::Votes)
            .unwrap_or(Map::new(&env))
    }

    /// Verificar si una wallet ya votó
    pub fn has_voted(env: Env, voter: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::HasVoted(voter))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Env};

    fn setup() -> (Env, VotingContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register(
            VotingContract,
            (
                admin.clone(),
                vec![
                    &env,
                    symbol_short!("Opcion_A"),
                    symbol_short!("Opcion_B"),
                    symbol_short!("Opcion_C"),
                ],
            ),
        );
        let client = VotingContractClient::new(&env, &contract_id);
        (env, client, admin)
    }

    #[test]
    fn test_admin_is_set() {
        let (_, client, admin) = setup();
        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    #[should_panic]
    fn test_constructor_requires_admin_auth() {
        // Sin mock_all_auths() — el admin debe autorizar el deploy explícitamente.
        let env = Env::default();
        let admin = Address::generate(&env);
        env.register(
            VotingContract,
            (admin, vec![&env, symbol_short!("Opcion_A")]),
        );
    }

    #[test]
    fn test_vote_success() {
        let (env, client, _) = setup();
        let voter = Address::generate(&env);

        let result = client.vote(&voter, &symbol_short!("Opcion_A"));
        assert!(result);

        let votes = client.get_votes();
        assert_eq!(votes.get(symbol_short!("Opcion_A")).unwrap(), 1);
        assert!(client.has_voted(&voter));
    }

    #[test]
    fn test_double_vote_returns_false() {
        let (env, client, _) = setup();
        let voter = Address::generate(&env);

        assert!(client.vote(&voter, &symbol_short!("Opcion_A")));
        assert!(!client.vote(&voter, &symbol_short!("Opcion_B"))); // segundo voto → false
    }

    #[test]
    fn test_multiple_voters() {
        let (env, client, _) = setup();
        let v1 = Address::generate(&env);
        let v2 = Address::generate(&env);
        let v3 = Address::generate(&env);

        client.vote(&v1, &symbol_short!("Opcion_A"));
        client.vote(&v2, &symbol_short!("Opcion_A"));
        client.vote(&v3, &symbol_short!("Opcion_B"));

        let votes = client.get_votes();
        assert_eq!(votes.get(symbol_short!("Opcion_A")).unwrap(), 2);
        assert_eq!(votes.get(symbol_short!("Opcion_B")).unwrap(), 1);
    }

    #[test]
    fn test_empty_votes_on_init() {
        let (_, client, _) = setup();
        let votes = client.get_votes();
        // El mapa existe pero todas las opciones tienen 0 votos
        assert_eq!(votes.get(symbol_short!("Opcion_A")).unwrap_or(0), 0);
    }
}
