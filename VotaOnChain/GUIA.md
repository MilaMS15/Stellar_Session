# VotaOnChain — Guía paso a paso

Construye esta dApp de votación on-chain en ~2 horas. Cada paso es independiente — si ya tienes algo instalado, sáltalo.

---

## PASO 0 — Setup inicial (5 min)

```bash
npm create vite@latest vota-onchain -- --template react-ts
cd vota-onchain
npm install
npm install @stellar/stellar-sdk
npx jsr add @creit-tech/stellar-wallets-kit
npm install @neondatabase/serverless
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

En `tailwind.config.js`:
```js
content: ["./index.html", "./src/**/*.{ts,tsx}"]
```

En `src/index.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

---

## PASO 1 — Contrato Soroban (20 min)

### 1.1 Instalar Rust + Soroban CLI

```bash
# Rust (si no lo tienes)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32v1-none

# Stellar CLI
cargo install --locked stellar-cli
```

### 1.2 Crear el contrato

```bash
stellar contract init vota-contract
cd vota-contract
```

Reemplaza `src/lib.rs` con el contrato del proyecto (`contract/src/lib.rs` en este repo).

El contrato expone estas funciones:

| Función | Argumentos | Retorno | Descripción |
|---|---|---|---|
| `__constructor` | `admin: Address, options: Vec<Symbol>` | — | Corre una sola vez, atómicamente, al desplegar. Fija el admin y las opciones |
| `get_admin` | — | `Address` | Retorna el admin del contrato |
| `vote` | `voter: Address, option: Symbol` | `bool` | Registra un voto (false si ya votó) |
| `get_votes` | — | `Map<Symbol, u32>` | Retorna todos los votos |
| `has_voted` | `voter: Address` | `bool` | Verifica si ya votó |

> El contrato usa `__constructor` en vez de un `init()` manual — así el deploy y la inicialización (admin + opciones) ocurren en la misma transacción, sin ventana entre ambos pasos en la que alguien más pueda inicializar el contrato primero.

### 1.3 Compilar, deployar e inicializar

```bash
# Compilar (genera el .wasm en target/wasm32v1-none/release/)
stellar contract build

# Crear identidad para deployar (solo la primera vez)
stellar keys generate deployer --network testnet

# Fondear con Friendbot
stellar keys fund deployer --network testnet

# Deployar en testnet — los argumentos del constructor van después de "--"
# y quedan fijados atómicamente en el mismo deploy (deployer = admin)
stellar contract deploy \
  --wasm target/wasm32v1-none/release/vota_onchain.wasm \
  --source deployer \
  --network testnet \
  -- \
  --admin deployer \
  --options '["Opcion_A","Opcion_B","Opcion_C"]'
```

> Copia el **Contract ID** que imprime el deploy — se ve así: `CXXXXXXX...`
> Lo necesitas en el `.env.local` del frontend.

> Los IDs de opciones deben ser strings de máximo 10 caracteres (limitación del tipo `Symbol` en Soroban). Sin espacios.

### 1.4 Verificar que funciona

```bash
# Leer votos (debe retornar un mapa vacío o con ceros)
stellar contract invoke \
  --id TU_CONTRACT_ID \
  --network testnet \
  -- get_votes
```

---

## PASO 2 — Neon (base de datos, opcional, 5 min)

Neon es una base de datos PostgreSQL serverless. Se usa para guardar metadata de votos (wallet, opción, timestamp, txHash). **La dApp funciona sin esto** — el voto igual queda on-chain en Soroban.

### 2.1 Crear el proyecto

1. Ve a [neon.tech](https://neon.tech) → **New project**
2. Elige la región más cercana (us-east o eu-west)
3. Copia la **Connection string** desde el dashboard (Settings → Connection Details)

Tiene este formato:
```
postgresql://usuario:password@ep-xxxxx.us-east-2.aws.neon.tech/neondb?sslmode=require
```

### 2.2 Crear la tabla

En el **SQL Editor** de Neon, ejecuta:

```sql
create table votes (
  id uuid default gen_random_uuid() primary key,
  wallet_address text not null,
  option_voted text not null,
  tx_hash text,
  created_at timestamp with time zone default now()
);

-- Índices para consultas rápidas
create index on votes(wallet_address);
create index on votes(option_voted);
```

### 2.3 Configurar la variable de entorno

```bash
cp .env.example .env.local
```

Edita `.env.local`:

```
VITE_CONTRACT_ID=TU_CONTRACT_ID_DE_TESTNET
VITE_DATABASE_URL=postgresql://usuario:password@ep-xxxxx.us-east-2.aws.neon.tech/neondb?sslmode=require
```

> **Nota:** En producción nunca expongas la connection string en el frontend. Para una demo o prototipo es aceptable — si quieres más seguridad, agrega una Vercel Function como proxy.

---

## PASO 3 — Frontend (30 min)

Los archivos del frontend ya están listos en `src/`. Solo necesitas correr:

```bash
npm run dev
```

### Qué hace cada archivo

| Archivo | Responsabilidad |
|---|---|
| `src/lib/stellar.ts` | `getVotes`, `checkHasVoted`, `buildVoteTx`, `submitSignedTx` |
| `src/lib/kit.ts` | Singleton de StellarWalletsKit (Freighter) |
| `src/lib/db.ts` | Guardar metadata en Neon vía `@neondatabase/serverless` (opcional) |
| `src/components/WalletConnect.tsx` | Conectar/desconectar Freighter |
| `src/components/VoteForm.tsx` | Seleccionar opción y firmar transacción |
| `src/components/Results.tsx` | Barra de progreso con votos en tiempo real |
| `src/App.tsx` | Estado global + polling de votos cada 10s |

### Flujo completo de una transacción

```
1. buildVoteTx(walletAddress, optionId)
      → server.getAccount(walletAddress)
      → TransactionBuilder + contract.call('vote', ...)
      → server.prepareTransaction(tx)   ← agrega auth entries
      → retorna XDR (string)

2. StellarWalletsKit.signTransaction(xdr) ← usuario firma en Freighter
      → retorna signedTxXdr

3. submitSignedTx(signedTxXdr)
      → server.sendTransaction(tx)
      → polling hasta status !== 'NOT_FOUND'
      → retorna txHash
```

---

## PASO 4 — Verificar Neon (si lo configuraste)

Si configuraste Neon en el PASO 2, verifica que los votos se están guardando:

```sql
-- En el SQL Editor de neon.tech
SELECT * FROM votes ORDER BY created_at DESC LIMIT 10;
```

Para consultar votos agrupados por opción:

```sql
SELECT option_voted, COUNT(*) AS total
FROM votes
GROUP BY option_voted
ORDER BY total DESC;
```

---

## PASO 5 — Deploy en Vercel (5 min)

```bash
# Build
npm run build

# Deploy (necesitas Vercel CLI o hacerlo desde vercel.com)
npx vercel --prod
```

En Vercel, agrega las variables de entorno:
- `VITE_CONTRACT_ID`
- `VITE_DATABASE_URL` (si usas Neon)

---

## Adaptar a tu proyecto

Para cambiar la pregunta y las opciones, edita dos lugares:

**1. `src/lib/stellar.ts`** — IDs y labels:
```typescript
export const VOTE_OPTIONS = [
  { id: 'mi_opcion1', label: 'Nombre visible para el usuario' },
  { id: 'mi_opcion2', label: 'Otra opción' },
];
```

**2. Al deployar el contrato** — mismos IDs como argumento del constructor:
```bash
-- --admin deployer --options '["mi_opcion1","mi_opcion2"]'
```

> Los IDs deben coincidir exactamente entre el contrato y el frontend.

---

## Troubleshooting

**"Account not found" al deployar**
```bash
stellar keys fund deployer --network testnet
```

**No se puede volver a inicializar / "already initialized"**
El contrato usa `__constructor`, que solo corre una vez, atómicamente, en el momento del deploy — no existe una forma de volver a llamarlo. Si necesitas otras opciones o otro admin, deploya un contrato nuevo (obtendrás un Contract ID distinto).

**El frontend no lee los votos**
- Verifica que `VITE_CONTRACT_ID` en `.env.local` sea el correcto
- Confirma que el contrato fue deployado con `--admin` y `--options` (ver 1.3)
- Abre la consola del browser y busca el error exacto

**Freighter no conecta**
- Verifica que Freighter esté configurado en **Testnet** (no Mainnet)
- Recarga la extensión o el browser
- Verifica que el sitio tenga permisos en Freighter
