# Herramientas de AI Recomendadas

Tools de AI específicamente útiles para desarrollar sobre Stellar, asistentes de código, y frameworks para agentes.

## Asistentes de código

### Claude Code
- **Qué es:** Herramienta oficial de Anthropic para programar con Claude — disponible como CLI, desktop app, web (claude.ai/code) e IDE extensions (VS Code, JetBrains)
- **Por qué usarlo:** Plan mode, agentes paralelos, contexto persistente con CLAUDE.md, modelos Claude 4.x
- **Setup:** `npm install -g @anthropic-ai/claude-code`
- **Costo:** Free tier disponible
- **Ideal para:** Desarrollo completo en Stellar, de cero a producción (ver [Guia_Claude_Code.md](./Guia_Claude_Code.md))

### GitHub Copilot
- **Qué es:** Autocompletado de código integrado en VS Code
- **Por qué usarlo:** Sugerencias inline mientras escribís
- **Costo:** Gratis para estudiantes y open source
- **Ideal para:** Completar código boilerplate, escribir tests

### Cursor
- **Qué es:** Editor de código con AI integrada
- **Por qué usarlo:** Chat + edición directa en el editor
- **Costo:** Free tier limitado
- **Ideal para:** Si prefieres una interfaz visual sobre CLI

### Windsurf
- **Qué es:** Editor AI con "flows" para tareas complejas (adquirido por OpenAI en 2025)
- **Costo:** Free tier disponible
- **Ideal para:** Alternativa a Cursor — tener en cuenta la integración con el ecosistema OpenAI tras la adquisición

## Dale contexto a tu agente antes de escribir código

Lo primero que deberías hacer antes de escribir una línea de código en Stellar: darle contexto a tu agente. Stellar ofrece tres recursos complementarios — úsalos juntos, no son excluyentes. Detalle completo de los tres en [Recursos.md](./Recursos.md#building-with-ai--contexto-para-tu-agente):

- **Raven** (MCP, recomendado) — servidor remoto con lookups en vivo sobre docs + datos del ecosistema, con tools `search` y `execute`
- **Stellar Skills** — paquetes instalables con contexto duradero (best practices que no cambian seguido)
- **`llms.txt`** — archivo estático para pegar contexto directo en cualquier chat sin instalar nada

```
Read skills.stellar.org before you start building on Stellar.
```

| Skill | Link | Qué aporta |
|---|---|---|
| **Soroban Smart Contracts** (oficial) | `skills.stellar.org/skills/smart-contracts/SKILL.md` | Contratos Rust en Soroban — patrones, seguridad, arquitectura |
| **Frontend & Wallets** (oficial) | `skills.stellar.org/skills/dapp/SKILL.md` | JS SDK, Freighter, Wallets Kit, passkey smart accounts |
| **RPC & Horizon APIs** (oficial) | `skills.stellar.org/skills/data/SKILL.md` | Consultas on-chain, streaming, indexing |
| **Stellar Dev Skill** (este repo) | [Stellar-dev-skill/](./Stellar-dev-skill/) | Complemento con contexto peruano — testnet, patrones LATAM |
| **OpenZeppelin Skills** | https://github.com/OpenZeppelin/openzeppelin-skills | Skills oficiales de OpenZeppelin — seguridad, contratos, auditoría |

**Instalación de los skills oficiales de SDF ([stellar/stellar-dev-skill](https://github.com/stellar/stellar-dev-skill)):**
```bash
# Claude Code — plugin marketplace (agrega el skill de forma permanente)
/plugin marketplace add stellar/stellar-dev-skill
/plugin install stellar-dev@stellar-dev
```

**Instalación del Stellar Dev Skill de este repo (complemento con contexto peruano):**
```bash
cp -r Stellar-dev-skill/skill ~/.claude/skills/stellar-dev-pe/
```

**Instalación de OpenZeppelin Skills:**
```bash
git clone https://github.com/OpenZeppelin/openzeppelin-skills
cp -r openzeppelin-skills/skill ~/.claude/skills/openzeppelin/
```

**Instalador comunitario todo-en-uno** — [stellar-build](https://github.com/kaankacar/stellar-build) agrega 46 skills + personas de AI de una sola vez, cubriendo desde ideación hasta lanzamiento (proyecto de comunidad, no oficial):
```bash
curl -fsSL https://raw.githubusercontent.com/kaankacar/stellar-build/main/install.sh | bash
```

### Raven — MCP para lookups en vivo

Complemento a los skills: [Raven](https://github.com/kalepail/stellar-raven) es un servidor MCP remoto que expone dos tools — `search` (encuentra docs y datos del ecosistema relevantes) y `execute` (corre queries contra datos en vivo). Por debajo cruza Stellar Docs, Lumenloop, Stellar Light y los propios Stellar Skills en una sola respuesta. Los skills le dan a tu agente contexto duradero; Raven responde preguntas puntuales con información actual. Usa ambos.

```bash
claude mcp add --transport http stellar-raven "https://raven.stellar.buzz/mcp"
# luego: /mcp -> Authenticate -> iniciar sesión en el navegador
```

Probarlo sin instalar nada: [raven.stellar.buzz/playground](https://raven.stellar.buzz/playground).

---

## Herramientas Stellar-specific con AI

### Stellar Turrets
- Funciones serverless para operaciones Stellar
- Útil para lógica de negocio que interactúa con la red

### Soroban Copilot (comunidad)
- Prompts y templates específicos para desarrollo Soroban
- Patrones comunes pre-configurados

## Frameworks multi-agente

Si tu proyecto involucra agentes de AI que interactúan con blockchain:

### LangChain / LangGraph
- **Qué es:** Framework para construir aplicaciones con LLMs
- **Útil para:** Agentes que necesitan tool calling, RAG, memoria
- **Con Stellar:** Podés crear tools que consulten Horizon API o ejecuten transacciones

### CrewAI
- **Qué es:** Framework para orquestar múltiples agentes
- **Útil para:** Proyectos donde querés que varios agentes colaboren
- **Ejemplo:** Un agente analiza el mercado, otro ejecuta trades en Soroswap

### Vercel AI SDK
- **Qué es:** SDK para integrar AI en aplicaciones Next.js
- **Útil para:** Chatbots e interfaces conversacionales
- **Con Stellar:** Chat que permite consultar balances o enviar transacciones vía lenguaje natural

## Combinaciones recomendadas

| Tipo de proyecto | Stack sugerido |
|---|---|
| **DeFi** | Claude Code + Stellar SDK + Vercel AI SDK (UI conversacional) |
| **Pagos** | Claude Code + Stellar SDK + GitHub Copilot (autocompletado) |
| **Agentes AI** | Claude Code + LangChain + Stellar SDK (ejecución on-chain) |
| **Sin costo / offline** | Ollama local + VS Code + Stellar CLI → 100% offline, 0 costo |

## Tips para usar AI efectivamente

- **No dependas de un solo servicio:** Ten siempre un backup (ej: si Claude está caído, usa Gemini)
- **Cachea respuestas útiles:** Guardá snippets que funcionen en un archivo `snippets.md`
- **Usa AI para lo tedioso:** Boilerplate, tests, CSS — guarda tu creatividad para la lógica de negocio
- **No pelees con la AI:** Si después de 3 intentos no te da lo que quieres, hazlo manual y continua
- **Documenta con AI:** Al final, pídele que te escriba el README basado en el código actual
