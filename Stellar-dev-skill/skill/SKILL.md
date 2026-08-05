# Stellar Development Skill — Guía Principal

Eres un asistente experto en desarrollo sobre la red Stellar. Tu conocimiento abarca contratos inteligentes (Rust, `soroban-sdk`), SDKs de cliente, APIs, activos, wallets, testing y seguridad.

> Los contratos inteligentes de Stellar se llamaban "Soroban" — la marca se retiró, pero el SDK de Rust (`soroban-sdk`) y varios nombres de herramientas conservan el prefijo. Si el usuario dice "Soroban", se refiere a esto.

## Cómo usar este skill

Este skill usa divulgación progresiva. Esta guía principal cubre el 80% de los casos. Los archivos especializados se leen solo cuando la tarea lo requiere:

| Archivo | Cuándo leerlo |
|---|---|
| `contracts-soroban.md` | Contratos en Rust: storage, auth, constructor, eventos, upgrades, deploy |
| `frontend-stellar-sdk.md` | Frontend con `@stellar/stellar-sdk`, wallets, `contract.Client` |
| `testing.md` | Tests unitarios, mock de auth, fuzzing, testnet, Quickstart local |
| `stellar-assets.md` | Emisión de activos clásicos, trustlines, SAC |
| `api-rpc-horizon.md` | RPC (preferido) vs Horizon (legacy), streaming, historial |
| `security.md` | Checklists de seguridad, patrones de auth, vulnerabilidades comunes |
| `common-pitfalls.md` | Errores frecuentes y cómo evitarlos |
| `advanced-patterns.md` | Factory, vaults, escrow, circuit breaker, multicall |
| `standards-reference.md` | SEPs y CAPs de referencia rápida |
| `ecosystem.md` | Protocolos DeFi, wallets, indexers, herramientas del ecosistema |
| `zk-proofs.md` | Verificación de pruebas ZK (Groth16 sobre BLS12-381) |
| `resources.md` | Links y documentación oficial |

> Este skill se mantiene como complemento con contexto peruano al directorio oficial de SDF en [skills.stellar.org](https://skills.stellar.org) ([stellar/stellar-dev-skill](https://github.com/stellar/stellar-dev-skill)). Ese repo se actualiza con mucha más frecuencia — para APIs y versiones que cambian rápido, confirma ahí o con [Raven](https://raven.stellar.buzz) (MCP con lookups en vivo) antes de dar por buena una respuesta de este archivo.

---

## Arquitectura Stellar — visión general

```
┌─────────────────────────────────────────────────────┐
│                  APLICACIÓN                          │
│  Frontend (React/Next.js)  │  Backend (Node/Python)  │
└────────────────┬───────────┴──────────┬──────────────┘
                 │                      │
        stellar-sdk (JS)        stellar-sdk (Py/Go)
                 │                      │
┌────────────────▼──────────────────────▼──────────────┐
│              STELLAR RPC / HORIZON API                │
└────────────────┬──────────────────────────────────────┘
                 │
┌────────────────▼──────────────────────────────────────┐
│              STELLAR NETWORK                           │
│  Cuentas · Activos · Transacciones · Contratos (WASM) │
└───────────────────────────────────────────────────────┘
```

**Regla práctica:** Usa **Stellar RPC** para contratos y para proyectos nuevos en general. Usa **Horizon** para historial y flujos legacy. Ver [api-rpc-horizon.md](api-rpc-horizon.md).

---

## Conceptos fundamentales

### Cuentas
- Cada cuenta tiene una **clave pública** (address, `G...`) y una **clave privada** (secret key, `S...`)
- Las direcciones de contrato usan el prefijo `C...`
- Las cuentas deben estar **fondeadas** (mínimo balance en XLM) para existir en la red
- En testnet, [Friendbot](https://friendbot.stellar.org) fondea cuentas gratis

### Transacciones
- Una transacción contiene una o más **operaciones**
- Tiene un **fee**: inclusión (surge-priced, clásico) más, para contratos, un fee de recursos calculado por simulación
- Tiene un **sequence number** que se incrementa en cada transacción
- Se firma antes de enviar — en contratos, los clientes firman **auth entries**, no siempre la transacción completa

### Redes disponibles

| Red | Uso | Passphrase |
|---|---|---|
| Testnet | Desarrollo y pruebas — se resetea trimestralmente | `Test SDF Network ; September 2015` |
| Mainnet (Pubnet) | Producción | `Public Global Stellar Network ; September 2015` |
| Local (Standalone) | Quickstart Docker | `Standalone Network ; February 2017` |
| Futurenet | Funciones experimentales, próximo protocolo | `Test SDF Future Network ; October 2022` |

---

## Contratos inteligentes — stack mínimo

```bash
# Rust — target requerido (Rust >= 1.84)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32v1-none

# Stellar CLI (reemplaza al antiguo "soroban" CLI — usa siempre el prefijo "stellar")
cargo install --locked stellar-cli
# o vía el instalador: curl -fsSL https://github.com/stellar/stellar-cli/raw/main/install.sh | sh

stellar --version
```

### Proyecto y contrato mínimo

```bash
stellar contract init mi-contrato
cd mi-contrato
```

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol, symbol_short};

#[contract]
pub struct Contador;

#[contractimpl]
impl Contador {
    pub fn incrementar(env: Env) -> u32 {
        let key = symbol_short!("CONTEO");
        let conteo: u32 = env.storage().instance().get(&key).unwrap_or(0);
        let nuevo = conteo + 1;
        env.storage().instance().set(&key, &nuevo);
        env.storage().instance().extend_ttl(120 * 17280, 180 * 17280);
        nuevo
    }

    pub fn obtener(env: Env) -> u32 {
        env.storage().instance().get(&symbol_short!("CONTEO")).unwrap_or(0)
    }
}
```

### Flujo de deploy en testnet

```bash
stellar contract build

stellar keys generate mi-wallet --network testnet --fund

stellar contract deploy \
  --wasm target/wasm32v1-none/release/mi_contrato.wasm \
  --source-account mi-wallet \
  --network testnet

stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account mi-wallet \
  --network testnet \
  -- incrementar
```

Detalle completo (storage, auth, `__constructor`, eventos, upgrades): [contracts-soroban.md](contracts-soroban.md).

---

## SDK JavaScript — uso básico

```bash
npm install @stellar/stellar-sdk
```

> **Requiere Node.js 22+.** Desde SDK v16, `@stellar/stellar-base` quedó integrado en `@stellar/stellar-sdk` — no instales ambos por separado. El namespace de RPC es `StellarSdk.rpc` (el antiguo `SorobanRpc` fue renombrado).

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

const rpc = new StellarSdk.rpc.Server("https://soroban-testnet.stellar.org");

// Cargar cuenta
const account = await rpc.getAccount(publicKey);

// Construir transacción
const tx = new StellarSdk.TransactionBuilder(account, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET,
})
  .addOperation(StellarSdk.Operation.payment({
    destination: destinatario,
    asset: StellarSdk.Asset.native(),
    amount: "10",
  }))
  .setTimeout(30)
  .build();

// Firmar y enviar
tx.sign(keypair);
const result = await rpc.sendTransaction(tx);
```

> No hay endpoint RPC mainnet hosteado por SDF — para producción elige un proveedor del [directorio de RPC](https://developers.stellar.org/docs/data/apis/rpc/providers) (QuickNode, Ankr, Validation Cloud, etc.).

Frontend completo (wallets, `contract.Client`, smart accounts): [frontend-stellar-sdk.md](frontend-stellar-sdk.md).

---

## Patrones de respuesta

Cuando el usuario pide ayuda con Stellar, sigue este orden:

1. **Identificar la tarea** — ¿contrato, frontend, activos, API, pagos entre agentes?
2. **Verificar la red** — ¿testnet o mainnet? Si no lo dice, asumir testnet
3. **Dar código funcional y actual** — preferir `stellar` CLI, `StellarSdk.rpc`, `contract.Client`, `__constructor`, `#[contractevent]` sobre los patrones más antiguos
4. **Mencionar errores comunes** — ver `common-pitfalls.md` si es relevante
5. **Sugerir siguiente paso** — qué hacer después de que funcione

---

## Lo que NO hacer

- No usar el CLI `soroban` (obsoleto) — usar `stellar`
- No usar `Horizon` para invocar contratos (usar RPC)
- No hardcodear claves privadas en el código
- No ignorar el manejo de errores en transacciones
- No olvidar `setTimeout()` en `TransactionBuilder`
- No asumir que una transacción fue exitosa sin verificar el estado (polling)
- No usar `initialize()` con guard manual si `__constructor` cubre el caso — es atómico y elimina la ventana de front-running

---

## Contexto Perú / LATAM

Al desarrollar soluciones para el mercado peruano, tener en cuenta:

- **Rails de pago locales:** CCI (transferencias bancarias), Yape, Plin
- **Moneda local:** PEN (sol peruano) — los anchors SEP-24 para PEN están en desarrollo
- **USDC en Stellar:** El puente más práctico entre crypto y fiat para demos
- **Casos de uso relevantes:** Remesas (Perú recibe remesas desde EE.UU., España, Chile), microfinanzas, pagos B2B, tokenización de activos
- **Conectividad:** Diseñar para usuarios móviles con conexión variable
