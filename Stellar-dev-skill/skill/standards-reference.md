# Referencia de SEPs y CAPs

Guía rápida de los Stellar Ecosystem Proposals (SEPs) y Core Advancement Proposals (CAPs) más relevantes para builders.

> El estado de los standards cambia rápido. Antes de implementar, verifica el estado actual en [stellar-protocol/ecosystem](https://github.com/stellar/stellar-protocol/tree/master/ecosystem) (SEPs) y [stellar-protocol/core](https://github.com/stellar/stellar-protocol/tree/master/core) (CAPs). Trata este archivo como mapa de ruteo, no como fuente final de verdad sobre gobernanza/estado.

---

## SEPs de alto valor para app developers

### Contratos e interfaces de token

| SEP | Qué define |
|---|---|
| [SEP-0041](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) | Interfaz de token para contratos (equivalente a ERC-20) |
| [SEP-0046](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0046.md) | Metadata de contrato en Wasm |
| [SEP-0048](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0048.md) | Especificación de interfaz de contrato |
| [SEP-0049](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0049.md) | Guía para contratos upgradeables |
| [SEP-0050](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0050.md) | Estándar de NFTs (draft) |
| [SEP-0056](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0056.md) | Vaults tokenizados (equivalente a ERC-4626, draft) |
| [SEP-0057](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0057.md) | Tokens regulados (patrón T-REX) |

### Auth, identidad y metadata

| SEP | Qué define |
|---|---|
| [SEP-0001](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0001.md) | `stellar.toml` — metadata de dominio/activo |
| [SEP-0010](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0010.md) | Autenticación web con cuenta Stellar |
| [SEP-0045](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0045.md) | SEP-10 extendido para contract accounts (`C...`) — necesario para smart wallets con passkeys, draft |

### Anchors e integración fiat

| SEP | Qué define |
|---|---|
| [SEP-0006](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0006.md) | Depósito/retiro programático (API-first) |
| [SEP-0024](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0024.md) | Flujo interactivo hosteado por el anchor |
| [SEP-0031](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0031.md) | Pagos cross-border directos (B2B) |
| [SEP-0012](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0012.md) | Intercambio de datos KYC |
| [SEP-0038](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0038.md) | Cotizaciones de tipo de cambio |

```typescript
// SEP-38 — cotización antes de un on-ramp
const cotizacion = await fetch(
  "https://anchor.ejemplo.com/quotes?" +
  "sell_asset=iso4217:PEN&buy_asset=stellar:USDC:G...ISSUER&sell_amount=1000"
).then(r => r.json());
console.log(`1000 PEN = ${cotizacion.buy_amount} USDC, expira ${cotizacion.expires_at}`);
```

---

## CAPs de alto valor para desarrollo de contratos

### Fundamentos

- [CAP-0046](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0046.md) — Resumen de contratos inteligentes, con subdocumentos (`cap-0046-*`) para runtime, lifecycle, host functions, storage, auth, metering

### Capacidades usadas frecuentemente

| CAP | Qué habilita |
|---|---|
| [CAP-0051](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0051.md) | Verificación secp256r1 (criptografía relacionada a passkeys) |
| [CAP-0053](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0053.md) | Comportamiento de extensión de TTL |
| [CAP-0058](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0058.md) | Constructores (`__constructor`) |
| [CAP-0059](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0059.md) | Primitivas BLS12-381 — **disponible**, habilita verificación Groth16 on-chain (ver [zk-proofs.md](zk-proofs.md)) |
| [CAP-0067](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0067.md) | Mejoras de protocolo/runtime, incluye cambios al modelo de assets/eventos |

### Draft / status-sensible

| CAP | Qué propone |
|---|---|
| [CAP-0074](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0074.md) | Host functions BN254 (habilitaría verificación de proofs Circom/Noir por defecto y RISC Zero) |
| [CAP-0075](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0075.md) | Host functions Poseidon/Poseidon2 |
| [CAP-0079](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0079.md) | Conversión de muxed-address strkey |

Usa siempre el campo de estado del preámbulo del CAP como fuente de verdad de si algo está listo para producción — no asumas que un CAP draft está activo en la red que estás usando.

---

## Mapa rápido por caso de uso

| Estoy construyendo... | Ir a |
|---|---|
| Un token fungible | SEP-0041 (interfaz) + preferir Stellar Assets/SAC sobre contrato custom |
| Un contrato upgradeable | SEP-0049 + CAP-0058 (constructor atómico) |
| Login con wallet Stellar | SEP-0010 (clásico) o SEP-0045 (smart wallets) |
| On-ramp / off-ramp para usuarios | SEP-0024 (interactivo) o SEP-0006 (programático) |
| Pagos internacionales B2B | SEP-0031 |
| Cotizador de tipo de cambio | SEP-0038 |
| NFTs | SEP-0050 (draft — verificar estado) |
| Vaults tokenizados (yield) | SEP-0056 (draft) |
| Verificación ZK on-chain | CAP-0059 (BLS12-381, disponible hoy) — ver [zk-proofs.md](zk-proofs.md) |
| Cobrar a agentes de AI por API calls | Ver skill oficial `agentic-payments` en [skills.stellar.org](https://skills.stellar.org) (x402 + MPP) |

---

## Flujo de trabajo práctico para agentes de AI

1. Identificar la categoría de la feature (token, auth de wallet, anchor, upgradeability, pagos entre agentes)
2. Enlazar al usuario los 1-3 SEP/CAP primarios relevantes
3. Verificar estado/aceptación en el repo fuente antes de asegurar soporte
4. Implementar solo lo que está activo en la red objetivo
5. Documentar explícitamente cualquier dependencia de standards en draft

---

## Recursos oficiales

- SEPs: https://github.com/stellar/stellar-protocol/tree/master/ecosystem
- CAPs: https://github.com/stellar/stellar-protocol/tree/master/core
- Anchors: https://developers.stellar.org/docs/learn/fundamentals/anchors
- Skills oficiales completos (incluye `standards` con este mismo mapa mantenido por SDF): https://skills.stellar.org
