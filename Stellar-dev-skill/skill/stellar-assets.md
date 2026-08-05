# Activos Stellar y Stellar Asset Contract (SAC)

Guía para emitir activos clásicos de Stellar, gestionar trustlines y usar el puente SAC hacia contratos.

---

## Dos mecanismos de token en Stellar

1. **Stellar Assets (Classic)**: nativo del protocolo, muy eficiente, soporte completo del ecosistema (wallets, DEX, exchanges)
2. **Contract tokens (SEP-41)**: contratos custom con lógica flexible

**Recomendación:** preferir Stellar Assets salvo que necesites lógica de token custom.

---

## Activos nativos de Stellar (Classic)

| Tipo | Descripción |
|---|---|
| Native (XLM) | Moneda nativa, sin trustline |
| Credit Alphanum4 | Código de 1-4 caracteres |
| Credit Alphanum12 | Código de 5-12 caracteres |

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

const xlm = StellarSdk.Asset.native();
const usdc = new StellarSdk.Asset("USDC", "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN");
```

### Emitir un activo

```typescript
const server = new StellarSdk.Horizon.Server("https://horizon-testnet.stellar.org");

// 1. Crear cuentas emisora y distribuidora (separadas)
const issuerKeypair = StellarSdk.Keypair.random();
const distributorKeypair = StellarSdk.Keypair.random();
await fetch(`https://friendbot.stellar.org?addr=${issuerKeypair.publicKey()}`);
await fetch(`https://friendbot.stellar.org?addr=${distributorKeypair.publicKey()}`);

const asset = new StellarSdk.Asset("MYTOKEN", issuerKeypair.publicKey());

// 2. El distribuidor crea trustline al emisor
const distributorAccount = await server.loadAccount(distributorKeypair.publicKey());
const trustlineTx = new StellarSdk.TransactionBuilder(distributorAccount, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET,
})
  .addOperation(StellarSdk.Operation.changeTrust({ asset, limit: "1000000" }))
  .setTimeout(180)
  .build();
trustlineTx.sign(distributorKeypair);
await server.submitTransaction(trustlineTx);

// 3. El emisor envía tokens al distribuidor
const issuerAccount = await server.loadAccount(issuerKeypair.publicKey());
const issueTx = new StellarSdk.TransactionBuilder(issuerAccount, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET,
})
  .addOperation(StellarSdk.Operation.payment({
    destination: distributorKeypair.publicKey(),
    asset,
    amount: "1000000",
  }))
  .setTimeout(180)
  .build();
issueTx.sign(issuerKeypair);
await server.submitTransaction(issueTx);
```

### Bloquear la cuenta emisora (supply fijo)

```typescript
const lockTx = new StellarSdk.TransactionBuilder(issuerAccount, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET,
})
  .addOperation(StellarSdk.Operation.setOptions({ masterWeight: 0 }))
  .setTimeout(180)
  .build();
// El emisor nunca podrá emitir más tokens
```

---

## Flags del emisor (compliance)

```typescript
const setFlagsTx = new StellarSdk.TransactionBuilder(issuerAccount, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET,
})
  .addOperation(StellarSdk.Operation.setOptions({
    setFlags:
      StellarSdk.AuthRequiredFlag |
      StellarSdk.AuthRevocableFlag |
      StellarSdk.AuthClawbackEnabledFlag,
  }))
  .setTimeout(180)
  .build();
```

| Flag | Efecto |
|---|---|
| `AUTH_REQUIRED` | Los usuarios necesitan aprobación antes de recibir tokens |
| `AUTH_REVOCABLE` | El emisor puede congelar balances |
| `AUTH_IMMUTABLE` | Los flags no se pueden cambiar (permanente) |
| `AUTH_CLAWBACK_ENABLED` | El emisor puede hacer clawback de tokens |

### Autorizar trustline (con AUTH_REQUIRED)

```typescript
const authorizeTx = new StellarSdk.TransactionBuilder(issuerAccount, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET,
})
  .addOperation(StellarSdk.Operation.setTrustLineFlags({
    trustor: userPublicKey,
    asset,
    flags: { authorized: true },
  }))
  .setTimeout(180)
  .build();
```

### Clawback

```typescript
const clawbackTx = new StellarSdk.TransactionBuilder(issuerAccount, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET,
})
  .addOperation(StellarSdk.Operation.clawback({ asset, from: targetAccountId, amount: "100" }))
  .setTimeout(180)
  .build();
```

---

## Trustlines

```typescript
const changeTrustTx = new StellarSdk.TransactionBuilder(userAccount, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET,
})
  .addOperation(StellarSdk.Operation.changeTrust({ asset, limit: "10000" })) // 0 elimina la trustline
  .setTimeout(180)
  .build();
```

```typescript
const account = await server.loadAccount(userPublicKey);
const trustline = account.balances.find(
  (b) => b.asset_type !== "native" && b.asset_code === "USDC" && b.asset_issuer === usdcIssuer
);
if (trustline) {
  console.log("Balance:", trustline.balance, "Autorizado:", trustline.is_authorized);
}
```

---

## Stellar Asset Contract (SAC)

El SAC expone un activo clásico con interfaz de contrato — así los contratos pueden usar cualquier activo Stellar.

### Deploy y dirección del SAC

```bash
stellar contract asset deploy \
  --asset USDC:GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN \
  --source-account alice \
  --network testnet
```

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

const asset = new StellarSdk.Asset("USDC", issuerPublicKey);
const contractId = asset.contractId(StellarSdk.Networks.TESTNET); // dirección SAC determinística
```

### Usar SAC desde un contrato Soroban

El cliente de token vive en el SDK principal — **no existe el crate `soroban_token_sdk`**:

```rust
use soroban_sdk::token::{TokenClient, StellarAssetClient};

pub fn depositar(env: Env, token: Address, usuario: Address, monto: i128) {
    usuario.require_auth();
    let token_client = TokenClient::new(&env, &token);
    token_client.transfer(&usuario, &env.current_contract_address(), &monto);
}

pub fn saldo_token(env: Env, token: Address, cuenta: Address) -> i128 {
    TokenClient::new(&env, &token).balance(&cuenta)
}

// Superficie admin del SAC — requiere ser el admin del activo
pub fn acunar(env: Env, token: Address, hacia: Address, monto: i128) {
    StellarAssetClient::new(&env, &token).mint(&hacia, &monto);
}
```

Puntos a internalizar:

- Auth: `transfer`/`approve`/`burn` autorizan a `from`; `transfer_from`/`burn_from` autorizan al `spender`. `mint`/`clawback`/`set_admin`/`set_authorized` son superficie admin del SAC, no parte de SEP-41.
- **El SAC hereda la semántica del activo clásico**: los balances de cuentas (`G...`) viven en trustlines (sin trustline o no autorizada → falla la transferencia; tope de balance de 64 bits), los balances de contratos (`C...`) viven en storage de contrato (i128 completo). Aplican los flags del issuer — `AUTH_REQUIRED`, freezes vía `AUTH_REVOCABLE`, clawback. Transferir al issuer quema; desde el issuer emite.

---

## Interfaz de token completa (SEP-41)

```
balance(id: Address) -> i128
transfer(from: Address, to: Address, amount: i128)
approve(from: Address, spender: Address, amount: i128, expiration_ledger: u32)
allowance(from: Address, spender: Address) -> i128
transfer_from(spender: Address, from: Address, to: Address, amount: i128)
burn(from: Address, amount: i128)
burn_from(spender: Address, from: Address, amount: i128)
decimals() -> u32
name() -> String
symbol() -> Symbol
```

Cualquier token custom que implemente esta interfaz interopera con wallets e indexers igual que el SAC.

---

## Cuándo usar cada opción

**Stellar Assets (classic)** — token fungible estándar (moneda, stablecoin), necesitas soporte de ecosistema completo (wallets, exchanges, DEX vía order book), cumplimiento regulatorio (freeze, clawback), performance.

**Custom contract token** — lógica de transferencia compleja (royalties, fees, restricciones), esquemas de autorización no estándar, NFTs o tokens semi-fungibles.

> Antes de un token 100% custom, considera un activo Stellar normal más un contrato "SAC admin" delgado que maneje `mint`, `burn` y `clawback` del SAC — mantiene la compatibilidad de ecosistema (DEX, anchors, wallets, trustlines) mientras agregas lógica custom encima.

---

## Consultar activos

```typescript
const assets = await server.assets().forCode("USDC").call();
const assetDetails = await server.assets().forCode("USDC").forIssuer(issuerPublicKey).call();
// stats incluye: amount total emitido, num_accounts (trustlines), flags
```

---

## SEPs relevantes para activos

- **SEP-1** (`stellar.toml`) — metadata del activo publicada en el dominio
- **SEP-10** — autenticación web con cuenta Stellar
- **SEP-24** — on/off ramp fiat con webview interactivo
- **SEP-45** — SEP-10 extendido para smart wallets (`C...`)
- **SEP-50** — interfaz estándar para NFTs (draft, ver implementación de referencia en OpenZeppelin Stellar Contracts)

Ver [standards-reference.md](standards-reference.md) para el mapa completo.

---

## Casos de uso para Perú

### Token PEN simulado para demos

```typescript
// Para demos en testnet — emitir un "PENPE" que simule soles peruanos
const PEN_TESTNET = new StellarSdk.Asset("PENPE", emisorPubKey);
// Flujo: 1) usuario "deposita" PEN simulado (Yape/Plin) → 2) se emite PENPE en testnet
// → 3) el usuario transfiere PENPE → 4) se quema en el off-ramp
```

### Remesas con USDC en Stellar testnet

```typescript
const USDC_TESTNET = new StellarSdk.Asset("USDC", "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5");
// Flujo: remitente (EE.UU.) envía USDC → transferencia instantánea a < $0.01
// → receptor (Perú) hace off-ramp a PEN vía anchor
```

---

## Herramientas útiles

```bash
curl https://horizon-testnet.stellar.org/accounts/<PUBLIC_KEY>
curl "https://horizon-testnet.stellar.org/assets?asset_code=PENPE&asset_issuer=<ISSUER>"
```

Stellar Laboratory (https://lab.stellar.org) — crear trustlines y pagos manualmente sin código.
