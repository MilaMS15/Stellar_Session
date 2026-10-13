# Smart Contract - Soroban (Stellar)

Contrato inteligente desarrollado en Rust para la red Stellar (Soroban).

### Contract ID (Testnet)
CDNZBJHOU7V2ZCO7Y4PIREBJNSEPBMPX3KZZWB447FGVZEIVWWGGHTVV

### Funciones del Contrato
- `__constructor`: Inicializa el contrato asignando el administrador.
- `get_admin`: Retorna la dirección del administrador.
- `save_record`: Guarda un registro on-chain asociado a un usuario y clave.
- `get_record`: Consulta un registro almacenado.
- `get_count`: Retorna el contador total de registros guardados.
