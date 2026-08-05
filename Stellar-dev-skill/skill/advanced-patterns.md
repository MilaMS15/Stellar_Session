# Patrones avanzados de contratos

Arquitecturas complejas, multicall, factory pattern, vaults y otros patrones para proyectos maduros.

---

## Factory Pattern — desplegar contratos desde un contrato

```rust
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Val, Vec};

#[contract]
pub struct Factory;

#[contractimpl]
impl Factory {
    pub fn __constructor(env: Env, admin: Address, wasm_hash: BytesN<32>) {
        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("wasm"), &wasm_hash);
    }

    pub fn crear_contrato(
        env: Env,
        creador: Address,
        salt: BytesN<32>,
        args_inicializacion: Vec<Val>,
    ) -> Address {
        creador.require_auth();

        let wasm_hash: BytesN<32> = env.storage().instance().get(&symbol_short!("wasm")).unwrap();

        // Direcciones deterministas por (deployer, salt) — deployed_address()
        // las calcula sin desplegar, útil para derivar salts intencionalmente.
        let nuevo_contrato = env
            .deployer()
            .with_address(creador.clone(), salt)
            .deploy_v2(wasm_hash, args_inicializacion);

        let key = symbol_short!("count");
        let count: u32 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().persistent().set(&count, &nuevo_contrato);
        env.storage().instance().set(&key, &(count + 1));

        nuevo_contrato
    }

    pub fn obtener_contrato(env: Env, indice: u32) -> Option<Address> {
        env.storage().persistent().get(&indice)
    }
}
```

`deploy_v2` pasa los argumentos de `__constructor` directamente al desplegar — no hace falta una llamada `init` separada después. Autoriza quién puede desplegar y emite un evento por deployment para que las instancias sean indexables.

---

## Vault Pattern — pool de liquidez básico

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, contractevent, contracttype, Address, Env};
use soroban_sdk::token::TokenClient;

#[contracttype]
#[derive(Clone)]
enum Clave {
    Token,
    TotalShares,
    Shares(Address),
    Admin,
}

#[contractevent]
pub struct Deposit {
    #[topic]
    pub usuario: Address,
    pub monto: i128,
    pub shares: i128,
}

#[contract]
pub struct Vault;

#[contractimpl]
impl Vault {
    pub fn __constructor(env: Env, admin: Address, token: Address) {
        env.storage().instance().set(&Clave::Admin, &admin);
        env.storage().instance().set(&Clave::Token, &token);
        env.storage().instance().set(&Clave::TotalShares, &0_i128);
    }

    pub fn depositar(env: Env, usuario: Address, monto: i128) -> i128 {
        usuario.require_auth();
        assert!(monto > 0, "monto inválido");

        let token: Address = env.storage().instance().get(&Clave::Token).unwrap();
        let total_shares: i128 = env.storage().instance().get(&Clave::TotalShares).unwrap_or(0);
        let token_client = TokenClient::new(&env, &token);
        let total_assets = token_client.balance(&env.current_contract_address());

        // Redondeo conservador: shares = 1:1 si el vault está vacío
        let shares = if total_shares == 0 || total_assets == 0 {
            monto
        } else {
            monto * total_shares / total_assets
        };

        token_client.transfer(&usuario, &env.current_contract_address(), &monto);

        let shares_usuario: i128 = env.storage().persistent().get(&Clave::Shares(usuario.clone())).unwrap_or(0);
        env.storage().persistent().set(&Clave::Shares(usuario.clone()), &(shares_usuario + shares));
        env.storage().instance().set(&Clave::TotalShares, &(total_shares + shares));

        Deposit { usuario, monto, shares }.publish(&env);
        shares
    }

    pub fn retirar(env: Env, usuario: Address, shares: i128) -> i128 {
        usuario.require_auth();
        assert!(shares > 0, "shares inválidas");

        let token: Address = env.storage().instance().get(&Clave::Token).unwrap();
        let total_shares: i128 = env.storage().instance().get(&Clave::TotalShares).unwrap();
        let token_client = TokenClient::new(&env, &token);
        let total_assets = token_client.balance(&env.current_contract_address());

        let shares_usuario: i128 = env.storage().persistent().get(&Clave::Shares(usuario.clone())).unwrap_or(0);
        assert!(shares_usuario >= shares, "shares insuficientes");

        let monto = shares * total_assets / total_shares;

        // Actualizar estado ANTES de la llamada externa
        env.storage().persistent().set(&Clave::Shares(usuario.clone()), &(shares_usuario - shares));
        env.storage().instance().set(&Clave::TotalShares, &(total_shares - shares));

        token_client.transfer(&env.current_contract_address(), &usuario, &monto);
        monto
    }

    pub fn shares_de(env: Env, usuario: Address) -> i128 {
        env.storage().persistent().get(&Clave::Shares(usuario)).unwrap_or(0)
    }

    pub fn total_assets(env: Env) -> i128 {
        let token: Address = env.storage().instance().get(&Clave::Token).unwrap();
        TokenClient::new(&env, &token).balance(&env.current_contract_address())
    }
}
```

---

## Multicall — agrupar múltiples operaciones

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

async function multicall(
  rpc: StellarSdk.rpc.Server,
  cuenta: StellarSdk.Account,
  operaciones: { contractId: string; funcion: string; args: StellarSdk.xdr.ScVal[] }[],
  firmar: (xdr: string) => Promise<string>,
  networkPassphrase: string,
) {
  const builder = new StellarSdk.TransactionBuilder(cuenta, {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase,
  });

  for (const op of operaciones) {
    const contrato = new StellarSdk.Contract(op.contractId);
    builder.addOperation(contrato.call(op.funcion, ...op.args));
  }

  const tx = builder.setTimeout(30).build();

  const sim = await rpc.simulateTransaction(tx);
  if (StellarSdk.rpc.Api.isSimulationError(sim)) {
    throw new Error(`Error en simulación: ${sim.error}`);
  }

  const txFinal = StellarSdk.rpc.assembleTransaction(tx, sim).build();
  const xdrFirmado = await firmar(txFinal.toXDR());
  const txFirmada = StellarSdk.TransactionBuilder.fromXDR(xdrFirmado, networkPassphrase) as StellarSdk.Transaction;

  return rpc.sendTransaction(txFirmada);
}
```

---

## Patrón de escrow genérico

```rust
use soroban_sdk::{contractevent, contracttype, Address, Env};
use soroban_sdk::token::TokenClient;

#[contracttype]
#[derive(Clone)]
pub struct Escrow {
    pub depositante: Address,
    pub beneficiario: Address,
    pub arbitro: Address,
    pub token: Address,
    pub monto: i128,
    pub estado: EstadoEscrow,
    pub vencimiento: u64,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum EstadoEscrow {
    Pendiente,
    Completado,
    Cancelado,
    EnDisputa,
}

#[contractevent]
pub struct EscrowCreado {
    #[topic]
    pub id: u64,
    pub depositante: Address,
    pub monto: i128,
}

#[contractimpl]
impl ContratoEscrow {
    pub fn crear(
        env: Env,
        depositante: Address,
        beneficiario: Address,
        arbitro: Address,
        token: Address,
        monto: i128,
        duracion_ledgers: u64,
    ) -> u64 {
        depositante.require_auth();

        let ledger_actual = env.ledger().sequence() as u64;
        let id = ledger_actual; // usar un contador dedicado en producción, no el ledger

        let escrow = Escrow {
            depositante: depositante.clone(),
            beneficiario,
            arbitro,
            token: token.clone(),
            monto,
            estado: EstadoEscrow::Pendiente,
            vencimiento: ledger_actual + duracion_ledgers,
        };

        TokenClient::new(&env, &token).transfer(&depositante, &env.current_contract_address(), &monto);

        env.storage().persistent().set(&id, &escrow);
        env.storage().persistent().extend_ttl(&id, 30 * 17280, 120 * 17280);

        EscrowCreado { id, depositante, monto }.publish(&env);
        id
    }

    pub fn liberar(env: Env, id: u64, quien_libera: Address) {
        let escrow: Escrow = env.storage().persistent().get(&id).unwrap();

        // El depositante o el árbitro pueden liberar — autorizar explícitamente
        // a quien realmente llama, no asumir un rol
        assert!(
            quien_libera == escrow.depositante || quien_libera == escrow.arbitro,
            "no autorizado"
        );
        quien_libera.require_auth();

        assert!(escrow.estado == EstadoEscrow::Pendiente, "escrow no está pendiente");

        let mut escrow_actualizado = escrow.clone();
        escrow_actualizado.estado = EstadoEscrow::Completado;
        env.storage().persistent().set(&id, &escrow_actualizado);

        TokenClient::new(&env, &escrow.token).transfer(
            &env.current_contract_address(),
            &escrow.beneficiario,
            &escrow.monto,
        );
    }
}
```

> Para escrow production-ready sin reinventar la rueda, considera [Trustless Work](https://docs.trustlesswork.com) — escrow como servicio con REST API y SDK React ya construidos sobre Stellar.

---

## Rate limiting — límite de velocidad en contratos

```rust
#[contracttype]
#[derive(Clone)]
struct LimiteVelocidad {
    pub monto_periodo: i128,
    pub ledger_inicio: u32,
    pub duracion_ledgers: u32,
}

fn verificar_limite(env: &Env, usuario: &Address, monto: i128, max_por_periodo: i128) {
    let clave = (soroban_sdk::symbol_short!("ratelim"), usuario.clone());
    let ledger_actual = env.ledger().sequence();

    let mut limite: LimiteVelocidad = env.storage().temporary()
        .get(&clave)
        .unwrap_or(LimiteVelocidad { monto_periodo: 0, ledger_inicio: ledger_actual, duracion_ledgers: 720 });

    if ledger_actual >= limite.ledger_inicio + limite.duracion_ledgers {
        limite.monto_periodo = 0;
        limite.ledger_inicio = ledger_actual;
    }

    assert!(limite.monto_periodo + monto <= max_por_periodo, "límite de velocidad excedido");

    limite.monto_periodo += monto;
    env.storage().temporary().set(&clave, &limite);
}
```

---

## Pausa de emergencia (circuit breaker)

Preferí el macro de OpenZeppelin (`#[when_not_paused]`, ver [SetUp_Dev.md](../../SetUp_Dev.md) en la raíz de este repo) si tu contrato ya usa `stellar-contract-utils`. Implementación manual:

```rust
#[contractimpl]
impl MiProtocolo {
    pub fn pausar(env: Env) {
        Self::require_admin(&env);
        env.storage().instance().set(&soroban_sdk::symbol_short!("paused"), &true);
    }

    pub fn reanudar(env: Env) {
        Self::require_admin(&env);
        env.storage().instance().set(&soroban_sdk::symbol_short!("paused"), &false);
    }

    fn verificar_no_pausado(env: &Env) {
        let pausado: bool = env.storage().instance().get(&soroban_sdk::symbol_short!("paused")).unwrap_or(false);
        assert!(!pausado, "protocolo pausado");
    }

    pub fn depositar(env: Env, usuario: Address, monto: i128) {
        Self::verificar_no_pausado(&env);
        usuario.require_auth();
    }
}
```

---

## Governance con timelock

Para acciones sensibles (upgrades, cambios de config): `propose_*` guarda la acción pendiente más un ledger de ejecución habilitada, `execute_*` aplica el delay, `cancel_*` permite abortar. Para multisig, separa roles proponer/aprobar/ejecutar y previene replay con IDs de propuesta únicos. Para políticas de firma, una custom account (`__check_auth`) suele ser más limpia que multisig dentro del contrato.

---

## Integración con protocolos DeFi del ecosistema

```typescript
// Blend — lending/borrowing (ver docs actuales para la interfaz exacta del SDK)
import { Pool, PoolContract } from "@blend-capital/blend-sdk";

const pool = new PoolContract(BLEND_POOL_ADDRESS);
// Proveer liquidez y solicitar préstamos vía el cliente de contrato del SDK de Blend —
// revisar https://docs.blend.capital antes de fijar la forma exacta de la llamada,
// el SDK evoluciona junto con el protocolo.
```

Antes de integrar un protocolo externo (Blend, Soroswap, DeFindex, Reflector), lee su documentación actual en vez de copiar snippets — estos SDKs cambian con frecuencia. Ver [ecosystem.md](ecosystem.md) para el directorio de protocolos.
