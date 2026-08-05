# Ecosistema Stellar — Protocolos, Wallets y Herramientas

Guía del ecosistema actual de Stellar: protocolos DeFi, wallets, herramientas de desarrollo, indexers y proyectos de referencia.

> **Directorios canónicos** — para las listas más actualizadas: [Stellar Ecosystem](https://stellar.org/ecosystem) (directorio oficial, buscable por país/activo/categoría), [SCF Projects](https://communityfund.stellar.org/projects) (proyectos financiados con status), [Stellar en DefiLlama](https://defillama.com/chain/stellar) (TVL en vivo). Trata métricas/status de proyectos como volátiles.

---

## Protocolos DeFi

### Blend Protocol — Lending/Borrowing
- **Qué hace:** Protocolo de liquidez permissionless — pools de lending/borrowing
- **Docs:** https://docs.blend.capital · **GitHub:** https://github.com/blend-capital/blend-contracts-v2
- **Relevante para Perú:** préstamos sin banco, colateralización de activos tokenizados

### Soroswap — DEX y agregador
- **Qué hace:** Primer DEX + agregador en contratos de Stellar, rutea entre Aqua, Phoenix y el DEX clásico
- **Docs:** https://docs.soroswap.finance

### Aquarius / AQUA Network
- **Qué hace:** Capa de liquidez gobernada por votación con AMM
- **Web:** https://aqua.network

### Phoenix Protocol
- **Qué hace:** AMM nativo de contratos en Stellar
- **GitHub:** https://github.com/Phoenix-Protocol-Group

### DeFindex — Yield Vaults
- **Qué hace:** Vaults y estrategias de yield automatizadas, con integración a Blend
- **Docs:** https://docs.defindex.io

### Orbit CDP Protocol
- **Qué hace:** Stablecoins colateralizadas (USD, EUR, MXN) contra XLM/otros
- **Docs:** https://docs.orbitcdp.finance

### Reflector Network — Oráculo de precios
- **Qué hace:** Oráculo de precios comunitario, compatible SEP-40
- **Docs:** https://developers.stellar.org/docs/data/oracles/oracle-providers
- **Integraciones:** Blend, OrbitCDP, DeFindex

### Trustless Work — Escrow como servicio
- **Qué hace:** Escrow programático sobre Stellar — REST API, SDK React, componentes UI
- **Docs:** https://docs.trustlesswork.com
- **Relevante para Perú:** pagos seguros en trabajo remoto/freelance

---

## Wallets

### Freighter — Extensión de referencia de SDF
- **Instalación:** https://freighter.app · **Docs:** https://docs.freighter.app
- **API:** `@stellar/freighter-api` — usa `getAddress()`, no el deprecado `getPublicKey()`

```typescript
import { isConnected, getAddress, signTransaction } from "@stellar/freighter-api";

const { isConnected: instalado } = await isConnected();
const { address } = await getAddress();
const { signedTxXdr } = await signTransaction(txXdr, {
  networkPassphrase: "Test SDF Network ; September 2015",
});
```

### LOBSTR
- App móvil (iOS/Android) + web, la wallet más usada por usuarios finales. Soporta DEX trading, multisig, SEP-10.

### xBull / Rabet / Hana Wallet
- Extensiones alternativas de navegador, todas integrables vía Stellar Wallets Kit.

### Stellar Wallets Kit — integración multi-wallet (v2)

```bash
npx jsr add @creit-tech/stellar-wallets-kit
```

```typescript
import { StellarWalletsKit, Networks } from "@creit-tech/stellar-wallets-kit";
import { defaultModules } from "@creit-tech/stellar-wallets-kit/modules/utils";

StellarWalletsKit.init({ modules: defaultModules(), network: Networks.TESTNET });
const { address } = await StellarWalletsKit.authModal();
```

Soporta Freighter, LOBSTR, xBull, Albedo, Rabet, Hana, Ledger, Trezor, WalletConnect. **v2 es estático y vive en JSR** (`@creit-tech/...`), no en npm (`@creit.tech/...` era v1).

### Smart Accounts con passkeys

Autenticación WebAuthn (huella, Face ID), sin extensión de navegador:

- **Smart Account Kit** (recomendado, activo): https://github.com/kalepail/smart-account-kit — construido sobre [OpenZeppelin Stellar Contracts](https://github.com/OpenZeppelin/stellar-contracts)
- **Passkey Kit** (legacy): https://github.com/kalepail/passkey-kit — usar Smart Account Kit para proyectos nuevos
- **OpenZeppelin Relayer**: fee sponsorship / transacciones sin gas — https://docs.openzeppelin.com/relayer

---

## Herramientas de desarrollo

### Stellar CLI

```bash
cargo install --locked stellar-cli
# o: curl -fsSL https://github.com/stellar/stellar-cli/raw/main/install.sh | sh

stellar contract build          # compilar
stellar contract deploy         # desplegar
stellar contract invoke         # invocar función
stellar contract bindings       # generar bindings TypeScript
stellar keys generate           # crear keypair
stellar network add             # configurar red
```

> El CLI `soroban` (sin el prefijo `stellar`) es obsoleto — todos los comandos migraron a `stellar`.

### Scaffold Stellar
- **Qué hace:** scaffolding full-stack (contratos + frontend React/Vite/TS)
- **Web:** https://scaffoldstellar.org · **GitHub:** https://github.com/theahaco/scaffold-stellar

### Stellar Laboratory
- **URL:** https://lab.stellar.org — crear y enviar transacciones manualmente, explorar XDR

### Stellar Expert
- **URL:** https://stellar.expert — block explorer completo, testnet y mainnet

### Friendbot
- **Testnet:** https://friendbot.stellar.org?addr=<PUBLIC_KEY> — fondea cuentas de prueba gratis

---

## Indexers y datos

Para queries complejas más allá de RPC/Horizon:

| Indexer | Qué ofrece |
|---|---|
| **Mercury** (https://mercurydata.app) | Retroshades, GraphQL, ejecución serverless al cierre de ledger (Zephyr VM) |
| **SubQuery** (https://subquery.network) | Multi-chain, handlers de bloque/tx/operación/evento |
| **Goldsky** (https://goldsky.com) | Pipelines en tiempo real, subgraphs |
| **StellarExpert API** (https://stellar.expert/openapi.html) | REST gratis, sin auth |

Directorio completo: https://developers.stellar.org/docs/data/indexers

---

## AI y MCP

### Raven — MCP para lookups en vivo
Servidor MCP remoto que busca en documentación de Stellar y datos del ecosistema en tiempo real, cruzando todo en una sola respuesta. Complementa a los skills estáticos (que dan contexto duradero de best practices).

- **Endpoint MCP:** https://raven.stellar.buzz/mcp · **Playground:** https://raven.stellar.buzz/playground
- **GitHub:** https://github.com/kalepail/stellar-raven
- **Conectar en Claude Code:** `claude mcp add --transport http stellar-raven "https://raven.stellar.buzz/mcp"`

### Stellar Skills (skills.stellar.org)
Directorio oficial de skills modulares para agentes de AI — smart contracts, dApp, assets, data, agentic payments, zk-proofs, standards. Ver [resources.md](resources.md) para instalación.

---

## Librerías y SDKs

### JavaScript / TypeScript

| Paquete | Uso |
|---|---|
| `@stellar/stellar-sdk` | SDK principal — transacciones, RPC, Horizon, `contract.Client` |
| `@stellar/freighter-api` | Solo Freighter |
| `@creit-tech/stellar-wallets-kit` (JSR) | Multi-wallet, v2 |
| `smart-account-kit` | Smart wallets con passkeys |
| `@blend-capital/blend-sdk` | Integración con Blend |

### Rust (contratos)

| Crate | Uso |
|---|---|
| `soroban-sdk` | SDK base — incluye `soroban_sdk::token::TokenClient`, no hace falta un crate de token aparte |
| `stellar-tokens`, `stellar-access`, `stellar-contract-utils`, `stellar-macros` | OpenZeppelin Stellar Contracts — tokens, ownable, pausable, upgradeable |

### Python

```bash
pip install stellar-sdk
```

```python
from stellar_sdk import Server, Keypair, TransactionBuilder, Network, Asset

server = Server("https://horizon-testnet.stellar.org")
keypair = Keypair.from_secret("S...")
account = server.load_account(keypair.public_key)

tx = (
    TransactionBuilder(source_account=account, network_passphrase=Network.TESTNET_NETWORK_PASSPHRASE, base_fee=100)
    .append_payment_op(destination="G...", asset=Asset.native(), amount="10")
    .set_timeout(30)
    .build()
)
tx.sign(keypair)
response = server.submit_transaction(tx)
```

---

## Proyectos de referencia en GitHub

| Proyecto | Descripción |
|---|---|
| `stellar/soroban-examples` | Ejemplos oficiales de contratos (incluye `groth16_verifier`, atomic swap, timelock) |
| `OpenZeppelin/stellar-contracts` | Contratos auditados y seguros |
| `stellar/soroban-example-dapp` | dApp de crowdfunding con frontend Next.js |
| `kalepail/smart-account-kit` | Smart wallets con passkeys |
| `stellar/stellar-mpp-sdk` | Machine Payments Protocol (Charge + Channel) |

---

## Canales de la comunidad

| Canal | Para qué |
|---|---|
| [Discord de Stellar](https://discord.gg/stellar) | Soporte técnico, comunidad general |
| [Stellar Stack Exchange](https://stellar.stackexchange.com) | Preguntas técnicas indexadas |
| [GitHub Discussions](https://github.com/stellar/stellar-protocol/discussions) | Propuestas y discusión de protocolo |
| [SDF Blog](https://stellar.org/blog) | Anuncios oficiales |

---

## Contexto del ecosistema para Perú y LATAM

### Proyectos relevantes activos

- **Lendara Protocol** — Inversiones tokenizadas para LATAM (inmobiliaria, comercio, agricultura) — ver [Lendara_Protocol.md](../../Lendara_Protocol.md) en la raíz de este repo
- **Bitso** — Exchange con soporte XLM/USDC en México, Argentina, Brasil
- **Buda.com** — Exchange en Chile, Colombia, Perú — compra de XLM con PEN
- **Beans App** — Wallet con on/off ramp para LATAM

### Casos de uso con tracción en LATAM

1. **Remesas** — Envío de dinero desde EE.UU., España, Chile a Perú con USDC
2. **Pagos freelance** — Cobro en USDC sin cuenta bancaria en dólares
3. **Microfinanzas** — Préstamos colateralizados sin historial crediticio tradicional
4. **Tokenización de activos** — Fraccionamiento de bienes raíces o activos productivos
5. **Ahorro en dólares** — USDC como cobertura frente a depreciación del PEN
