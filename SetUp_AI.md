# Setup de AI Gratis

No necesitas una suscripción paga para construir con AI sobre Stellar. Aquí van las mejores opciones gratuitas, ordenadas por facilidad de setup.

*Actualizado a setiembre 2026 — los free tiers de los proveedores cloud cambian seguido, así que antes de un evento vale la pena confirmar límites y modelos vigentes en el dashboard de cada uno.*

---

## APIs en la nube (sin tarjeta de crédito)

### Tier 1 — Las más útiles para empezar

| Servicio | Modelo destacado | Límite gratis | Ideal para |
|---|---|---|---|
| Google AI Studio | Gemini 3.6 Flash (familia Flash) | ~1,500 requests/día | Razonamiento largo, análisis de código |
| Groq | Llama 3.3 70B, GPT-OSS 120B/20B | Rate limit alto (miles de tokens/min) | Iteración rápida, respuestas muy veloces |
| Mistral | Mistral Large 3, Codestral | Tier "Experiment": ~1B tokens/mes, 2 req/min | Generación de código, debugging |

> **Consejo:** Regístrate en los tres antes del evento. Si uno se cae o llega al límite, tienes backup inmediato.
>

### Tier 2 — Alternativas sólidas

| Servicio | Modelo destacado | Notas |
|---|---|---|
| Anthropic | Claude Haiku 4.5 / Sonnet 5 / Opus 5 | Sin free tier continuo en la API — al crear cuenta en `platform.claude.com` recibes ~$5 de crédito sin tarjeta. Claude.ai (chat web) sí tiene un plan Free permanente con límite diario de mensajes |
| OpenRouter | 28+ modelos con sufijo `:free` (incluye Nemotron 3 Ultra, 1M de contexto) | $0 por token, sin tarjeta, ~20 requests/min |
| HuggingFace Inference | Modelos open source | Ideal para experimentar con modelos especializados |
| Cohere | Command A (256K de contexto) | Trial key sin expiración: 1,000 llamadas/mes, 20 req/min — solo para prototipos |
| Together AI | Llama 4, DeepSeek V4, Qwen3 | $25–50 de créditos al registrarte; programa para startups hasta $50K |

---

## Modelos locales (en tu propia laptop)

Si tienes una laptop con 16 GB de RAM o más, puedes correr modelos locales sin depender de internet. Muy útil cuando tu conexión no es confiable o estás en un evento con WiFi saturada.

### Setup con Ollama

```bash
# Instalar Ollama (Linux/macOS)
curl -fsSL https://ollama.com/install.sh | sh

# Modelos recomendados para coding (setiembre 2026)
ollama pull qwen3-coder:30b         # ~19 GB a Q4 — el más fuerte para trabajo agéntico y refactors multi-archivo
ollama pull qwen2.5-coder:7b        # 4.7 GB — sigue siendo excelente para autocompletado/FIM
ollama pull llama3.1:8b             # General purpose

# Verificar que corre bien
ollama run qwen2.5-coder:7b "Hola, escribe un hello world en Rust"
```

### Modelos recomendados según tu RAM

| RAM disponible | Modelo sugerido | Tamaño | Capacidad |
|---|---|---|---|
| 8 GB | `qwen2.5-coder:3b` | ~2 GB | Autocompletado básico |
| 16 GB | `qwen2.5-coder:7b` | ~4.7 GB | Generación de código sólida, autocompletado FIM |
| 32 GB | `qwen3-coder:30b` (Q4) | ~19 GB | Trabajo agéntico, refactors multi-archivo, razonamiento técnico |
| 64 GB+ | `qwen2.5-coder:32b` o `qwen3-coder:30b` a mayor precisión | ~20 GB+ | Casi al nivel de modelos cloud |

### Cuándo usar modelos locales vs nube

- **Local:** Cuando tu conexión a internet no es confiable, o cuando quieres respuestas instantáneas sin latencia de red
- **Nube:** Cuando necesitas razonamiento complejo, contexto largo (archivos grandes) o los últimos modelos
- **Combo ideal:** Modelo local para autocompletado + API en nube para consultas grandes

---

## Alquilar GPU (si tu laptop no da)

Para modelos más grandes (30B+), puedes alquilar una GPU por horas a bajo costo. Los precios son **por hora** — calcula según las horas reales de uso, no asumas un paquete fijo de "fin de semana".

| Servicio | Costo aprox. | GPU disponible | Notas |
|---|---|---|---|
| RunPod | ~$1.50–3.50 USD/hora | A100, H100 (Community/Secure Cloud) | Buena interfaz, fácil de usar |
| Vast.ai | ~$0.70–2 USD/hora (subasta/spot) | Variado | Más barato, más configuración |
| Lambda | ~$2–4 USD/hora on-demand | A100, H100, B200 | Muy estable, buena documentación |
| Google Colab Pro | $9.99 USD/mes (100 compute units) | T4, L4, A100 | Alternativa si solo necesitas un notebook |

> **Tips para alquiler de GPU:**
> - Reserva con anticipación — la demanda sube durante eventos grandes
> - Usa spot/subasta (Vast.ai) si puedes tolerar interrupciones ocasionales — suele ser lo más barato para sesiones cortas
> - Ten un script de setup listo (`setup.sh`) para no perder tiempo configurando el entorno
> - Considera Google Colab si solo necesitas correr un script puntual

---

## Integrar AI en tu proyecto Stellar

### Con Vercel AI SDK (Next.js)

```bash
npm install ai @ai-sdk/google @ai-sdk/groq
```

```typescript
import { generateText } from "ai";
import { google } from "@ai-sdk/google";

const { text } = await generateText({
  model: google("gemini-3.6-flash"),
  prompt: "Explica esta transacción Stellar: " + txData,
});
```

### Con la API de Groq directamente

```typescript
import Groq from "groq-sdk";

const groq = new Groq({ apiKey: process.env.GROQ_API_KEY });

const response = await groq.chat.completions.create({
  model: "llama-3.3-70b-versatile",
  messages: [{ role: "user", content: "Analiza este contrato Soroban..." }],
});
```

### Variables de entorno para múltiples proveedores

```bash
# .env.local
GOOGLE_GENERATIVE_AI_API_KEY=tu-key-aqui
GROQ_API_KEY=tu-key-aqui
MISTRAL_API_KEY=tu-key-aqui
ANTHROPIC_API_KEY=tu-key-aqui
```

> Nunca pongas las keys directamente en el código. Usa siempre `.env.local` y agrega ese archivo a `.gitignore`.

---

## Setup rápido recomendado

**Opción A — Minimalista (0 configuración):**
```
Google AI Studio + Groq → sin costo, listo en 5 minutos
```

**Opción B — Con laptop potente:**
```
Ollama + qwen3-coder:30b + Groq como backup → sin costo, funciona offline
```

**Opción C — Máxima potencia gratis:**
```
$5 de crédito inicial de Anthropic + Google AI Studio + Mistral API → 3 modelos con distintas fortalezas
```

**Opción D — Para proyectos con AI integrada:**
```
Vercel AI SDK + Groq (velocidad) + Gemini (razonamiento) → sin costo, cambia de modelo con una línea
```

---

## Checklist antes de empezar

- [ ] Crear cuenta en Google AI Studio y obtener API key
- [ ] Crear cuenta en Groq y obtener API key
- [ ] Crear cuenta en Mistral y obtener API key
- [ ] (Opcional) Crear cuenta en `platform.claude.com` para el crédito inicial de Anthropic
- [ ] (Opcional) Instalar Ollama y descargar al menos un modelo
- [ ] Testear que cada API key funciona antes del evento
- [ ] Guardar las keys en un archivo `.env.local` (nunca en el código)
- [ ] Agregar `.env.local` al `.gitignore`
- [ ] Tener un modelo local como backup por si falla el internet
