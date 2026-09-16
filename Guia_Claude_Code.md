# Guía de Claude Code para Builders de Stellar

Claude Code es una CLI que te permite trabajar con Claude directamente desde tu terminal. Es especialmente útil para prototipar rápido — te deja iterar sin salir de tu flujo de desarrollo, ya sea en una sesión corta de fin de semana o en un proyecto que vas a mantener.

## Instalación

```bash
# Instalador nativo (recomendado — se auto-actualiza solo)
curl -fsSL https://claude.ai/install.sh | bash        # macOS / Linux / WSL
irm https://claude.ai/install.ps1 | iex                # Windows (PowerShell)

# Alternativas: brew install --cask claude-code · winget install Anthropic.ClaudeCode
# (npm install -g @anthropic-ai/claude-code sigue funcionando, pero ya no es el método recomendado)

# Verificar instalación
claude --version

# Iniciar en tu proyecto
cd mi-proyecto
claude
```

## Plan Mode — Planifica antes de codear

Antes de escribir código, activa Plan Mode para que Claude lea el proyecto y proponga una arquitectura sin tocar nada hasta que apruebes.

```bash
claude --permission-mode plan
```

O ya dentro de una sesión, presiona `Shift+Tab` para ciclar entre modos de permiso hasta ver `⏸ plan mode on` en la barra de estado. Luego describe lo que querés construir:

```
> Quiero construir una app que permita enviar remesas PEN→USDC
> usando Stellar, con una interfaz simple en Next.js
```

Esto te da:

- Estructura de archivos propuesta
- Dependencias necesarias
- Pasos de implementación ordenados
- Riesgos técnicos identificados

> **Pro tip:** Guardá el plan en un archivo `PLAN.md` y usalo como referencia durante todo el desarrollo.

## CLAUDE.md — Tu contexto persistente

Creá un archivo `CLAUDE.md` en la raíz de tu proyecto con contexto importante:

```markdown
# CLAUDE.md

## Proyecto
App de remesas PERU usando Stellar/Soroban.

## Stack
- Frontend: Next.js 15 + TypeScript
- Blockchain: Stellar SDK + Soroban
- Styling: Tailwind CSS

## Decisiones técnicas
- Usar Stellar testnet (no mainnet)
- USDC como stablecoin principal
- Freighter wallet para auth

## Convenciones
- Español para UI, inglés para código
- Componentes en PascalCase
- Funciones en camelCase
```

Claude lee este archivo automáticamente y mantiene el contexto entre sesiones. También podés usar `.claude/rules/*.md` para instrucciones específicas de una carpeta, y `CLAUDE.local.md` (gitignored) para overrides personales que no compartís con el equipo.

## Subagentes y trabajo en paralelo

Pedirle a Claude en una sola conversación que "haga 3 cosas en paralelo" **no** dispara ejecución paralela real — hay que ser explícito sobre el mecanismo:

- **Subagentes** (dentro de una sesión): Claude lanza agentes con su propio contexto para tareas independientes y te devuelve un resumen. Pedilo explícitamente:
  ```
  > Investiga en paralelo con subagentes: el módulo de auth,
  > cómo integrar Stellar SDK en el frontend, y ejemplos de
  > contratos Soroban para pagos
  ```
- **Worktrees** (sesiones paralelas sin choques): cada sesión trabaja en su propio branch/checkout de git.
  ```bash
  claude --worktree feature-auth
  ```
- **Background agents / Agent teams**: sesiones independientes que corren en paralelo y podés monitorear desde una sola vista, o un equipo de sesiones que Claude coordina.

### Cuándo usar cada uno
- **Subagentes:** investigación o tareas de solo lectura que no chocan entre sí
- **Worktrees:** cambios de código simultáneos pero en ramas distintas
- **Background agents:** tareas largas que corren mientras seguís trabajando en otra cosa

### Cuándo NO paralelizar
- Cuando una tarea depende del resultado de otra
- Para editar el mismo archivo desde múltiples agentes/sesiones a la vez
- Cuando necesitás razonamiento secuencial

## Otras funciones útiles

- **Hooks** — automatizan acciones en eventos del ciclo de vida (ej. correr el linter después de cada edit)
- **MCP** — conecta Claude a servicios externos (bases de datos, Slack, navegador, APIs de Stellar/Horizon)
- **Skills** — flujos y conocimiento reutilizable que invocás con `/<nombre>` o que Claude carga solo
- **Sesiones** — `claude --continue` retoma la última sesión, `/resume` elige una del historial
- **`/context`** — muestra qué hay cargado en el contexto actual; `/compact` lo resume para liberar espacio

## Workflows recomendados

### Fase 1 — Setup y MVP
```
1. Activa Plan Mode (Shift+Tab) y describe tu idea completa
2. Revisa y ajusta el plan
3. "Implementa el paso 1 del plan"
4. Itera paso por paso
```

### Fase 2 — Features y integración
```
1. "Revisa el estado actual del proyecto y dime qué falta"
2. "Implementa [feature específica]"
3. "Corre los tests y arregla lo que falle"
```

### Fase 3 — Polish y demo
```
1. "Revisa el código buscando bugs obvios"
2. "Mejora la UI para la demo"
3. "Escribeme un script de demo de 3 minutos"
```

## Tips clave

- **Sé específico:** "Crea un componente que muestre el balance de USDC del usuario conectado con Freighter" > "Haz el frontend"
- **Itera en pequeño:** Pide cambios incrementales, no reescrituras completas
- **Usa el contexto:** Referenciá archivos específicos cuando pidas cambios
- **Confia pero verifica:** Siempre revisá el código generado, especialmente interacciones con blockchain
- **Commitea seguido:** Pedile a Claude que haga commits descriptivos después de cada feature funcional
