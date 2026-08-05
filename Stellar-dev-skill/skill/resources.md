# Recursos de referencia — Stellar

Links curados de documentación oficial, herramientas, comunidad y contexto peruano.

---

## Stellar Skills oficiales y Raven

| Recurso | URL | Para qué |
|---|---|---|
| Stellar Skills | https://skills.stellar.org | Directorio oficial de skills de SDF — instalar vía plugin marketplace de Claude Code o `npx skills` |
| Raven (MCP) | https://raven.stellar.buzz/mcp | Lookups en vivo sobre docs y datos del ecosistema — complementa a los skills estáticos |

```bash
# Skills oficiales — Claude Code
/plugin marketplace add stellar/stellar-dev-skill
/plugin install stellar-dev@stellar-dev

# Raven
claude mcp add --transport http stellar-raven "https://raven.stellar.buzz/mcp"
```

---

## Documentación oficial

| Recurso | URL | Para qué |
|---|---|---|
| Stellar Developers | https://developers.stellar.org | Punto de entrada principal |
| Build Smart Contracts | https://developers.stellar.org/docs/build/smart-contracts | Guías de contratos |
| Stellar SDK (JS) | https://stellar.github.io/js-stellar-sdk/ | Referencia API del SDK JS |
| Stellar SDK (Python) | https://stellar-sdk.readthedocs.io | Referencia API del SDK Python |
| Stellar Laboratory | https://lab.stellar.org | Herramienta interactiva |
| Freighter API | https://docs.freighter.app | Integración con wallet Freighter |
| Stellar Wallets Kit (v2) | https://github.com/Creit-Tech/Stellar-Wallets-Kit | Multi-wallet, JSR |

---

## Contratos — referencia técnica

| Recurso | URL |
|---|---|
| Getting Started | https://developers.stellar.org/docs/build/smart-contracts/getting-started |
| soroban-sdk (docs.rs) | https://docs.rs/soroban-sdk |
| Ejemplos de contratos | https://github.com/stellar/soroban-examples |
| OpenZeppelin Stellar Contracts | https://github.com/OpenZeppelin/stellar-contracts · [Wizard](https://wizard.openzeppelin.com/stellar) |
| Stellar CLI | https://developers.stellar.org/docs/tools/stellar-cli |
| RPC providers (mainnet) | https://developers.stellar.org/docs/data/apis/rpc/providers |

---

## Protocolos DeFi

| Protocolo | Docs |
|---|---|
| Blend Protocol | https://docs.blend.capital |
| Soroswap | https://docs.soroswap.finance |
| DeFindex | https://docs.defindex.io |
| Trustless Work | https://docs.trustlesswork.com |
| Lendara Protocol | https://lendaraprotocol.gitbook.io/lendara |

---

## Herramientas de desarrollo

| Herramienta | URL | Uso |
|---|---|---|
| Stellar Laboratory | https://lab.stellar.org | Test de transacciones |
| Stellar Expert | https://stellar.expert | Block explorer |
| Friendbot (testnet) | https://friendbot.stellar.org | Fondear cuentas de prueba |
| Scaffold Stellar | https://scaffoldstellar.org | Scaffolding full-stack |

---

## Estándares SEP (selección)

| SEP | Descripción | Link |
|---|---|---|
| SEP-1 | `stellar.toml` | [sep-0001](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0001.md) |
| SEP-10 | Auth web | [sep-0010](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0010.md) |
| SEP-24 | Transferencias interactivas | [sep-0024](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0024.md) |
| SEP-38 | Cotizaciones | [sep-0038](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0038.md) |
| SEP-41 | Token interface | [sep-0041](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) |
| SEP-45 | Web Auth para contract accounts | [sep-0045](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0045.md) |

Lista completa y mapa por caso de uso: [standards-reference.md](standards-reference.md).

---

## Repositorios de referencia

```
stellar/js-stellar-sdk           → SDK oficial JavaScript
stellar/py-stellar-base          → SDK oficial Python
stellar/go                       → SDK oficial Go
stellar/soroban-examples         → Contratos de ejemplo (incluye groth16_verifier)
stellar/stellar-cli               → CLI oficial
stellar/stellar-dev-skill        → Skills oficiales de AI (fuente de skills.stellar.org)
kalepail/smart-account-kit       → Smart wallets con passkeys
kalepail/stellar-raven           → MCP con lookups en vivo
OpenZeppelin/stellar-contracts   → Contratos auditados
```

---

## Comunidad y soporte

| Canal | URL | Para qué |
|---|---|---|
| Discord Stellar | https://discord.gg/stellar | Soporte técnico, comunidad |
| Stack Exchange | https://stellar.stackexchange.com | Preguntas con respuestas indexadas |
| SDF Blog | https://stellar.org/blog | Anuncios oficiales |

---

## Contexto para Perú

| Recurso | URL / Descripción |
|---|---|
| Buda.com | Exchange con soporte PEN — compra XLM con transferencia bancaria |
| Binance P2P | Compra USDC/XLM con Yape o transferencia bancaria |
| Stellar Anchor List | https://www.stellar.org/ecosystem/anchors — anchors verificados por SDF |
| Guía Stellar Build Peru | Este repositorio — contexto local para builders peruanos |

---

## Instalación rápida

```bash
# Rust + target WASM
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32v1-none   # Rust 1.84+ requerido

# Stellar CLI (reemplaza al antiguo "soroban" CLI)
cargo install --locked stellar-cli

# SDK JavaScript
npm install @stellar/stellar-sdk @stellar/freighter-api

# Stellar Wallets Kit v2 (JSR, no npm)
npx jsr add @creit-tech/stellar-wallets-kit

# Identidad de testnet, fondeada
stellar keys generate mi-wallet --network testnet --fund
```

---

## Sobre versiones

Este archivo evita fijar números de versión específicos a propósito — `soroban-sdk`, el CLI y el protocolo avanzan juntos y rápido. Antes de fijar una versión en `Cargo.toml` o `package.json`:

- `soroban-sdk`: la mayor sigue al protocolo. Última versión en [crates.io/crates/soroban-sdk](https://crates.io/crates/soroban-sdk).
- `@stellar/stellar-sdk`: última en [npmjs.com/package/@stellar/stellar-sdk](https://www.npmjs.com/package/@stellar/stellar-sdk) — requiere Node 22+ desde v16.
- Protocolo de red activo: `stellar network settings --network mainnet` o [Stellar Lab](https://lab.stellar.org).

> Para contexto siempre actualizado, prioriza [skills.stellar.org](https://skills.stellar.org) o [Raven](https://raven.stellar.buzz) sobre cualquier número fijado en este repositorio.
