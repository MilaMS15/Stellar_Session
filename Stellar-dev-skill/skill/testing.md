# Testing en Stellar

Estrategia de testing para contratos, del test unitario nativo más rápido hasta pruebas contra estado real de mainnet.

Capas, de más rápida a más lenta:

1. **Tests unitarios** — Rust nativo con `testutils` del SDK (no WASM, debugger completo)
2. **Integración local** — contenedor Stellar Quickstart
3. **Testnet** — ensayo en red pública
4. **Fork tests** — replay contra estado real del ledger

---

## Tests unitarios

```rust
// src/test.rs — archivo separado incluido desde lib.rs con `mod test;`
#![cfg(test)]
extern crate std; // los contratos son no_std; los tests pueden usar std explícitamente
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};
use crate::{Contract, ContractClient};

#[test]
fn test_basico() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());   // () = sin argumentos de constructor
    let client = ContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    client.set_owner(&user);
    assert_eq!(client.get_value(), 0);
}
```

`env.register` (reemplaza al antiguo `register_contract`) toma los argumentos del constructor como segundo parámetro: `env.register(Contract, (admin.clone(),))` para un contrato con `__constructor(env, admin)`.

> **Los límites de recursos se aplican en tests** (SDK 25+): `Env::default()` aplica los límites de CPU/memoria de mainnet, así que un contrato que reventaría el budget en producción falla también en el test unitario — es una feature. Para tests deliberadamente pesados: `env.cost_estimate().disable_resource_limits()`.

### Simular autenticación

`mock_all_auths()` aprueba todo — cómodo, pero puede esconder un `require_auth` faltante. Combínalo siempre con aserciones sobre `env.auths()`, o mockea auths específicas:

```rust
#[test]
fn test_auth() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let otro = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &user,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "transfer",
            args: (&user, &otro, &100i128).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.transfer(&user, &otro, &100);
    assert!(!env.auths().is_empty()); // verificar que la auth fue realmente requerida
}
```

`env.auths()` devuelve pares `(Address, AuthorizedInvocation)` **solo de la invocación más reciente** — cualquier llamada posterior (incluso una lectura) lo resetea, así que asértalo inmediatamente después de la llamada bajo prueba.

### Verificar que falla correctamente

```rust
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_transferencia_sin_fondos() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let otro = Address::generate(&env);
    client.transfer(&user, &otro, &1000_i128); // debe fallar
}
```

Con errores tipados (`#[contracterror]`), el panic message incluye el código — usa `try_transfer` en el cliente si prefieres capturar el `Result` en vez de esperar un panic.

### Eventos

Los eventos definidos con `#[contractevent]` se comparan directamente contra la XDR emitida:

```rust
use soroban_sdk::{testutils::Events as _, Event as _};

let expected = Transfer { from: from.clone(), to: to.clone(), amount: 100 };
assert_eq!(env.events().all(), std::vec![expected.to_xdr(&env, &contract_id)]);
```

`env.events().all()` también se resetea en cada llamada — asértalo justo después.

### Tiempo y estado del ledger

```rust
env.ledger().set_timestamp(1000);
env.ledger().set_sequence_number(1000);
// ... actuar, luego avanzar:
env.ledger().set_timestamp(2500);
```

### TTL de storage

```rust
use soroban_sdk::testutils::storage::Persistent as _;

let ttl: u32 = env.as_contract(&contract_id, || {
    env.storage().persistent().get_ttl(&DataKey::MyData)
});
assert!(ttl > 0);
```

### Test con tokens SAC

```rust
use soroban_sdk::testutils::Address as _;

#[test]
fn test_con_token() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token = soroban_sdk::token::Client::new(&env, &token_id.address());

    let usuario = Address::generate(&env);
    let sac_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id.address());
    sac_admin.mint(&usuario, &1000_i128);

    assert_eq!(token.balance(&usuario), 1000);

    let contract_id = env.register(MiProtocolo, ());
    let cliente = MiProtocoloClient::new(&env, &contract_id);
    cliente.depositar(&token_id.address(), &usuario, &500_i128);
    assert_eq!(token.balance(&usuario), 500);
}
```

> `soroban_token_sdk` no existe como crate separado en el SDK actual — el cliente de token vive en `soroban_sdk::token`.

### Cross-contract

Registra una dependencia real desde su WASM y prueba la interacción:

```rust
mod token { soroban_sdk::contractimport!(file = "token.wasm"); }

let token_id = env.register(token::WASM, ());
let vault_id = env.register(VaultContract, ());
// mueve ambos por sus clientes tipados
```

---

## Correr los tests

```bash
cargo test
cargo test test_transferencia_autenticada
cargo test -- --nocapture
```

---

## Red local (Quickstart)

```bash
stellar container start local     # o: docker run --rm -it -p 8000:8000 stellar/quickstart:latest --local
stellar keys generate mi-cuenta --network local --fund
stellar contract deploy --wasm target/wasm32v1-none/release/mi_contrato.wasm \
  --source-account mi-cuenta --network local
```

Endpoints: RPC `http://localhost:8000/soroban/rpc`, Horizon `http://localhost:8000`, passphrase `"Standalone Network ; February 2017"`.

---

## Testnet

```bash
stellar keys generate mi-cuenta --network testnet --fund
stellar contract build
stellar contract deploy \
  --wasm target/wasm32v1-none/release/mi_contrato.wasm \
  --source-account mi-cuenta \
  --network testnet
```

- **Testnet se resetea trimestralmente** — todo se borra. Scriptea tus deploys, nunca trates el estado de testnet como duradero.
- Testnet corre la próxima versión de protocolo antes que mainnet — es donde verificas contra un upgrade próximo.

### Script de deploy y prueba

```bash
#!/bin/bash
set -e

echo "Compilando..."
stellar contract build

echo "Desplegando en testnet..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/mi_contrato.wasm \
  --source-account mi-cuenta \
  --network testnet \
  -- --admin "$(stellar keys address mi-cuenta)")
echo "Contract ID: $CONTRACT_ID"

echo "Prueba de función..."
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source-account mi-cuenta \
  --network testnet \
  -- mi_funcion --param "hola"
```

---

## Tests de integración con TypeScript

Contra local o testnet, el mismo flujo que usa un frontend (simular, ensamblar, firmar, enviar):

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

const rpc = new StellarSdk.rpc.Server(process.env.RPC_URL!);
const account = await rpc.getAccount(keypair.publicKey());
const contract = new StellarSdk.Contract(contractId);

const tx = new StellarSdk.TransactionBuilder(account, { fee: "100", networkPassphrase })
  .addOperation(contract.call("initialize", StellarSdk.Address.fromString(keypair.publicKey()).toScVal()))
  .setTimeout(30)
  .build();

const sim = await rpc.simulateTransaction(tx);
const prepared = StellarSdk.rpc.assembleTransaction(tx, sim).build();
prepared.sign(keypair);
const result = await rpc.sendTransaction(prepared);
```

En Rust, marca los tests que dependen de red con `#[ignore]` y córrelos con `cargo test -- --ignored` donde haya red disponible.

---

## Fuzz testing

Todos los tipos `#[contracttype]` implementan `SorobanArbitrary` bajo el feature `testutils`, así que el fuzzing funciona listo con `cargo-fuzz`:

```bash
rustup install nightly
cargo install --locked cargo-fuzz
cargo fuzz init
```

```rust
// fuzz/fuzz_targets/fuzz_deposit.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use soroban_sdk::{testutils::Address as _, Address, Env};
use my_contract::{Contract, ContractClient};

fuzz_target!(|amount: i128| {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    let _ = client.try_deposit(&user, &amount); // nunca debe hacer panic inesperado
});
```

```bash
cargo +nightly fuzz run fuzz_deposit
```

## Property-based testing

`proptest` + `SorobanArbitrary` corre en `cargo test` normal — úsalo para fijar invariantes encontrados por fuzzing:

```rust
proptest! {
    #[test]
    fn deposit_then_withdraw_preserves_balance(amount in 1i128..=i128::MAX) {
        client.deposit(&user, &amount);
        client.withdraw(&user, &amount);
        prop_assert_eq!(client.balance(&user), 0);
    }
}
```

## Snapshots, fork tests y mutation tests

- **Test snapshots**: cada test escribe un snapshot JSON de eventos + estado final del ledger en `test_snapshots/`. Comitéalos — los diffs exponen cambios de comportamiento no intencionados.
- **Fork testing**: `stellar snapshot create --address C... --output json --out snapshot.json`, luego `Env::from_ledger_snapshot_file("snapshot.json")` para probar contra estado real de red.
- **Mutation testing**: `cargo install --locked cargo-mutants && cargo mutants` — muta tu código fuente y reporta `MISSED` donde los tests no lo detectaron.

## Perfilado de recursos

```bash
stellar contract invoke --id CONTRACT_ID --source-account mi-cuenta --network testnet \
  --send=no -- funcion --arg valor
```

La simulación reporta instrucciones de CPU, lecturas/escrituras y fees sin enviar. En tests unitarios, `env.cost_estimate().resources()` da los mismos números programáticamente.

---

## CI

```yaml
name: Test contracts
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: rustup target add wasm32v1-none
      - run: cargo test
      - run: cargo build --release --target wasm32v1-none
```

---

## Checklist de testing

- [ ] Tests unitarios cubren todas las funciones públicas, incluidos los caminos de error (`try_` methods)
- [ ] Casos límite: montos en cero, valores máximos, estado vacío
- [ ] Autorización verificada con `env.auths()` o `mock_auths` específicos
- [ ] Eventos asertados (comparación `event.to_xdr`)
- [ ] Comportamiento de TTL de storage validado
- [ ] Interacciones cross-contract probadas contra WASM real
- [ ] Fuzz targets para rutas que mueven valor (deposit, withdraw, swap)
- [ ] Test snapshots comiteados
- [ ] Test de integración contra red local en CI
- [ ] Ensayo en testnet antes de mainnet
