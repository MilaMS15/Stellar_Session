# Pruebas ZK y privacidad en Stellar

Verificación ZK en Stellar. La capacidad depende de la versión de protocolo y del SDK — verifica siempre el estado del CAP, la versión de red, y el soporte de host functions en tu `soroban-sdk` antes de asumir que una primitiva está disponible.

---

## Qué hay disponible hoy — verificar antes de construir

| Primitiva | CAP | Estado |
|---|---|---|
| Operaciones BLS12-381 (suma/mul G1/G2, MSM, pairing check, hash-to-curve) | [CAP-0059](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0059.md) | **Disponible** (Protocolo 22+) |
| Host functions BN254 | [CAP-0074](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0074.md) | Propuesto — verificar estado actual |
| Hash Poseidon/Poseidon2 | [CAP-0075](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0075.md) | Propuesto — verificar estado actual |

**La curva lo decide todo.** Las pruebas BLS12-381 verifican nativamente on-chain hoy; las pruebas BN254 (default de Circom, Barretenberg, el wrapper Groth16 de RISC Zero) están bloqueadas hasta CAP-0074.

| Toolchain | Sistema de prueba | Curva | On-chain en Stellar |
|---|---|---|---|
| Circom + snarkjs (`-p bls12381`) | Groth16 | BLS12-381 | ✅ Hoy, vía CAP-0059 |
| Circom + snarkjs (default) | Groth16 | BN254 | Bloqueado en CAP-0074 |
| Noir + Barretenberg | UltraHonk | BN254 | Todavía no — atestiguar verificación off-chain |
| RISC Zero (STARK → Groth16) | Groth16 | BN254 | Bloqueado en CAP-0074 — atestiguar mientras tanto |

> Esto reemplaza el estado anterior de este archivo, donde asumíamos que Soroban no tenía ninguna operación de curva elíptica nativa. Desde el Protocolo 22, BLS12-381 sí está disponible — lo que faltaba era CAP-0074 (BN254), que sigue en propuesta.

---

## Casos de uso prácticos

| Caso de uso | Qué prueba | Por qué ZK |
|---|---|---|
| KYC sin datos | "Tengo KYC aprobado" | Sin revelar nombre/DNI |
| Solvencia | "Tengo > X saldo" | Sin revelar el saldo exacto |
| Elegibilidad | "Soy mayor de 18" | Sin revelar fecha de nacimiento |
| Votación anónima | "Voté exactamente una vez" | Sin revelar el voto |

---

## El verificador on-chain (Groth16 sobre BLS12-381)

El [ejemplo oficial `groth16_verifier`](https://github.com/stellar/soroban-examples/tree/main/groth16_verifier) es la implementación canónica:

```rust
#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    crypto::bls12_381::{Fr, G1Affine, G2Affine},
    vec, Env, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Groth16Error {
    MalformedVerifyingKey = 0,
}

#[derive(Clone)]
#[contracttype]
pub struct VerificationKey {
    pub alpha: G1Affine,
    pub beta: G2Affine,
    pub gamma: G2Affine,
    pub delta: G2Affine,
    pub ic: Vec<G1Affine>,
}

#[derive(Clone)]
#[contracttype]
pub struct Proof {
    pub a: G1Affine,
    pub b: G2Affine,
    pub c: G1Affine,
}

#[contract]
pub struct Groth16Verifier;

#[contractimpl]
impl Groth16Verifier {
    pub fn verify_proof(
        env: Env,
        vk: VerificationKey,
        proof: Proof,
        pub_signals: Vec<Fr>,
    ) -> Result<bool, Groth16Error> {
        let bls = env.crypto().bls12_381();

        if pub_signals.len() + 1 != vk.ic.len() {
            return Err(Groth16Error::MalformedVerifyingKey);
        }
        let mut vk_x = vk.ic.get(0).unwrap();
        for (s, v) in pub_signals.iter().zip(vk.ic.iter().skip(1)) {
            let prod = bls.g1_mul(&v, &s);
            vk_x = bls.g1_add(&vk_x, &prod);
        }

        let neg_a = -proof.a;
        let vp1 = vec![&env, neg_a, vk.alpha, vk_x, proof.c];
        let vp2 = vec![&env, proof.b, vk.beta, vk.gamma, vk.delta];

        Ok(bls.pairing_check(vp1, vp2))
    }
}
```

Las codificaciones de punto son big-endian sin comprimir: `G1Affine` son 96 bytes, `G2Affine` 192 bytes, `Fr` 32 bytes. El test suite del ejemplo muestra la conversión exacta desde tipos arkworks (`ark-bls12-381` + `ark-serialize`) — reutilízala al armar fixtures desde tu toolchain.

En producción, envolvé este verificador con lógica de aplicación: fija la `VerificationKey` al desplegar (en el constructor) en vez de recibirla como argumento de la llamada, y ata las pruebas al contexto (ver Anti-patrones abajo).

---

## Circom → verificación on-chain (funciona hoy)

Circom soporta BLS12-381 como campo objetivo — por eso es el toolchain que verifica nativamente en Stellar ahora mismo.

```bash
# 1. Circuito
cat > multiplier.circom <<'EOF'
pragma circom 2.1.6;
template Multiplier() {
    signal input a;
    signal input b;
    signal output c;
    c <== a * b;
}
component main = Multiplier();
EOF

# 2. Compilar para BLS12-381 (NO el default bn128 — ese está bloqueado en CAP-0074)
circom multiplier.circom --r1cs --wasm -p bls12381

# 3. Trusted setup
snarkjs powersoftau new bls12-381 12 pot12_0000.ptau
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="contrib" -e="random"
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau
snarkjs groth16 setup multiplier.r1cs pot12_final.ptau multiplier.zkey
snarkjs zkey export verificationkey multiplier.zkey verification_key.json

# 4. Witness + prueba
echo '{"a": 3, "b": 11}' > input.json
node multiplier_js/generate_witness.js multiplier_js/multiplier.wasm input.json witness.wtns
snarkjs groth16 prove multiplier.zkey witness.wtns proof.json public.json

# 5. Sanity-check off-chain antes de ir on-chain
snarkjs groth16 verify verification_key.json public.json proof.json
```

Luego convierte `proof.json`/`verification_key.json` (coordenadas en string decimal) a los tipos del contrato — serializa cada punto sin comprimir big-endian en los layouts de 96/192 bytes (vía arkworks, como en los tests del ejemplo) — e invoca `verify_proof`. Los `public.json` se convierten en el argumento `Vec<Fr>`; el contrato debe validar además *qué significan* esas señales.

Para una aplicación real, el flujo por prueba es: el cliente prueba localmente (WASM prover o nativo), envía `(proof, public_signals)` en una invocación de contrato, el contrato verifica + aplica política + actualiza estado.

---

## Noir y RISC Zero (verificación off-chain + atestación, por ahora)

Noir (Barretenberg → UltraHonk sobre BN254) y RISC Zero (STARK envuelto en Groth16 sobre BN254) todavía no verifican nativamente on-chain en Stellar — ni el sistema de prueba ni la curva están soportados hoy.

Dos patrones mientras se cierra la brecha:

1. **Oráculo de atestación**: un servicio verificador corre la verificación off-chain (`bb verify` para Noir, o el receipt de RISC Zero) y envía una atestación firmada; el contrato hace `require_auth()` de la dirección del atestador y aplica política. **Esto no es ZK trustless** — es un oráculo de cómputo verificable, y la suposición de confianza (el atestador) debe ser explícita y documentada.
2. **Cambiar el stack de prueba para la parte on-chain**: expresar la parte crítica on-chain como un circuito Circom/Groth16-BLS12-381 (walkthrough arriba) y dejar Noir/RISC Zero para componentes off-chain.

Sigue CAP-0074 (BN254): cuando se implemente, la verificación BN254 Groth16 será posible directamente.

---

## Patrones de arquitectura

- **Gateway de verificación**: aislar los checks criptográficos en un contrato/módulo verificador dedicado — normalizar inputs, verificar, emitir eventos explícitos de éxito/fallo. Superficie de auditoría más chica, upgrades más limpios.
- **Separación policy-and-proof**: `Verifier` (validez criptográfica) → `Policy` (reglas de negocio/compliance) → `Application` (transición de estado). Cada uno testeable y upgradeable por separado.
- **Gating por capacidad**: habilitar flujos ZK solo donde las primitivas requeridas están confirmadas disponibles; mantener fallbacks deterministas y documentar la matriz de red/protocolo soportada.

---

## Anti-patrones — qué evitar

**Verificar la prueba pero no la declaración.** Una prueba válida solo muestra que *algún* witness satisface el circuito. El contrato debe validar la semántica de los inputs públicos: para quién es esta prueba, qué root de Merkle, qué acción, qué monto.

**Falta de anti-replay.** Las pruebas válidas pueden reenviarse. Ata un nonce/sesión/acción a los inputs públicos y persiste un guard de replay (set de nullifiers) on-chain:

```rust
// MAL — sin prevención de replay attacks
pub fn verificar_y_dar_acceso(env: Env, usuario: Address, proof: Proof, pub_signals: Vec<Fr>) {
    let valido = Groth16Verifier::verify_proof(env.clone(), vk, proof, pub_signals).unwrap();
    assert!(valido, "prueba inválida");
    dar_acceso(&env, &usuario); // la misma prueba puede reusarse!
}

// BIEN — marcar la prueba como usada antes de aplicar el efecto
pub fn verificar_y_dar_acceso(env: Env, usuario: Address, proof: Proof, pub_signals: Vec<Fr>) {
    let hash = env.crypto().sha256(&proof.a.to_bytes().into());
    assert!(!env.storage().temporary().has(&hash), "prueba ya usada");

    let valido = Groth16Verifier::verify_proof(env.clone(), vk, proof, pub_signals).unwrap();
    assert!(valido, "prueba inválida");

    env.storage().temporary().set(&hash, &true);
    dar_acceso(&env, &usuario);
}
```

**Mismatch de curva.** Circom usa bn128 por defecto; en Stellar compila con `-p bls12381` o tu prueba será inverificable on-chain.

**Higiene del trusted setup.** Groth16 necesita un setup de fase 2 específico del circuito — para producción usa una ceremonia real multi-parte, no un dev setup de un solo contribuyente.

---

## Testing

- Unitario: validación de dominio de inputs, protección de replay, corrección de eventos, rechazo de pruebas malformadas/manipuladas
- Integración: flujo completo prove → submit → verify → transición de estado contra red local
- Operacional: costos de recursos para tamaños de prueba realistas vía simulación (`--send=no`) — los pairing checks son caros, presupuesta antes de comprometerte a verificar por transacción

---

## Referencias

- [groth16_verifier example](https://github.com/stellar/soroban-examples/tree/main/groth16_verifier) — verificador canónico + fixtures de test con arkworks
- [Docs SDK BLS12-381](https://docs.rs/soroban-sdk/latest/soroban_sdk/crypto/bls12_381/index.html)
- [Circom docs](https://docs.circom.io) · [snarkjs](https://github.com/iden3/snarkjs) · [Noir docs](https://noir-lang.org/docs) · [RISC Zero docs](https://dev.risczero.com)
