# dinah Storage — primeros contratos (puntos 140–147)

Storage empieza con una API de clave/valor para bytes opacos, sin serializar structs Rust ni acoplar el contrato a un dominio. El módulo público `dinah::storage` contiene `Storage`, `MemoryStore`, `DiskStore` y `StorageError`. Es una base incremental, no una base de datos de producción.

## API común — punto 141

`Storage` expone `get(&self, key) -> Option<Vec<u8>>`, `put(&mut self, key, value)` y `delete(&mut self, key)`. Los valores devueltos son copias propiedad del llamador; no se devuelven referencias que dependan de la vida del store. `put` y las implementaciones concretas validan el límite actual de 16 MiB por clave y valor. Los errores de E/S y de formato se representan con `StorageError`.

La API es síncrona y requiere `&mut self` para mutaciones. No promete acceso concurrente, transacciones multi-clave, orden global entre procesos, índices secundarios, compactación, cifrado ni control de acceso. Las aplicaciones son responsables de validar y versionar sus payloads.

## Memoria — punto 142

`MemoryStore` usa `BTreeMap<Vec<u8>, Vec<u8>>`: claves binarias ordenadas lexicográficamente, reemplazo explícito al repetir `put`, lectura clonada y eliminación que informa si existía. Su contenido vive únicamente mientras viva la instancia y no se comparte entre procesos. `len` e `is_empty` son observacionales.

## Disco y formato — puntos 143–144

`DiskStore::open(path)` crea o abre un archivo de journal local y reconstruye el mapa aplicando sus registros completos en orden. Cada registro v1 tiene una cabecera de 14 bytes: magic `DNST`, versión de formato de un byte, opcode de un byte, longitud de clave `u32` little-endian y longitud de valor `u32` little-endian; a continuación se guardan los bytes de clave y valor. Opcode 1 es PUT; opcode 2 es DELETE con valor vacío. Las longitudes se validan antes de procesar el registro y cada campo se limita a 16 MiB.

Este formato es interno y versionado, no una codificación portable de objetos Rust. No contiene todavía checksum ni MAC; un archivo con registros completos malformados produce un error. Un registro final incompleto se trata como una escritura interrumpida: al abrir se conservan los registros anteriores y se trunca el sufijo incompleto antes de aceptar nuevas escrituras. El formato no puede detectar todas las corrupciones silenciosas; integridad/checksums queda para el punto 148.

## Persistencia y journal — puntos 145–146

Cada `put` o eliminación efectiva añade un registro y llama a `flush`; las actualizaciones posteriores reemplazan la clave durante replay. `delete` de una clave ausente no añade un registro. El journal es append-only y todavía no se compacta. `flush` solo vacía buffers de usuario: no se llama a `sync_all` y no se promete durabilidad ante pérdida de energía, fallo del kernel o caché del dispositivo. No se garantiza atomicidad frente a todos los tipos de fallo ni se coordinan múltiples instancias abiertas sobre el mismo archivo.

## Recuperación — punto 147

Al abrir, el store recorre los registros desde el principio y aplica PUT/DELETE en secuencia. Rechaza magic incorrecto, versión desconocida, opcode inválido, campos fuera de límite y DELETE con payload. Si el último registro está incompleto, recupera hasta el último límite válido y trunca el sufijo. Esta recuperación inicial no sustituye checksums, copias de seguridad, migraciones ni pruebas exhaustivas de corrupción.

## Pruebas

`tests/storage.rs` cubre reemplazo y borrado en memoria, persistencia/replay en disco, recuperación ante sufijo truncado y rechazo de una cabecera completa malformada. Las pruebas usan archivos temporales propios y no dependen de servicios externos.

## Siguiente trabajo

Quedan fuera de estos puntos iniciales: checksums/integridad (148), índices y estructuras especializadas (149–150), integración con Runtime (151), pruebas más amplias de corrupción/concurrencia y benchmarks (152–155), y documentación final de Storage (156). La implementación no se presenta como una base de datos transaccional ni como una garantía de persistencia fuerte.
