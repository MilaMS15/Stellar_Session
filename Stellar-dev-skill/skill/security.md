# Seguridad en Stellar y contratos

Checklist de seguridad, clases de vulnerabilidad y patrones seguros para contratos y aplicaciones Stellar.

---

## Modelo de amenaza

Asume que el atacante controla:

- Todos los argumentos pasados a las funciones del contrato
- El orden y timing de las transacciones
- Todas las cuentas excepto las que requieren firma
- La capacidad de desplegar contratos que imitan tu interfaz
- La forma del estado on-chain que puede crear legalmente (cantidad de entradas, TTLs — puede incluso extender el TTL de tus entradas)

## Lo que la plataforma ya descarta

- **No hay `delegatecall`** — los contratos no pueden ejecutar bytecode ajeno en su propio contexto; los hijacks estilo proxy no existen.
- **No hay reentrancy** — el host bloquea llamadas reentrantes, directas o indirectas. Las llamadas externas siguen siendo fronteras de fallo/budget/side-effect — ordena las actualizaciones de estado deliberadamente.
- **Protección de replay de auth gestionada por el host** — nonces y expiraciones en las entradas de autorización las aplica el host, no tu código.
- **Autorización explícita** — `require_auth()` es opt-in, lo que significa que *olvidarlo* es el modo de fallo a buscar.

---

## Clases de vulnerabilidad

### 1. Autorización faltante

```rust
// MAL — cualquiera puede vaciar la cuenta
pub fn withdraw(env: Env, to: Address, amount: i128) {
    transfer_tokens(&env, &to, amount);
}

// BIEN
pub fn withdraw(env: Env, to: Address, amount: i128) {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
    transfer_tokens(&env, &to, amount);
}
```

Cada ruta privilegiada necesita `require_auth()` sobre la dirección correcta — cargada de storage, no un parámetro que provee el caller (`who.require_auth()` sobre un `who` arbitrario no prueba nada).

### 2. Auth replay a través de middleware (falta `require_auth` en la capa externa)

La variante cross-contract del punto anterior, y el bug de auth más común en producción real:

```rust
// MAL — nunca autoriza a `user` aquí
pub fn settle(env: Env, user: Address, amount: i128) {
    token_client.transfer(&user, &recipient, &amount);
}
```

Si `user` pre-firmó un árbol de auth que incluye el `token.transfer` interno, *cualquiera* puede llamar `settle(user, ...)` y consumirlo. Re-autoriza en **cada** capa que ejerza la autoridad de una dirección, no solo en la llamada más profunda.

### 3. Reinicialización

Solo relevante si usas un `initialize` con guard manual en vez de `__constructor` (que no puede volver a correr):

```rust
// BIEN — rechaza una segunda llamada
pub fn initialize(env: Env, admin: Address) {
    if env.storage().instance().has(&DataKey::Admin) {
        panic_with_error!(&env, Error::AlreadyInitialized);
    }
    env.storage().instance().set(&DataKey::Admin, &admin);
}
```

Sin el guard, cualquiera puede llamar `initialize` primero (o de nuevo) y capturar el rol de admin. **Prefiere `__constructor`** — elimina esta clase de bug por completo.

### 4. Llamadas a contratos arbitrarios

Llamar a la dirección que pasa el usuario le permite a un atacante sustituir un contrato que imita la interfaz:

```rust
// BIEN — allowlist de contratos externos
pub fn swap(env: Env, token: Address, amount: i128) {
    let allowed: Vec<Address> = env.storage().instance().get(&DataKey::AllowedTokens).unwrap();
    if !allowed.contains(&token) {
        panic_with_error!(&env, Error::TokenNotAllowed);
    }
}
```

Una dirección de contrato provista por el usuario es código arbitrario: puede fallar, hacer trap, quemar budget, emitir eventos engañosos, o llamar a otros contratos (solo no puede reentrarte a ti).

### 5. Overflow/underflow de enteros

`overflow-checks = true` en el perfil release atrapa overflows en runtime (panic), pero la matemática explícita con checked falla más limpio:

```rust
let new_balance = balance.checked_add(amount).expect("overflow");
```

Valida también signo y rango en los inputs — los montos `i128` admiten negativos, y `transfer(from, to, -1000)` es un retiro de `to` si no se valida.

### 6. Colisiones de claves de storage

Claves sin tipo pueden sobreescribir datos ajenos silenciosamente. Usa siempre un enum `#[contracttype]` para las claves.

### 7. Races de check-then-act

El estado puede cambiar entre transacciones. Haz checks y cambios de estado atómicamente en una sola invocación, y toma los límites de slippage (`min_out`) del caller:

```rust
pub fn swap(env: Env, user: Address, amount_in: i128, min_out: i128) {
    user.require_auth();
    let amount_out = calculate_output(amount_in);
    if amount_out < min_out {
        panic_with_error!(&env, Error::SlippageExceeded);
    }
}
```

La variante de allowance de tokens: `approve` sobreescribe, así que reducir una allowance no-cero puede sufrir front-running (gastar la vieja, luego la nueva). Aprobar a 0 primero, o usar auth de monto exacto en vez de allowances permanentes.

### 8. Fallas de TTL y archivado

- **Estado crítico expira**: una entrada `temporary` desaparece silenciosamente (para siempre), o entradas `persistent` se archivan y agregan fricción de restauración. Extiende TTLs en rutas calientes.
- **TTL usado como mecanismo de seguridad**: cualquiera puede extender el TTL de cualquier entrada vía `ExtendFootprintTTLOp`, así que "esta entrada expira, por lo tanto el permiso termina" está roto por diseño. Guarda un deadline explícito en el valor y verifícalo.

### 9. Confiar en valores de retorno cross-contract

Valida datos de contratos externos — allowlist de oráculos, sanity-check de magnitudes, exigir freshness:

```rust
let price: i128 = oracle_client.get_price(&asset);
if price <= 0 || price > MAX_REASONABLE_PRICE {
    panic_with_error!(&env, Error::InvalidPrice);
}
```

### 10. Agotamiento de recursos / fee griefing

Una función cuyo costo peor-caso sobre input controlado por el atacante excede los límites de red es una denegación de servicio contra usuarios legítimos. Busca: loops acotados por colecciones o cantidad de entradas controladas por el usuario, lecturas de storage o eventos por iteración, cantidad de firmas sin límite en `__check_auth`. Acota los conteos de iteración explícitamente.

### 11. Trampas de custom accounts (`__check_auth`)

- Verifica el `signature_payload` mismo — verificar cualquier otro mensaje autoriza llamadas arbitrarias.
- Aplica políticas desde `auth_contexts` (límites de gasto, allowlist de funciones); ignorarlo hace que las wallets "con política" sean decorativas.
- Nunca hagas `require_auth` de la propia dirección de la cuenta dentro de `__check_auth` (está evaluando la auth, no consumiéndola).

---

## Revisión para contratos que consumen tokens (AMMs, vaults, escrows, lenders)

Cualquier contrato que reciba una dirección de token debe asumir que puede ser el SAC de XLM nativo, el SAC de un activo clásico envuelto, o un contrato custom:

- [ ] ¿Qué tokens se aceptan — allowlisted o permissionless? (Permissionless ⇒ aplica la clase #4)
- [ ] ¿Se manejan los decimales? Consultar y guardar al registrar; nunca asumir 7.
- [ ] ¿Se re-verifica el monto recibido después de `transfer` donde importa? (tokens con fee-on-transfer entregan menos de lo pedido)
- [ ] ¿Sobreviven las particularidades SAC/clásico — destinatario sin trustline, activos `AUTH_REQUIRED`, cuentas congeladas, clawback del issuer?
- [ ] Flujos `transfer_from`: ¿la UX coordina la allowance, y el código autoriza al `spender` (no a `from`)?
- [ ] ¿Se emiten los shapes de evento estándar de token, para que indexers/wallets vean los flujos?

---

## Checklist de seguridad — frontend

```typescript
// MAL — exponer clave privada (NUNCA hacer esto)
const PRIVATE_KEY = process.env.NEXT_PUBLIC_SECRET_KEY; // PELIGROSO

// BIEN — usar la wallet del usuario para firmar
const { signedTxXdr } = await StellarWalletsKit.signTransaction(txXdr);

// BIEN — simular y mostrar al usuario qué va a firmar
const sim = await rpc.simulateTransaction(tx);
console.log(`Esta transacción costará ${sim.minResourceFee} stroops`);
```

- [ ] Nunca exponer claves privadas en variables de entorno del cliente
- [ ] Simular siempre antes de enviar (`simulateTransaction`)
- [ ] Mostrar detalles de la operación al usuario antes de pedir firma
- [ ] Verificar el resultado de transacciones enviadas (polling, no fire-and-forget)
- [ ] Timeout suficiente en `TransactionBuilder` (`setTimeout`)
- [ ] Validar inputs del usuario antes de construir transacciones
- [ ] Verificar el network passphrase antes de firmar (evitar firmar para la red equivocada)

---

## Manejo seguro de claves en desarrollo

```bash
# BIEN — archivo ignorado por git
echo "STELLAR_SECRET=S..." > .env.local
echo ".env.local" >> .gitignore
echo ".env*.local" >> .gitignore

# Verificar que no se commitea ninguna clave
grep -r "^S[A-Z2-7]\{55\}" . --include="*.ts" --include="*.js" --include="*.env"
```

---

## Manejo de errores explícito

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    InsufficientBalance = 2,
    InvalidAmount = 3,
    Unauthorized = 4,
}

pub fn withdraw(env: Env, user: Address, amount: i128) -> Result<i128, Error> {
    user.require_auth();
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    let balance = get_balance(&env, &user);
    if balance < amount {
        return Err(Error::InsufficientBalance);
    }
    set_balance(&env, &user, balance - amount);
    Ok(balance - amount)
}
```

Usa `panic_with_error!(&env, Error::X)` en vez de `panic!("mensaje")` — un panic plano llega al cliente como error opaco de host, no matcheable.

---

## Herramientas de seguridad

**Análisis estático**

- [Scout](https://github.com/CoinFabrik/scout-soroban) (CoinFabrik): `cargo install cargo-scout-audit && cargo scout-audit` — 20+ detectores. Salida SARIF para CI, extensión VSCode disponible.
- [Security Detectors SDK](https://github.com/OpenZeppelin/soroban-security-detectors-sdk) (OpenZeppelin): detectores pre-armados (`auth_missing`, `unchecked_ft_transfer`, extensión de TTL incorrecta) más un framework para escribir los propios.

**Verificación formal**

- [Certora Sunbeam](https://docs.certora.com/en/latest/docs/sunbeam/index.html): specs como macros de Rust, opera sobre bytecode WASM.
- [Komet](https://docs.runtimeverification.com/komet) (Runtime Verification): property tests + verificación formal vía semántica KWasm.

**Otras verificaciones**

```bash
cargo clippy -- -D warnings
cargo audit  # dependencias con vulnerabilidades conocidas
grep -n "unwrap()" src/lib.rs  # revisar cada unwrap — puede causar panic
```

**Base de conocimiento**: [sorobansecurity.com](https://sorobansecurity.com) — reportes de auditoría y base de vulnerabilidades buscable.

---

## Auditorías y bounties

- **[Audit Bank](https://stellar.org/grants-and-funding/soroban-audit-bank)** — auditorías subsidiadas por SDF para protocolos financiados por SCF.
- **[Immunefi — Stellar](https://immunefi.com/bug-bounty/stellar/)** — hasta $250K por vulnerabilidades de core/SDK/CLI.
- **[Immunefi — OpenZeppelin on Stellar](https://immunefi.com/bug-bounty/openzeppelin-stellar/)** — hasta $25K por la librería de contratos auditados.

Antes de pedir una auditoría: corre los analizadores estáticos, completa los checklists de arriba, documenta tu modelo de amenaza y ten la suite de tests en verde.

---

## Checklist de contrato

- [ ] Todas las funciones privilegiadas requieren autorización, cargada de storage
- [ ] Cada capa que usa la autoridad de una dirección la re-autoriza (sin auth replay de middleware)
- [ ] Inicialización solo puede ocurrir una vez (o usa `__constructor`)
- [ ] Llamadas a contratos externos validadas/allowlisted; valores de retorno sanity-checked
- [ ] Aritmética con checked; signos y rangos de inputs validados
- [ ] Claves de storage tipadas y sin colisión
- [ ] TTLs críticos extendidos proactivamente; sin asumir TTL como seguridad
- [ ] Loops y footprints acotados ante input adversarial
- [ ] Eventos emitidos para cambios de estado auditables (códigos de error nunca renumerados)
- [ ] Camino de upgrade con auth gateada, testeado (happy + failure), y replay-safe

---

## Resumen de riesgos por severidad

| Severidad | Patrón | Ejemplo |
|---|---|---|
| CRÍTICO | Falta de `require_auth()` | Cualquier función que mueve fondos |
| CRÍTICO | Auth replay a través de middleware | `settle(user, ...)` sin `user.require_auth()` |
| CRÍTICO | Inicialización múltiple sin protección | `initialize` sin verificar estado (usar `__constructor`) |
| ALTO | Aritmética sin control de overflow | Suma de montos sin `checked_add` |
| ALTO | Confiar en retorno de contrato externo sin validar | Precio de oráculo sin sanity-check |
| MEDIO | Inputs sin validar | Montos negativos o cero |
| MEDIO | Loops sin acotar sobre input del atacante | Fee griefing / DoS |
| BAJO | `unwrap()` en datos de storage | Panic si el dato no existe |
| BAJO | TTL no extendido en datos persistent | Datos que expiran inesperadamente |
