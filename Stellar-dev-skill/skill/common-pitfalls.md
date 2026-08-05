# Errores comunes y cómo evitarlos

Problemas frecuentes al desarrollar en Stellar, con sus soluciones.

---

## Errores de cuenta y transacciones

### `tx_bad_seq` — Sequence number incorrecto

```typescript
// MAL — reutilizar la misma cuenta cargada para múltiples txs seguidas
const cuenta = await rpc.getAccount(publicKey);
const tx1 = new StellarSdk.TransactionBuilder(cuenta, ...).build();
const tx2 = new StellarSdk.TransactionBuilder(cuenta, ...).build(); // mismo sequence!

// BIEN — recargar la cuenta antes de cada transacción
const cuenta1 = await rpc.getAccount(publicKey);
const tx1 = new StellarSdk.TransactionBuilder(cuenta1, ...).build();
await enviarYEsperar(tx1);
const cuenta2 = await rpc.getAccount(publicKey); // recargar
```

### `tx_insufficient_fee`

```typescript
// MAL — BASE_FEE puede ser insuficiente para una transacción de contrato compleja
const tx = new StellarSdk.TransactionBuilder(cuenta, { fee: StellarSdk.BASE_FEE }).build();

// BIEN — simular primero para obtener el fee real
const sim = await rpc.simulateTransaction(txSinFee);
const txConFee = StellarSdk.rpc.assembleTransaction(txSinFee, sim).build();
```

### Transacción expirada

```typescript
// BIEN — dar tiempo suficiente para firmar y enviar
const tx = new StellarSdk.TransactionBuilder(cuenta, { fee: StellarSdk.BASE_FEE })
  .addOperation(op)
  .setTimeout(300) // 5 min — suficiente para flujo con wallet
  .build();
```

---

## Errores de contratos

### Simulación exitosa pero transacción fallida

```typescript
// Causa: los datos del ledger cambiaron entre simulación y envío
const sim = await rpc.simulateTransaction(tx);
if (StellarSdk.rpc.Api.isSimulationError(sim)) throw new Error(sim.error);

// assembleTransaction actualiza el footprint con los datos de la simulación
const txFinal = StellarSdk.rpc.assembleTransaction(tx, sim).build();
// Firmar y enviar inmediatamente después
```

### `require_auth` falla en tests

```rust
// MAL — sin mock de auth
#[test]
fn test_transferir() {
    let env = Env::default();
    let client = MiContratoClient::new(&env, &id);
    client.transferir(&usuario, &otro, &100); // FALLA — no hay auth simulada
}

// BIEN
#[test]
fn test_transferir() {
    let env = Env::default();
    env.mock_all_auths();
    let client = MiContratoClient::new(&env, &id);
    client.transferir(&usuario, &otro, &100);
}
```

### Datos expirados por TTL

```rust
// BIEN — extender TTL al escribir (threshold, extend_to — no min/max)
pub fn actualizar_posicion(env: Env, usuario: Address, monto: i128) {
    usuario.require_auth();
    let clave = Clave::Posicion(usuario);
    env.storage().persistent().set(&clave, &monto);
    env.storage().persistent().extend_ttl(&clave, 30 * 17280, 120 * 17280);
}
```

### Usar `soroban_token_sdk::TokenClient`

```rust
// MAL — este crate no existe en el SDK actual
use soroban_token_sdk::TokenClient;

// BIEN — el cliente de token vive en el SDK principal
use soroban_sdk::token::TokenClient;
```

### Usar `env.events().publish()` en vez de `#[contractevent]`

```rust
// Funciona pero está deprecado
env.events().publish((symbol_short!("transfer"), from), (to, amount));

// Preferido — tipado, con topics explícitos
#[contractevent]
pub struct Transfer {
    #[topic] pub from: Address,
    #[topic] pub to: Address,
    pub amount: i128,
}
Transfer { from, to, amount }.publish(&env);
```

---

## Errores de SDK JavaScript

### `SorobanRpc` no existe / import falla

```typescript
// MAL — namespace antiguo (pre SDK v16)
import { SorobanRpc } from "@stellar/stellar-sdk";
const rpc = new SorobanRpc.Server(url);

// BIEN
import * as StellarSdk from "@stellar/stellar-sdk";
const rpc = new StellarSdk.rpc.Server(url);
```

### `Account not found` al cargar cuenta

```typescript
async function verificarOCrearCuenta(publicKey: string) {
  try {
    await rpc.getAccount(publicKey);
  } catch (e: any) {
    if (e.message?.includes("Not Found")) {
      await fetch(`https://friendbot.stellar.org?addr=${publicKey}`); // solo testnet
      await new Promise(r => setTimeout(r, 2000));
    } else {
      throw e;
    }
  }
}
```

### `getPublicKey()` de Freighter no existe / deprecado

```typescript
// MAL — deprecado
const { publicKey } = await freighter.getPublicKey();

// BIEN
import { getAddress } from "@stellar/freighter-api";
const { address } = await getAddress();
```

### Stellar Wallets Kit — mezclar v1 y v2

```typescript
// MAL — API v1 (npm, con punto, instanciada)
import { StellarWalletsKit } from "@creit.tech/stellar-wallets-kit";
const kit = new StellarWalletsKit({ ... });
await kit.openModal({ onWalletSelected: ... });

// BIEN — API v2 (JSR, con guion, estática)
import { StellarWalletsKit, Networks } from "@creit-tech/stellar-wallets-kit";
import { defaultModules } from "@creit-tech/stellar-wallets-kit/modules/utils";
StellarWalletsKit.init({ modules: defaultModules(), network: Networks.TESTNET });
const { address } = await StellarWalletsKit.authModal();
```

### El resultado de `scValToNative` es un BigInt inesperado

```typescript
// i128 en Soroban se recibe como BigInt en JS, no como number
const resultado = StellarSdk.scValToNative(retval); // 1000n, no 1000
const numero = Number(resultado); // convertir explícitamente si necesitas number
```

---

## Errores de compilación Rust

### Target incorrecto

```bash
# Error: error[E0463]: can't find crate for `std`
# MAL — target viejo
rustup target add wasm32-unknown-unknown

# BIEN — el único target que soporta el runtime de Stellar hoy (Rust >= 1.84)
rustup target add wasm32v1-none
```

### Contrato demasiado grande

```bash
# El límite real es 128 KB, no 64 KB
ls -la target/wasm32v1-none/release/*.wasm

# stellar contract build ya optimiza por defecto
stellar contract build

cargo install cargo-bloat
cargo bloat --release --target wasm32v1-none  # encontrar dependencias pesadas
```

### Error de `no_std`

```rust
// MAL — no disponible en no_std
println!("Debug: {}", valor);

// BIEN — usar eventos para diagnóstico on-chain, o std solo en tests
#[cfg(test)]
extern crate std;
```

---

## Errores de configuración y red

### CLI `soroban` no reconocido

```bash
# El CLI "soroban" es obsoleto — usa "stellar" para todo
# MAL
soroban contract deploy ...
# BIEN
stellar contract deploy ...
```

### Passphrase de red incorrecta

```typescript
const NETWORK_PASSPHRASE = {
  testnet: "Test SDF Network ; September 2015",
  mainnet: "Public Global Stellar Network ; September 2015",
  local: "Standalone Network ; February 2017",
  futurenet: "Test SDF Future Network ; October 2022",
};
```

### URL de RPC mainnet inventada

```typescript
// MAL — este endpoint no existe
const rpc = new StellarSdk.rpc.Server("https://soroban-mainnet.stellar.org");

// BIEN — usar un proveedor (ver directorio de RPC providers)
const rpc = new StellarSdk.rpc.Server(process.env.NEXT_PUBLIC_STELLAR_MAINNET_RPC_URL!);
```

---

## Gotchas que sorprenden

### `address.require_auth()` con sub-invocaciones

Si tu contrato llama a otro contrato que a su vez hace `require_auth` del mismo usuario, la auth del usuario debe abarcar toda la cadena de llamadas — un árbol, no una llamada aislada. El SDK JS arma el árbol automáticamente desde la simulación, pero si tu contrato hace una llamada *más profunda* que la directa, debe pre-autorizarla con `authorize_as_current_contract`.

### Anyone puede extender el TTL de tu contrato

`ExtendFootprintTTLOp` no requiere auth del contrato — es una feature (permite servicios de keep-alive benévolos), pero significa que el TTL nunca es un mecanismo de seguridad.

### Storage tras un upgrade

Después de un upgrade, el storage existente persiste tal cual — el nuevo WASM lo reinterpreta. Si cambias el esquema de datos, migra explícitamente con un entrypoint `migrate` gateado por versión, nunca asumas que el nuevo código puede leer datos con una forma distinta sin más.
