
# Stellar Build Peru — Guía para Builders

Guía práctica para desarrolladores en Perú que quieren empezar a construir sobre **Stellar** — sin importar tu experiencia previa con la red, acceso a herramientas de AI pagas, o el hardware que tengas.

Nació como material de apoyo para el Buildathon Ethereum Lima × Stellar (mayo 2026), organizado por BAF y el programa StarMaker en colaboración con Ethereum Lima. Ese evento ya terminó, pero la guía sigue viva: la mantenemos como punto de entrada general para cualquier builder peruano que quiera iniciar en el ecosistema Stellar.

---

## Orden de lectura recomendado

| # | Archivo | Qué vas a encontrar |
|---|---|---|
| 1 | [SetUp_AI.md](./SetUp_AI.md) | Opciones de AI sin tarjeta de crédito ni suscripción |
| 2 | [Guia_Claude_Code.md](./Guia_Claude_Code.md) | Workflows con Claude Code: plan mode, agentes paralelos, CLAUDE.md |
| 3 | [SetUp_Dev.md](./SetUp_Dev.md) | Testnet, configuración de Soroban, Freighter, errores comunes |
| 4 | [Prompts_Iniciales.md](./Prompts_Iniciales.md) | Prompts listos para copiar y pegar en tu sesión de Claude Code |
| 5 | [Recursos.md](./Recursos.md) | Stellar Skills, repos de referencia, documentación oficial, herramientas |
| 6 | [Herramientas_AI.md](./Herramientas_AI.md) | Tools de AI para Stellar, asistentes de código, frameworks multi-agente |
| 7 | [Lendara_Protocol.md](./Lendara_Protocol.md) | Protocolo de inversiones tokenizadas — SDK listo para usar con 5 templates |

---

## Por dónde empezar

Si es tu primera vez construyendo sobre Stellar:

1. **Dale contexto a tu agente de AI.** Antes de escribir código, dile a tu asistente (Claude Code, Cursor, lo que uses) que lea `skills.stellar.org` — es el directorio oficial de skills de Stellar Development Foundation, con guías para smart contracts, dApps, assets, APIs, pagos entre agentes y más. Ver [Recursos.md](./Recursos.md#building-with-ai--contexto-para-tu-agente).
2. **Elige tu setup de AI.** No necesitas pagar nada — [SetUp_AI.md](./SetUp_AI.md) cubre las mejores opciones gratuitas, en la nube o locales.
3. **Configura tu entorno de desarrollo.** [SetUp_Dev.md](./SetUp_Dev.md) te lleva de cero a un contrato Soroban deployado en testnet.
4. **Define qué vas a construir.** Piensa en un problema real — remesas, pagos para emprendedores sin cuenta bancaria, trazabilidad de productos, identidad digital, micropréstamos. Estas ideas son fértiles para Perú porque atacan fricciones reales del sistema financiero local.
5. **Itera con prompts probados.** [Prompts_Iniciales.md](./Prompts_Iniciales.md) tiene plantillas para arrancar proyectos, debuggear, e integrar el contexto peruano (CCI, Yape/Plin, remesas).

---

## Proyectos de ejemplo incluidos

### VotaOnChain

Mini dApp de votación comunitaria on-chain — ejemplo de referencia para builders.

**Stack:** React + TypeScript + Tailwind · Soroban (contrato en Rust) · Stellar Wallets Kit · Neon PostgreSQL (opcional)

```
VotaOnChain/
├── contract/          # Contrato Soroban en Rust — votación on-chain con 4 tests
└── src/
    ├── components/    # WalletConnect · VoteForm · Results
    └── lib/           # stellar.ts (RPC) · kit.ts (wallets) · db.ts (Neon)
```

Flujo: el usuario conecta Freighter → vota → el voto se registra en Soroban → los resultados se leen en tiempo real desde la cadena.

Ver [VotaOnChain/README.md](./VotaOnChain/README.md) para setup completo y [VotaOnChain/GUIA.md](./VotaOnChain/GUIA.md) para el paso a paso detallado.

---

## Skill de AI — Stellar Development

Skill para asistentes de AI con conocimiento profundo del ecosistema Stellar y contexto peruano. Es un complemento al directorio oficial [skills.stellar.org](https://skills.stellar.org) (ver [Recursos.md](./Recursos.md#building-with-ai--contexto-para-tu-agente)) — instala ambos, no uno en lugar del otro.

```
Stellar-dev-skill/
└── skill/
    ├── SKILL.md                # Guía principal — arquitectura, conceptos core, contexto Perú
    ├── contracts-soroban.md    # Contratos Rust: storage, auth, deploy, upgrade, TTL
    ├── frontend-stellar-sdk.md # React/Next.js, Stellar Wallets Kit, bindings TypeScript
    ├── testing.md              # Tests unitarios, testnet, Docker Quickstart
    ├── stellar-assets.md       # Activos clásicos, SAC, emisión, trustlines
    ├── api-rpc-horizon.md      # RPC vs Horizon, streaming, fees, errores
    ├── security.md             # Checklists, require_auth, auditoría
    ├── common-pitfalls.md      # Errores frecuentes y soluciones
    ├── advanced-patterns.md    # Factory, vaults, multicall, escrow, circuit breaker
    ├── standards-reference.md  # SEP-1/10/24/38/41 con código
    ├── ecosystem.md            # Blend, Soroswap, wallets, herramientas, LATAM
    ├── zk-proofs.md            # ZK proofs en Soroban
    └── resources.md            # Links y documentación oficial
```

Basado en [stellar/stellar-dev-skill](https://github.com/stellar/Stellar-dev-skill), adaptado con contexto peruano.

**Instalación en Claude Code:**
```bash
cp -r Stellar-dev-skill/skill ~/.claude/skills/stellar-dev-pe/
```

Ver [Stellar-dev-skill/README.md](./Stellar-dev-skill/README.md) para más opciones de instalación.

---

## Rails de pago en Perú

A diferencia de otras guías enfocadas en SPEI (México) o PIX (Brasil), esta se enfoca en las opciones disponibles para Perú:

- **Transferencias bancarias (CCI):** El estándar local para on/off ramps — Código de Cuenta Interbancario usado por BCP, Interbank, BBVA, Scotiabank
- **Yape / Plin:** Billeteras móviles con adopción masiva en Perú, ideales para simular flujos de on-ramp en demos
- **USDC en Stellar:** El puente más directo entre crypto y fiat vía exchanges como Bitso, Buda.com y Binance P2P
- **Anchors Stellar:** Servicios que conectan la red Stellar con el sistema financiero peruano
- **Remesas entrantes:** Caso de uso muy relevante para Perú — muchas familias reciben envíos desde EE.UU., España y Chile

---

## Filosofía de esta guía

- **Práctica y opinada** — Te decimos qué funciona, qué rompe, y en qué orden hacer las cosas.
- **Accesible** — Todo lo que recomendamos tiene una opción gratuita.
- **Contextualizada** — Los ejemplos y casos de uso son relevantes para Perú y LATAM.
- **Bilingüe cuando importa** — Documentación en español, código y comandos en inglés (como corresponde).

---

## Origen

Nació como material de apoyo para el **Buildathon Ethereum Lima × Stellar** — armada por **Ethereum Lima · StarMaker by BAF · Stellar · Minted In Peru**. Hoy la mantenemos como guía general y abierta para la comunidad builder de Stellar en Perú.

---

## Comunidad

¿Preguntas, dudas, o querés conectar con otros builders de Stellar en Perú? Únete al [grupo de WhatsApp](https://chat.whatsapp.com/JB5clEPH9uK2iwzeHkMPOJ).

---

## Contribuir

Si encuentras un error o quieres agregar algo, abre un issue o un PR. Esta guía es por y para la comunidad.

---

## Licencia

MIT

---

> Inspirada en [stellar-guide-vendimia-tech](https://github.com/BuenDia-Builders/stellar-guide-vendimia-tech) (Argentina), que a su vez se inspiró en [stellar-ai-guide-mx](https://github.com/kaankacar/stellar-ai-guide-mx) (México). Adaptada para el contexto peruano.
