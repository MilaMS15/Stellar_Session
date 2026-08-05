# Frontend con Stellar SDK

Guía para integrar Stellar en aplicaciones web con JavaScript/TypeScript, React y Next.js — SDK, wallets, firma, invocación de contratos.

---

## Instalación

> **Requiere Node.js 22+.** Desde SDK v16, `@stellar/stellar-base` está integrado en `@stellar/stellar-sdk` (no instales ambos), el SDK es ESM-first y usa `fetch` nativo en vez de axios. Si aún importas `@stellar/stellar-base` directamente, cambia el import y desinstala el paquete base.

```bash
npm install @stellar/stellar-sdk @stellar/freighter-api

# Multi-wallet — Wallets Kit v2 se distribuye en JSR, no en npm
npx jsr add @creit-tech/stellar-wallets-kit
```

---

## Inicialización del SDK

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

// Testnet
const testnetRpc = new StellarSdk.rpc.Server("https://soroban-testnet.stellar.org");
const testnetHorizon = new StellarSdk.Horizon.Server("https://horizon-testnet.stellar.org");
const testnetPassphrase = StellarSdk.Networks.TESTNET;

// Mainnet — no hay endpoint RPC hosteado por SDF, usa un proveedor
// (ver https://developers.stellar.org/docs/data/apis/rpc/providers)
const mainnetRpcUrl = process.env.NEXT_PUBLIC_STELLAR_MAINNET_RPC_URL;
if (!mainnetRpcUrl) throw new Error("Falta NEXT_PUBLIC_STELLAR_MAINNET_RPC_URL");
const mainnetRpc = new StellarSdk.rpc.Server(mainnetRpcUrl);
const mainnetPassphrase = StellarSdk.Networks.PUBLIC;
```

> El namespace de RPC es `StellarSdk.rpc` — el antiguo `SorobanRpc` fue renombrado en SDK v16+. Si ves código con `SorobanRpc.Server`, es de una versión anterior.

---

## Conectar wallet con Stellar Wallets Kit (v2)

Stellar Wallets Kit soporta múltiples wallets (Freighter, LOBSTR, xBull, etc.) con una sola interfaz. **v2 es un singleton estático** — se inicializa una vez y se llama por métodos estáticos, sin instanciar ni pasar referencias.

```typescript
import { StellarWalletsKit, Networks } from "@creit-tech/stellar-wallets-kit";
import { defaultModules } from "@creit-tech/stellar-wallets-kit/modules/utils";

// defaultModules() carga las wallets que no necesitan configuración extra.
// Módulos con prerequisitos (WalletConnect, Ledger, Trezor) se importan aparte.
StellarWalletsKit.init({
  modules: defaultModules(),
  network: Networks.TESTNET,
});

// authModal() abre el selector, activa el módulo elegido y devuelve la dirección
const { address } = await StellarWalletsKit.authModal();

// Firmar
const { signedTxXdr } = await StellarWalletsKit.signTransaction(txXdr);

// Desconectar
await StellarWalletsKit.disconnect();
```

> **Migrando desde v1?** v1 vivía en npm bajo `@creit.tech/stellar-wallets-kit` (con punto), con `new StellarWalletsKit({...})`, `allowAllModules()` y `openModal({ onWalletSelected })`. v2 se movió a JSR bajo `@creit-tech/stellar-wallets-kit` (con guion), es totalmente estático, reemplazó `allowAllModules()` por `defaultModules()`, y unificó selección + dirección en `authModal()`. Los IDs de wallet específicos ahora viven en subpaths por módulo, ej. `@creit-tech/stellar-wallets-kit/modules/freighter`.

---

## Conectar con Freighter directamente

```typescript
import {
  isConnected,
  getAddress,
  requestAccess,
  signTransaction,
  getNetwork,
} from "@stellar/freighter-api";

// Verificar instalación
const { isConnected: instalado } = await isConnected();
if (!instalado) {
  alert("Instala la extensión Freighter en tu navegador");
}

// getAddress() devuelve address: "" hasta que la app tenga acceso otorgado —
// una dirección no vacía significa que ya estás autorizado.
const { address } = await getAddress();

// Si no hay acceso, solicitarlo (prompt al usuario)
const { address: nuevaAddress } = await requestAccess();

// Firmar transacción
const { signedTxXdr } = await signTransaction(txXdr, {
  networkPassphrase: "Test SDF Network ; September 2015",
});
```

> `getPublicKey()` está deprecado — usa `getAddress()`.

---

## Invocar un contrato con `contract.Client` (forma recomendada)

La manera canónica de llamar a un contrato desde JS es `contract.Client`, no construir manualmente `Contract.call` + `assembleTransaction`. El cliente lee la interfaz del contrato desde la red, así que cada método es invocable por nombre y devuelve un `AssembledTransaction` — resultado nativo en JS, sin armar `ScVal` a mano.

```typescript
import { contract } from "@stellar/stellar-sdk";

interface CounterContract {
  increment: (
    options?: contract.MethodOptions,
  ) => Promise<contract.AssembledTransaction<number>>;
}

// signTransaction viene de la wallet (Freighter/Wallets Kit en el navegador).
// contract.basicNodeSigner(keypair, networkPassphrase) es el equivalente para Node.
export async function getCounterClient(
  contractId: string,
  publicKey: string,
  signTransaction: contract.ClientOptions["signTransaction"],
) {
  return contract.Client.from<CounterContract>({
    contractId,
    rpcUrl: "https://soroban-testnet.stellar.org",
    networkPassphrase: "Test SDF Network ; September 2015",
    publicKey,
    signTransaction,
  });
}

// Preview (simulación gratuita) y luego firmar + enviar para aplicar on-chain.
export async function increment(client: contract.Client & CounterContract) {
  const tx = await client.increment();
  console.log("preview:", tx.result); // valor de retorno predicho, sin firma
  const sent = await tx.signAndSend(); // envía y espera confirmación
  return sent.result;
}
```

Para contratos con muchos métodos, genera la interfaz TypeScript con el CLI de bindings en vez de escribirla a mano:

```bash
stellar contract bindings typescript \
  --id CONTRACT_ID \
  --network testnet \
  --output-dir ./src/contracts/mi-contrato
```

<details>
<summary><b>Avanzado: invocación de bajo nivel sin cliente</b></summary>

Solo cuando necesitas control directo sobre la transacción (ej. combinar una llamada de contrato con operaciones clásicas).

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

export async function invokeContract(
  rpc: StellarSdk.rpc.Server,
  sourceAddress: string,
  contractId: string,
  method: string,
  args: StellarSdk.xdr.ScVal[],
  networkPassphrase: string,
) {
  const account = await rpc.getAccount(sourceAddress);
  const contract = new StellarSdk.Contract(contractId);

  const transaction = new StellarSdk.TransactionBuilder(account, {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(180)
    .build();

  // prepareTransaction simula y aplica footprint/auth/fees en un solo paso
  const prepared = await rpc.prepareTransaction(transaction);
  return prepared.toXDR();
}
```

**Construir argumentos `ScVal` a mano** (solo necesario en la ruta de bajo nivel):

```typescript
import * as StellarSdk from "@stellar/stellar-sdk";

const addressVal = StellarSdk.Address.fromString(address).toScVal();
const i128Val = StellarSdk.nativeToScVal(BigInt(amount), { type: "i128" });
const u32Val = StellarSdk.nativeToScVal(42, { type: "u32" });
```

</details>

---

## Enviar y esperar confirmación

```typescript
async function submitSorobanTransaction(rpc: StellarSdk.rpc.Server, signedXdr: string, networkPassphrase: string) {
  const transaction = StellarSdk.TransactionBuilder.fromXDR(signedXdr, networkPassphrase) as StellarSdk.Transaction;

  const response = await rpc.sendTransaction(transaction);
  if (response.status === "ERROR") {
    throw new Error(`Envío falló: ${response.errorResult}`);
  }

  // pollTransaction maneja el loop de reintentos (5 intentos, 1s de separación
  // por defecto — ajustable con { attempts, sleepStrategy })
  const result = await rpc.pollTransaction(response.hash);

  if (result.status === "SUCCESS") {
    return { hash: response.hash, result: result.returnValue };
  }
  throw new Error(`Transacción falló: ${result.status}`);
}
```

Para transacciones clásicas (pagos, trustlines), usa Horizon:

```typescript
async function submitClassicTransaction(horizon: StellarSdk.Horizon.Server, signedXdr: string, networkPassphrase: string) {
  const transaction = StellarSdk.TransactionBuilder.fromXDR(signedXdr, networkPassphrase) as StellarSdk.Transaction;
  const response = await horizon.submitTransaction(transaction);
  return { hash: response.hash, ledger: response.ledger };
}
```

> Para fallos de envío, Horizon devuelve códigos bajo `error.response?.data?.extras?.result_codes` (`transaction` + por `operation`).

---

## Leer saldos y estado

```typescript
import { NotFoundError } from "@stellar/stellar-sdk";

export async function getBalance(horizon: StellarSdk.Horizon.Server, address: string) {
  try {
    const account = await horizon.loadAccount(address);
    const nativeBalance = account.balances.find((b) => b.asset_type === "native");
    return nativeBalance?.balance || "0";
  } catch (error) {
    if (error instanceof NotFoundError) return "0"; // cuenta sin fondear
    throw error;
  }
}
```

Para leer estado de contrato de solo lectura, `rpc.Server` tiene atajos que arman la interfaz del contrato por ti (incluye el spec built-in de las Stellar Asset Contracts):

```typescript
// Ejecuta un método de solo lectura y devuelve el resultado ya decodificado
const { result: balance } = await rpc.queryContract<bigint>(
  tokenId,
  "balance",
  { id: "G..." } // args nombrados, por parámetro
);

// Descubre los métodos invocables de un contrato solo con su ID
const methods = await rpc.getContractMethods(tokenId);
```

---

## Hook React para wallet

```typescript
// hooks/useStellarWallet.ts
import { useState, useCallback } from "react";
import { StellarWalletsKit, Networks } from "@creit-tech/stellar-wallets-kit";
import { defaultModules } from "@creit-tech/stellar-wallets-kit/modules/utils";

StellarWalletsKit.init({ modules: defaultModules(), network: Networks.TESTNET });

export function useStellarWallet() {
  const [address, setAddress] = useState<string | null>(null);
  const [conectando, setConectando] = useState(false);

  const conectar = useCallback(async () => {
    setConectando(true);
    try {
      const { address } = await StellarWalletsKit.authModal();
      setAddress(address);
    } finally {
      setConectando(false);
    }
  }, []);

  const desconectar = useCallback(async () => {
    await StellarWalletsKit.disconnect();
    setAddress(null);
  }, []);

  const firmar = useCallback(async (txXdr: string) => {
    const { signedTxXdr } = await StellarWalletsKit.signTransaction(txXdr);
    return signedTxXdr;
  }, []);

  return { address, conectando, conectar, desconectar, firmar };
}
```

## Componente de conexión de wallet (React)

```tsx
// components/WalletConnect.tsx
import { useStellarWallet } from "../hooks/useStellarWallet";

export function WalletConnect() {
  const { address, conectando, conectar, desconectar } = useStellarWallet();

  const direccionCorta = (addr: string) => `${addr.slice(0, 4)}...${addr.slice(-4)}`;

  if (address) {
    return (
      <div className="flex items-center gap-2">
        <span className="font-mono text-sm">{direccionCorta(address)}</span>
        <button onClick={desconectar} className="px-3 py-1 bg-red-100 text-red-700 rounded">
          Desconectar
        </button>
      </div>
    );
  }

  return (
    <button onClick={conectar} disabled={conectando} className="px-4 py-2 bg-blue-600 text-white rounded disabled:opacity-50">
      {conectando ? "Conectando..." : "Conectar Wallet"}
    </button>
  );
}
```

---

## Smart Accounts (wallets con passkeys)

Para autenticación sin contraseña vía passkeys WebAuthn, usa Smart Account Kit:

```bash
npm install smart-account-kit
```

```typescript
import { SmartAccountKit, IndexedDBStorage } from "smart-account-kit";

const kit = new SmartAccountKit({
  rpcUrl: "https://soroban-testnet.stellar.org",
  networkPassphrase: "Test SDF Network ; September 2015",
  accountWasmHash: "YOUR_ACCOUNT_WASM_HASH",
  webauthnVerifierAddress: "CWEBAUTHN_VERIFIER_ADDRESS",
  storage: new IndexedDBStorage(),
});

// Al cargar la página — restauración silenciosa de sesión guardada
const result = await kit.connectWallet();

// Crear wallet nueva con passkey
const { contractId, credentialId } = await kit.createWallet("Mi App", "usuario@example.com", { autoSubmit: true });

// Firmar y enviar
await kit.signAndSubmit(transaction);
```

### Fee sponsorship con OpenZeppelin Relayer

El [OpenZeppelin Relayer](https://docs.openzeppelin.com/relayer/stellar) (también llamado Stellar Channels Service) maneja el envío de transacciones sin gas para el usuario — reemplaza al servicio deprecado Launchtube.

```typescript
import * as RPChannels from "@openzeppelin/relayer-plugin-channels";

const client = new RPChannels.ChannelsClient({
  baseUrl: "https://channels.openzeppelin.com/testnet",
  apiKey: "tu-api-key",
});

const response = await client.submitSorobanTransaction({ func: contractFunc, auth: contractAuth });
```

Instancia testnet hosteada: `https://channels.openzeppelin.com/testnet` (API keys en `/gen`).

---

## Variables de entorno para Next.js

```bash
# .env.local
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_RPC_URL=https://soroban-testnet.stellar.org
NEXT_PUBLIC_HORIZON_URL=https://horizon-testnet.stellar.org
NEXT_PUBLIC_CONTRACT_ID=C...
# Mainnet RPC — de un proveedor, no hay endpoint SDF público
NEXT_PUBLIC_STELLAR_MAINNET_RPC_URL=
```

```typescript
// lib/config.ts
const requireEnv = (name: string): string => {
  const value = process.env[name];
  if (!value) throw new Error(`Falta variable de entorno: ${name}`);
  return value;
};

export const CONFIG = {
  network: process.env.NEXT_PUBLIC_STELLAR_NETWORK as "testnet" | "mainnet",
  rpcUrl: process.env.NEXT_PUBLIC_STELLAR_NETWORK === "mainnet"
    ? requireEnv("NEXT_PUBLIC_STELLAR_MAINNET_RPC_URL")
    : "https://soroban-testnet.stellar.org",
  horizonUrl: process.env.NEXT_PUBLIC_HORIZON_URL!,
  contractId: process.env.NEXT_PUBLIC_CONTRACT_ID!,
  networkPassphrase:
    process.env.NEXT_PUBLIC_STELLAR_NETWORK === "mainnet"
      ? "Public Global Stellar Network ; September 2015"
      : "Test SDF Network ; September 2015",
} as const;
```

---

## Checklist de UX de transacciones

- [ ] Mostrar estado de carga durante la firma en la wallet
- [ ] Mostrar el hash de la transacción inmediatamente después de enviarla
- [ ] Rastrear estado de confirmación (pendiente → éxito/fallo)
- [ ] Manejar errores comunes con mensajes claros: wallet no conectada, usuario rechazó firmar, XLM insuficiente para fees, cuenta no fondeada, red incorrecta en la wallet, timeout
- [ ] Prevenir doble envío mientras se procesa
- [ ] Mostrar destino y monto antes de firmar
