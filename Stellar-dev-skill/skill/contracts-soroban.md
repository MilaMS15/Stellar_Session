# Contratos Soroban — Desarrollo en Rust

Referencia completa para escribir, compilar, desplegar e invocar contratos inteligentes en Stellar (`soroban-sdk`).

---

## Estructura de un proyecto

```bash
stellar contract init mi-contrato   # genera un workspace Cargo con contracts/
cd mi-contrato
rustup target add wasm32v1-none     # una vez por toolchain
```

### Cargo.toml esencial

```toml
[lib]
crate-type = ["lib", "cdylib"]   # "lib" es necesario para tests y fuzzing

[dependencies]
soroban-sdk = "23"   # sigue la versión del protocolo — revisa crates.io/crates/soroban-sdk
                      # para la última. Mainnet suele ir un protocolo detrás de testnet.

[dev-dependencies]
soroban-sdk = { version = "23", features = ["testutils"] }

[profile.release]
opt-level = "z"
overflow-checks = true
debug = 0
strip = "symbols"
debug-assertions = false
panic = "abort"
codegen-units = 1
lto = true
```

> **La versión mayor de `soroban-sdk` sigue la versión del protocolo** (SDK 23 ↔ protocolo 23). Esta regla se mantiene en el tiempo — no fijes un número de versión de memoria, resuélvelo desde crates.io o `cargo add soroban-sdk`.

---

## Anatomía de un contrato

Ejemplo compacto con estado, constructor, auth, TTL, error tipado y evento:

```rust
#![no_std]
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Counter,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
}

// Se emite como topics ("incremented", by) con data {count}
// — los campos #[topic] son topics, el resto va en el payload.
#[contractevent]
pub struct Incremented {
    #[topic]
    pub by: Address,
    pub count: u32,
}

#[contract]
pub struct CounterContract;

#[contractimpl]
impl CounterContract {
    // Corre una sola vez, atómicamente, al desplegar. Debe llamarse `__constructor`.
    // No vuelve a correr en un upgrade.
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Counter, &0u32);
    }

    pub fn increment(env: Env) -> Result<u32, Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        let count: u32 = env.storage().instance().get(&DataKey::Counter).unwrap_or(0);
        let count = count + 1;
        env.storage().instance().set(&DataKey::Counter, &count);

        // Extiende el TTL para que el estado del contrato no se archive
        env.storage().instance().extend_ttl(120 * 17280, 180 * 17280);

        Incremented { by: admin, count }.publish(&env);
        Ok(count)
    }

    pub fn get_count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Counter).unwrap_or(0)
    }
}
```

`__constructor` reemplaza al viejo patrón de `initialize()` con guard manual (`assert!(!has(&Admin))`) — es atómico y elimina la ventana de front-running entre deploy e inicialización. Un contrato desplegado sin constructor no puede ganar uno retroactivamente, así que planifica la inicialización en el momento del deploy.

---

## Tipos de almacenamiento

| Tipo | Vida útil | Comportamiento al expirar | Usar para |
|---|---|---|---|
| `instance()` | Un TTL compartido con la instancia del contrato | Archivado, restaurable | Admin, config, estado global pequeño |
| `persistent()` | TTL por clave | **Archivado**, restaurable | Balances de usuarios, cualquier dato que deba sobrevivir |
| `temporary()` | TTL por clave | **Eliminado permanentemente** | Cachés, datos acotados en el tiempo, precios de oráculo |

```rust
env.storage().instance().set(&DataKey::Admin, &admin);
env.storage().persistent().set(&DataKey::Balance(user), &balance);
env.storage().temporary().set(&DataKey::Quote(pair), &price);
```

`instance()` es una sola entrada de ledger compartida con la instancia del contrato — barato mantener todo caliente con un solo `extend_ttl`, pero limitado a **64 KB serializados** en mainnet. Nunca pongas ahí datos por usuario o sin límite de cantidad.

### TTL — semántica correcta

Cada entrada tiene un TTL contado en ledgers (~5s cada uno, 17,280/día). En mainnet, las entradas `persistent` nuevas arrancan con ~120 días de TTL, `temporary` con ~1 día, y ninguna entrada puede superar ~180 días (valores configurados por red, verificar con `stellar network settings`).

```rust
const DAY_IN_LEDGERS: u32 = 17280;
const BUMP_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;
const BUMP_TO: u32 = 120 * DAY_IN_LEDGERS;

// extend_ttl(threshold, extend_to): no hace nada a menos que el TTL actual
// sea menor que threshold; en ese caso lo fija en extend_to. Idempotente
// y de solo-piso — nunca acorta el TTL.
env.storage().instance().extend_ttl(BUMP_THRESHOLD, BUMP_TO);
env.storage().persistent().extend_ttl(&DataKey::Balance(user), BUMP_THRESHOLD, BUMP_TO);
```

Dos datos que sorprenden:

- **Cualquiera puede extender el TTL de cualquier entrada** vía la operación `ExtendFootprintTTLOp`, sin auth del contrato. El TTL **no es un mecanismo de seguridad** — si algo debe invalidarse tras una fecha, guarda esa fecha en el valor y verifícala explícitamente.
- **Archivado no es perdido.** Desde el protocolo 23, las entradas `persistent` archivadas declaradas en el footprint read-write de una transacción se restauran automáticamente (se re-paga la renta); el CLI y los SDKs lo manejan durante la simulación. `temporary` no tiene ese camino — expirado significa eliminado.

---

## Tipos de datos

```rust
use soroban_sdk::{symbol_short, vec, Address, Bytes, BytesN, Map, String, Symbol, Vec, U256, I256};

let addr: Address = env.current_contract_address();
let sym: Symbol = symbol_short!("transfer");   // ≤9 chars; Symbol admite hasta 32
let s: String = String::from_str(&env, "hola");
let v: Vec<u32> = vec![&env, 1, 2, 3];
let m: Map<Symbol, u32> = Map::new(&env);
```

- `i128` es el tipo canónico para montos de tokens (SEP-41). Admite negativos — valida `amount >= 0` en los inputs.
- Matemática de 256 bits: `U256`/`I256` con métodos `checked_add/sub/mul/pow/div` que devuelven `Option`.

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Balance(Address),
    Allowance(Address, Address), // (owner, spender)
}
```

Usa siempre un enum `#[contracttype]` para las claves de storage — centraliza el esquema y evita colisiones. Mantén las claves granulares (`Balance(Address)` por usuario, no un `Map` gigante): cada transacción declara su footprint de antemano, y las transacciones que tocan la misma entrada se serializan.

---

## Autenticación y autorización

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();
    // o vincular la auth a argumentos específicos:
    // from.require_auth_for_args((&to, amount).into_val(&env));
}
```

`require_auth()` le pregunta al host: ¿autorizó `from` *este contrato, esta función, estos argumentos exactos* en esta transacción? Para cada parámetro `Address`, pregúntate si la llamada gasta, reduce o reconfigura algo que esa dirección posee. Para rutas privilegiadas, carga el admin desde storage y autoriza **ese** valor, no un parámetro arbitrario:

```rust
fn require_admin(env: &Env) {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
}
```

### Árboles de auth y llamadas cross-contract

Un usuario firma un **árbol de invocaciones**, no una sola llamada. `require_auth` en lo profundo del stack solo pasa si el camino de llamadas y los argumentos reales coinciden con el árbol firmado. Implicación crítica: **re-autoriza en cada capa que use la autoridad de una dirección.** Si `deposit(user, amount)` llama a `token.transfer(&user, ...)` pero nunca llama a `user.require_auth()`, cualquiera puede llamar `deposit(victima, ...)` y consumir una auth interna pre-firmada — es el bug de auth más común en producción.

---

## Eventos

Definir eventos con el macro `#[contractevent]` (el antiguo `env.events().publish(topics, data)` está deprecado):

```rust
use soroban_sdk::contractevent;

#[contractevent]
pub struct Transfer {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

// en una función del contrato:
Transfer { from, to, amount }.publish(&env);
```

Personalizable con `#[contractevent(topics = ["custom", "prefix"])]` y `data_format = "map" | "vec" | "single-value"` (default `map`). Emite eventos para todo cambio de estado que quieras indexar o auditar — son mucho más baratos que guardar estado consultable, pero son efímeros (RPC suele retener ~7 días).

---

## Llamadas a otros contratos (cross-contract)

```rust
mod token_contract {
    soroban_sdk::contractimport!(
        file = "../token/target/wasm32v1-none/release/token.wasm"
    );
}

pub fn deposit(env: Env, user: Address, token: Address, amount: i128) {
    user.require_auth();
    let token_client = token_contract::Client::new(&env, &token);
    token_client.transfer(&user, &env.current_contract_address(), &amount);
}
```

Para cualquier contrato que implemente la interfaz estándar de token (incluye todo activo Stellar vía su SAC), el SDK trae un cliente listo — **no uses `soroban_token_sdk`, ese crate no existe en el SDK actual:**

```rust
use soroban_sdk::token::TokenClient;

let token = TokenClient::new(&env, &token_address);
token.transfer(&from, &to, &amount);
```

No hay `msg.sender` — pasa la `Address` que actúa explícitamente y autorízala.

---

## Manejo de errores

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    NotInitialized = 1,
    InsufficientBalance = 2,
    InvalidAmount = 3,
}

pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::InvalidAmount);
    }
    Ok(())
}
```

- Devolver `Result` da a quien llama errores tipados: los clientes generados exponen `foo()` (panic si hay error) y `try_foo()` que devuelve `Result<Result<T, E>, InvokeError>`.
- Al hacer panic, usa `panic_with_error!(&env, Error::X)`, no `panic!("mensaje")` — un panic plano llega como error opaco de host que el cliente no puede matchear.
- Los códigos de error son ABI pública — nunca los renumeres entre upgrades.

---

## Compilar, desplegar e invocar

```bash
# Compila WASM optimizado → target/wasm32v1-none/release/*.wasm
stellar contract build

# Crear y fondear una identidad (testnet)
stellar keys generate mi-wallet --network testnet --fund

# Deploy (los argumentos del constructor van después de "--")
stellar contract deploy \
  --wasm target/wasm32v1-none/release/mi_contrato.wasm \
  --source-account mi-wallet \
  --network testnet \
  -- \
  --admin mi-wallet

# Invocar
stellar contract invoke \
  --id CONTRACT_ID \
  --source-account mi-wallet \
  --network testnet \
  -- \
  increment
```

Para subir el WASM sin instanciar (factories, upgrades) usa `stellar contract upload` (`stellar contract install` es un alias deprecado). Para ver la interfaz de un contrato desplegado y generar bindings TypeScript:

```bash
stellar contract bindings typescript \
  --id CONTRACT_ID \
  --network testnet \
  --output-dir ./src/contracts/mi-contrato
```

---

## Test mínimo

```rust
// src/test.rs — incluido desde lib.rs con `mod test;`
#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_increment() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(CounterContract, (admin.clone(),));
    let client = CounterContractClient::new(&env, &contract_id);

    assert_eq!(client.increment(), 1);
    assert_eq!(client.get_count(), 1);
}
```

`env.register` (no el antiguo `register_contract`) toma los argumentos del constructor como segundo parámetro. Auth mocking, aserciones de eventos, fuzzing y fork tests: [testing.md](testing.md).

---

## Upgrade de contratos (upgradeability)

Soroban tiene upgradeability nativa a nivel de protocolo — no se necesitan proxies. El contrato reemplaza su propio WASM bytecode. Un contrato sin función de upgrade es inmutable por defecto. El nuevo WASM solo entra en efecto al terminar la invocación actual — la migración no puede ejecutarse en el mismo call que el upgrade.

### Opción A — Manual (baseline)

```rust
use soroban_sdk::{Address, BytesN, Env};

#[contractimpl]
impl MiContrato {
    pub fn upgrade(env: Env, nuevo_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.deployer().update_current_contract_wasm(nuevo_wasm_hash);
    }
}
```

```bash
# Subir el nuevo WASM y obtener el hash (no instancia)
stellar contract upload \
  --wasm target/wasm32v1-none/release/mi_contrato_v2.wasm \
  --source-account mi-wallet \
  --network testnet

stellar contract invoke \
  --id CONTRACT_ID \
  --source-account mi-wallet \
  --network testnet \
  -- upgrade --nuevo_wasm_hash <WASM_HASH>
```

### Opción B — OpenZeppelin `#[derive(Upgradeable)]`

Para contratos que solo actualizan el WASM sin migración de storage:

```rust
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::Upgradeable;

#[contract]
#[derive(Upgradeable)]
pub struct MyToken;

impl UpgradeableInternal for MyToken {
    fn _require_auth(env: &Env, operator: &Address) {
        let owner: Address = env.storage().instance().get(&symbol_short!("OWNER")).unwrap();
        operator.require_auth();
        assert!(operator == &owner, "no autorizado");
    }
}
```

### Opción C — OpenZeppelin `#[derive(UpgradeableMigratable)]`

Para contratos que necesitan actualizar storage durante el upgrade:

```rust
use stellar_contract_utils::upgradeable::UpgradeableMigratableInternal;
use stellar_macros::UpgradeableMigratable;

#[contract]
#[derive(UpgradeableMigratable)]
pub struct MyContract;

impl UpgradeableMigratableInternal for MyContract {
    type MigrationData = String;

    fn _require_auth(env: &Env, operator: &Address) {
        let owner: Address = env.storage().instance().get(&symbol_short!("OWNER")).unwrap();
        operator.require_auth();
        assert!(operator == &owner, "no autorizado");
    }

    fn _migrate(env: &Env, data: &Self::MigrationData) {
        env.storage().instance().set(&symbol_short!("VER"), data);
    }
}
```

El macro garantiza que `migrate()` solo se puede invocar después de un upgrade exitoso. Ejemplos completos en el directorio `examples/` de [stellar-contracts](https://github.com/OpenZeppelin/stellar-contracts).

### Control de acceso para upgrades

El módulo upgradeable no incluye access control — debes definirlo en `_require_auth`. Olvidarlo permite que cualquiera reemplace el código del contrato.

### Compatibilidad de storage entre versiones

El nuevo WASM reinterpreta el storage existente — cambios incompatibles corrompen el estado:

- No eliminar ni renombrar claves de storage existentes
- No cambiar el tipo de valores almacenados bajo claves existentes
- Agregar nuevas claves es seguro
- El constructor del nuevo contrato **no se invoca** en un upgrade — cualquier inicialización debe ocurrir vía migración

---

## Checklist antes de desplegar

- [ ] El contrato usa `__constructor` para inicialización atómica (o guard verificado si usa `initialize`)
- [ ] Toda función que modifica estado requiere `require_auth()` — en la dirección correcta, cargada de storage
- [ ] Cada capa que usa la autoridad de una dirección la re-autoriza (sin auth replay a través de middleware)
- [ ] No hay enteros que puedan hacer overflow (`checked_add`, `i128`)
- [ ] El almacenamiento `persistent` tiene TTL extendido donde corresponde
- [ ] Eventos emitidos con `#[contractevent]` para acciones importantes
- [ ] Existe función de upgrade (o se decidió conscientemente no tenerla)
- [ ] El WASM compilado pesa menos de 128 KB
- [ ] Errores tipados con `#[contracterror]`, sin `panic!()` planos
