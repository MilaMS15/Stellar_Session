# Recursos para Builders

Repos de referencia, documentación y herramientas útiles para construir en Stellar.

---

## Stellar Skills — contexto para tu agente de AI

[skills.stellar.org](https://skills.stellar.org) es el directorio oficial de Stellar Development Foundation con skills modulares para que cualquier agente de AI (Claude Code, Cursor, etc.) tenga contexto actualizado de Stellar antes de escribir código. Funciona con cualquier agente — solo dile:

```
Read skills.stellar.org before you start building on Stellar.
```

### Skills oficiales

| Skill | Link | Cubre |
|---|---|---|
| **Stellar Smart Contracts** | `skills.stellar.org/skills/smart-contracts/SKILL.md` | Escribir, testear, asegurar y deployar contratos Rust en Soroban — patrones, errores comunes, arquitectura |
| **Frontend & Wallets** | `skills.stellar.org/skills/dapp/SKILL.md` | dApps con JS SDK, Freighter, Wallets Kit, passkey smart accounts |
| **Stellar Assets & SAC** | `skills.stellar.org/skills/assets/SKILL.md` | Activos clásicos, trustlines, y el puente SAC para interop con contratos |
| **RPC & Horizon APIs** | `skills.stellar.org/skills/data/SKILL.md` | Consultar datos on-chain con RPC (preferido) y Horizon (legacy) — streaming, indexing, migración |
| **Agent Payments (x402 + MPP)** | `skills.stellar.org/skills/agentic-payments/SKILL.md` | Cobrar a agentes de AI por llamadas API con paywalls x402 o canales de pago MPP |
| **ZK Proofs** | `skills.stellar.org/skills/zk-proofs/SKILL.md` | Verificar pruebas Groth16 on-chain vía BLS12-381 — Circom, Noir, RISC Zero |
| **SEPs, CAPs & Ecosystem** | `skills.stellar.org/skills/standards/SKILL.md` | Elegir el SEP/CAP correcto, proyectos del ecosistema, links de referencia, MCPs |

Estos skills viven en el repo [stellar/stellar-dev-skill](https://github.com/stellar/stellar-dev-skill) (Apache-2.0), mantenido por SDF. Formas de instalar:

```bash
# Claude Code — vía plugin marketplace (recomendado)
/plugin marketplace add stellar/stellar-dev-skill
/plugin install stellar-dev@stellar-dev

# Cualquier agente compatible con Agent Skills — vía npx skills
npx skills add https://github.com/stellar/stellar-dev-skill

# Manual — clonar y copiar el directorio skills/ a la ruta de tu agente
git clone https://github.com/stellar/stellar-dev-skill
# Claude Code:  ~/.claude/skills/
# OpenCode:     ~/.config/opencode/skill/
# OpenAI Codex: ~/.codex/skills/
```

### Raven — MCP para lookups en vivo

[Raven](https://github.com/kalepail/stellar-raven) es un servidor MCP remoto que complementa los skills: mientras los skills le dan a tu agente contexto duradero (best practices que no cambian seguido), Raven busca en la documentación de Stellar y datos del ecosistema en tiempo real, y cruza todo en una sola respuesta. Usa ambos juntos.

```bash
claude mcp add --transport http stellar-raven "https://raven.stellar.buzz/mcp"
```

También hay un [playground en el navegador](https://raven.stellar.buzz) para probarlo sin instalar nada (requiere sign-in).

> Nuestro propio [Stellar-dev-skill/](./Stellar-dev-skill/) sigue disponible como complemento con contexto peruano — instálalo junto a los skills oficiales, no en su reemplazo. Ver [Herramientas_AI.md](./Herramientas_AI.md) para detalles.

### Skills de comunidad (selección relevante)

| Skill | Repo | Qué aporta |
|---|---|---|
| OpenZeppelin Contracts | `OpenZeppelin/openzeppelin-skills` | Scaffolding con librerías auditadas de OZ para Stellar — pausable, ownable |
| DeFindex SDK | `paltalabs/defindex-sdk` | Integrar vaults de yield con `@defindex/sdk` |
| Soroswap SDK | `soroswap/sdk` | Swaps, liquidez y price feeds con `@soroswap/sdk` |
| Trustless Work Escrow | `Trustless-Work/trustless-work-dev-skill` | Escrow y pagos por milestones — REST API, React SDK, Blocks UI |
| Anchors | `CheesecakeLabs/stellar-anchor-skill` | Integrar anchors (on/off ramps fiat, KYC) — SEP-1/6/10/12/24/31/38 |
| Stellar Builder Quickstart | `lumenloop/lumenloop-skills` | De una idea a un build path — primitivas, prior art del ecosistema |

> Los skills de comunidad no están auditados ni mantenidos por SDF. Revisa el código y la licencia antes de instalarlos.

---

## Documentación oficial

| Recurso | Link | Para qué |
|---|---|---|
| Stellar Docs | https://developers.stellar.org | Documentación completa de Stellar |
| Soroban Docs | https://soroban.stellar.org | Smart contracts en Stellar |
| Stellar SDK (JS) | https://stellar.github.io/js-stellar-sdk/ | SDK para frontend/backend |
| Stellar Laboratory | https://laboratory.stellar.org | Herramienta interactiva para testear operaciones |
| Freighter API | https://docs.freighter.app | Integración con wallet Freighter |

---

## Proyectos en Stellar — Para inspirarte

| Recurso | Link | Para qué |
|---|---|---|
| **Stellar Ecosystem** | https://stellar.org/ecosystem | Directorio de proyectos y apps construidas sobre Stellar |
| **Community Fund Projects** | https://communityfund.stellar.org/projects | Proyectos financiados por la comunidad Stellar — casos de uso reales |

---

## Protocolos DeFi en Stellar

### Blend Protocol — Lending/Borrowing
- **Qué es:** Protocolo de préstamos descentralizado en Stellar
- **Docs:** https://docs.blend.capital
- **Útil para:** Proyectos de DeFi, yield, credit scoring

### Soroswap — DEX Aggregator
- **Qué es:** Agregador de exchanges descentralizados en Soroban
- **Docs:** https://docs.soroswap.finance
- **Útil para:** Swaps, liquidez, price feeds

### Phoenix DEX
- **Qué es:** Exchange descentralizado nativo de Soroban
- **Útil para:** Trading, market making

### DeFindex — Yield Vaults
- **Qué es:** Gestión de vaults y estrategias DeFi
- **Útil para:** Automatización de yield, gestión de portfolios

### Reflector Oracle
- **Qué es:** Oráculo de precios para Soroban
- **Útil para:** Cualquier proyecto que necesite datos de precios on-chain

### Trustless Work — Escrow as a Service
- **Qué es:** Escrow descentralizado sobre Stellar
- **Docs:** https://docs.trustlesswork.com
- **Útil para:** Marketplaces, freelancing, pagos condicionales

### Lendara Protocol — Inversiones Tokenizadas
- **Qué es:** Protocolo de inversión reutilizable sobre Stellar/Soroban que usa Trustless Work como capa de escrow. Permite crear pools de inversión tokenizados para distintos sectores (inmobiliaria, comercio, agricultura, microfinanzas, eventos).
- **SDK:** `npm install @lendara/sdk`
- **Docs:** https://lendaraprotocol.gitbook.io/lendara
- **Guía completa:** [Lendara_Protocol.md](./Lendara_Protocol.md)
- **Útil para:** Tokenización de activos reales, crowdfunding, pools de inversión, DeFi con activos del mundo real

---

## Anchors y On/Off Ramps

| Servicio | Región | Notas |
|---|---|---|
| MoneyGram Access | Global | Cash in/out vía MoneyGram |
| Beans App | LatAm | Wallet con soporte Stellar |

> **Nota para Perú:** El ecosistema de anchors directos PEN↔Stellar está en desarrollo. Para prototipar, usa testnet y simula el flujo de on/off ramp. Plataformas como Bitso, Buda.com o Binance P2P permiten comprar XLM/USDC pero no son anchors SEP-24 formales. Yape y Plin pueden usarse como punto de entrada simulado para demos de on-ramp.

---

## Repos de ejemplo y starters

### Contratos Soroban
- **soroban-examples** — Ejemplos oficiales · https://github.com/stellar/soroban-examples
- **OpenZeppelin Stellar** — Contratos seguros para Soroban · [Docs](https://docs.openzeppelin.com/stellar-contracts) · [GitHub](https://github.com/OpenZeppelin/stellar-contracts) · [Wizard](https://wizard.openzeppelin.com/stellar)

### Frontend
- **Starter con Next.js + Stellar SDK** — Template básico de app web con Stellar
- **Soroban React Dapp** — Ejemplo de dApp React con integración Soroban

### Full Stack
- **Stellar Demo Wallet** — Wallet de referencia que implementa SEPs

---

## Herramientas de desarrollo

| Herramienta | Link | Uso |
|---|---|---|
| Stellar Expert | https://stellar.expert | Block explorer para testnet y mainnet |
| Friendbot | https://friendbot.stellar.org | Fondear cuentas en testnet |
| Stellar CLI | https://developers.stellar.org/docs/tools/developer-tools/cli | Compilar, deployar y llamar contratos |

---

## Checklist de recursos

- [ ] Leí las instrucciones de skills.stellar.org y las apliqué a mi agente de AI
- [ ] Leí la documentación de Soroban (al menos el quickstart)
- [ ] Probé Stellar Laboratory para entender las operaciones
- [ ] Me uní al Discord de Stellar para soporte de la comunidad
- [ ] Identifiqué qué protocolos DeFi son relevantes para mi proyecto
- [ ] Tengo los repos de ejemplo clonados por si necesito referencia offline
