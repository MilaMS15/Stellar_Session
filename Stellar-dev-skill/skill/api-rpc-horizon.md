# APIs de Stellar: RPC y Horizon

Acceso a datos de la red Stellar. Stellar RPC es el punto de entrada preferido para proyectos nuevos; Horizon queda para legacy e historial. Para historial más allá de la ventana de 7 días de RPC, usar Hubble/Galexie.

---

## Cuándo usar cada API

| API | Estado | Caso de uso |
|---|---|---|
| **Stellar RPC** | Preferido | Contratos, estado en tiempo real, proyectos nuevos |
| **Horizon** | Legacy-focused | Datos históricos, aplicaciones legacy |

**Recomendación:** usa RPC para todo proyecto nuevo. Usa Horizon principalmente para consultas históricas y compatibilidad legacy.

---

## Stellar RPC

### URLs

> SDF solo provee RPC público directo para Futurenet. Para mainnet, elige un proveedor del [directorio de RPC](https://developers.stellar.org/docs/data/apis/rpc/providers) — **no existe** un endpoint mainnet hosteado por SDF (`soroban-mainnet.stellar.org` no existe).

| Red | RPC URL |
|---|---|
| Mainnet | Endpoint de proveedor (ver directorio de RPC) |
| Testnet | `https://soroban-testnet.stellar.org` |
| Futurenet | `https://rpc-futurenet.stellar.org` |
| Local | `http://localhost:8000/soroban/rpc` |

### Setup

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

const rpc = new StellarSdk.rpc.Server("https://soroban-testnet.stellar.org");
```

> El namespace es `StellarSdk.rpc` (SDK v16+). Código con `SorobanRpc.Server` es de una versión anterior del SDK.

### Métodos principales

```typescript
// Cuenta (retorna solo lo necesario para construir transacciones)
const cuenta = await rpc.getAccount("G...");

// Health y ledger actual
const health = await rpc.getHealth();
const ledger = await rpc.getLatestLedger();

// Leer storage de contrato
const key = StellarSdk.xdr.LedgerKey.contractData(
  new StellarSdk.xdr.LedgerKeyContractData({
    contract: new StellarSdk.Address(contractId).toScAddress(),
    key: StellarSdk.xdr.ScVal.scvSymbol("mi_clave"),
    durability: StellarSdk.xdr.ContractDataDurability.persistent(),
  })
);
const entradas = await rpc.getLedgerEntries(key);

// Simular transacción (sin enviar)
const sim = await rpc.simulateTransaction(tx);
if (StellarSdk.rpc.Api.isSimulationError(sim)) {
  console.error("Simulación falló:", sim.error);
} else if (StellarSdk.rpc.Api.isSimulationSuccess(sim)) {
  console.log("Costo:", sim.cost, "Resultado:", sim.result);
}

// Enviar
const response = await rpc.sendTransaction(signedTransaction);

// Esperar resultado (usa el poller del SDK, no un while loop manual)
const result = await rpc.pollTransaction(response.hash);
// result.status: SUCCESS | FAILED | NOT_FOUND
```

### Atajos de lectura sin cliente manual

```typescript
// Ejecuta un método de solo lectura y devuelve el resultado ya decodificado —
// no necesitas armar ScVal a mano ni configurar un contract.Client
const { result: balance } = await rpc.queryContract<bigint>(tokenId, "balance", { id: "G..." });

// Descubre los métodos invocables de un contrato solo con su ID
const methods = await rpc.getContractMethods(tokenId);
```

### Eventos de contrato

```typescript
const eventos = await rpc.getEvents({
  startLedger: ledger.sequence - 1000,
  filters: [{
    type: "contract",
    contractIds: [contractId],
    topics: [["*", StellarSdk.xdr.ScVal.scvSymbol("transfer").toXDR("base64")]],
  }],
  limit: 100,
});

for (const evento of eventos.events) {
  console.log(evento.ledger, evento.topic.map(StellarSdk.scValToNative), StellarSdk.scValToNative(evento.value));
}
```

### Limitaciones de RPC

- **Historial de 7 días** para la mayoría de métodos (`getTransaction`, `getEvents`, etc.)
- **Excepción `getLedgers`**: "Infinite Scroll" consulta cualquier ledger hasta génesis vía el data lake
- **Sin streaming** — hay que hacer polling (no WebSocket)
- **Enfocado en contratos** — datos limitados de Stellar clásico

---

## Horizon API

### URLs

| Red | Horizon URL |
|---|---|
| Mainnet | `https://horizon.stellar.org` |
| Testnet | `https://horizon-testnet.stellar.org` |
| Local | `http://localhost:8000` |

### Uso básico

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

const horizon = new StellarSdk.Horizon.Server("https://horizon-testnet.stellar.org");

const cuenta = await horizon.loadAccount("G...");
console.log("Saldos:", cuenta.balances);

const txs = await horizon.transactions().forAccount("G...").order("desc").limit(10).call();

const pagos = await horizon.payments().forAccount("G...").order("desc").limit(20).call();

const orderbook = await horizon.orderbook(new StellarSdk.Asset("USDC", "G...ISSUER"), StellarSdk.Asset.native()).call();
```

### Streaming (Server-Sent Events)

```typescript
const cerrar = horizon.payments().forAccount("G...").cursor("now").stream({
  onmessage: (pago) => console.log("Nuevo pago:", pago),
  onerror: (error) => console.error("Error en stream:", error),
});
// cerrar() para detener
```

### HTTP directo

```bash
curl https://horizon-testnet.stellar.org/accounts/G...
curl "https://horizon-testnet.stellar.org/accounts/G.../transactions?order=desc&limit=10"
curl "https://horizon-testnet.stellar.org/assets?asset_code=USDC&asset_issuer=G..."
```

### Paginación

```typescript
let page = await server.transactions().forAccount(publicKey).limit(10).call();
if (page.records.length > 0) page = await page.next();
page = await page.prev();
```

---

## Migración: Horizon → RPC

```typescript
// Cuenta
const account = await rpc.getAccount(publicKey); // retorna menos que loadAccount, suficiente para tx

// Envío (Soroban)
const response = await rpc.sendTransaction(tx);
const result = await rpc.pollTransaction(response.hash);

// Streaming — no existe en RPC, hay que hacer polling
async function pollForUpdates() {
  const lastLedger = await rpc.getLatestLedger();
  // consultar eventos/transacciones nuevas
}
setInterval(pollForUpdates, 5000);
```

---

## Historial más allá de 7 días

- **Hubble** — dataset BigQuery de SDF, actualizado cada 30 min
- **Galexie** — pipeline de datos self-hosted ([GitHub](https://github.com/stellar/galexie))
- **Data Lake** — object store en la nube que alimenta "Infinite Scroll" de RPC; acceso público vía `s3://aws-public-blockchain/v1.1/stellar/ledgers/pubnet` (AWS Open Data)

### Indexers de terceros

Para queries complejas, streaming de eventos o pipelines custom:

- **Mercury** — indexer nativo de Stellar, Retroshades, API GraphQL (https://mercurydata.app)
- **SubQuery** — indexer multi-chain con soporte Stellar (https://subquery.network)
- **Goldsky** — pipelines de replicación en tiempo real (https://goldsky.com)
- **StellarExpert API** — REST gratis sin auth para assets, cuentas, ledgers (https://stellar.expert/openapi.html)

---

## Manejo de errores

```typescript
// RPC
try {
  const resultado = await rpc.sendTransaction(tx);
  if (resultado.status === "ERROR") console.error("Error de red:", resultado.errorResult);
} catch (e) {
  console.error("Error en simulación o envío:", e);
}

// Horizon
try {
  await horizon.submitTransaction(tx);
} catch (e: any) {
  const codes = e.response?.data?.extras?.result_codes;
  if (codes) {
    console.error("Transacción:", codes.transaction);
    console.error("Operaciones:", codes.operations);
  }
}
```

### Códigos de error comunes de Horizon

| Código | Significado |
|---|---|
| `tx_bad_seq` | Sequence number incorrecto — recargar cuenta |
| `tx_insufficient_fee` | Fee muy bajo — incrementar fee |
| `op_underfunded` | Saldo insuficiente |
| `op_no_trust` | Destinatario no tiene trustline |
| `op_line_full` | Trustline del destinatario está llena |
| `tx_too_late` | Expiró el timeout de la transacción |

---

## Fees dinámicos

```typescript
async function calcularFeeOptimo(rpc: StellarSdk.rpc.Server): Promise<string> {
  const stats = await rpc.getFeeStats();
  const feeRecomendado = parseInt(stats.sorobanInclusionFee.p90);
  return Math.ceil(feeRecomendado * 1.1).toString(); // margen 10%
}

// Para transacciones clásicas
const fee = (await horizon.fetchBaseFee()).toString();
```

---

## Comparación de respuestas

```typescript
// Horizon: id, sequence, balances[], signers[], flags, etc. — más rico
const cuentaHorizon = await horizon.loadAccount(pubKey);

// RPC: id, sequence — más limitado pero más rápido para contratos
const cuentaRpc = await rpc.getAccount(pubKey);

// Apps que solo usan contratos: RPC para todo.
// Apps que mezclan clásico + contratos: conectar ambas APIs.
```
