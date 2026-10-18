# DINAH Storage — guía de implementación (puntos 140–156)

Storage empieza con una API de clave/valor para bytes opacos, sin serializar structs Rust ni acoplar el contrato a un dominio. El módulo público `dinah::storage` contiene `Storage`, `MemoryStore`, `DiskStore` y `StorageError`. Es una base incremental, no una base de datos de producción.

## API común — punto 141

`Storage` expone `get(&self, key) -> Option<Vec<u8>>`, `put(&mut self, key, value)` y `delete(&mut self, key)`. Los valores devueltos son copias propiedad del llamador; no se devuelven referencias que dependan de la vida del store. `put` y las implementaciones concretas validan el límite actual de 16 MiB por clave y valor. Los errores de E/S y de formato se representan con `StorageError`.

La API es síncrona y requiere `&mut self` para mutaciones. No promete acceso concurrente, transacciones multi-clave, orden global entre procesos, índices secundarios, compactación, cifrado ni control de acceso. Las aplicaciones son responsables de validar y versionar sus payloads.

## Memoria — punto 142

`MemoryStore` usa `BTreeMap<Vec<u8>, Vec<u8>>`: claves binarias ordenadas lexicográficamente, reemplazo explícito al repetir `put`, lectura clonada y eliminación que informa si existía. `iter()` expone vistas prestadas de clave/valor en orden determinista, sin clonar cada entrada. Su contenido vive únicamente mientras viva la instancia y no se comparte entre procesos. `len` e `is_empty` son observacionales.

## Disco y formato — puntos 143–144

`DiskStore::open(path)` crea o abre un archivo de journal local y reconstruye el mapa aplicando sus registros completos en orden. La cabecera común tiene 14 bytes: magic `DNST`, versión de formato de un byte, opcode de un byte, longitud de clave `u32` little-endian y longitud de valor `u32` little-endian; después se guardan los bytes de clave y valor. Los registros nuevos usan formato v2 e incluyen al final un CRC-32 IEEE de 4 bytes. Los registros v1 históricos, sin checksum, siguen siendo legibles. Opcode 1 es PUT; opcode 2 es DELETE con valor vacío. Las longitudes se validan antes de procesar el registro y cada campo se limita a 16 MiB.

Este formato es interno y versionado, no una codificación portable de objetos Rust. El CRC-32 detecta corrupción accidental de registros v2 completos; no es criptográfico, no autentica el archivo y no protege frente a modificaciones deliberadas. Un checksum incorrecto en un registro completo produce error, no se salta silenciosamente. Un registro final incompleto, incluido un checksum truncado, se trata como escritura interrumpida: se conservan los registros anteriores y se trunca el sufijo antes de aceptar nuevas escrituras. Los registros v1 previos se leen sin checksum por compatibilidad.

## Persistencia y journal — puntos 145–146

Cada `put` o eliminación efectiva añade un registro y llama a `flush`; las actualizaciones posteriores reemplazan la clave durante replay. `delete` de una clave ausente no añade un registro. El journal es append-only y todavía no se compacta. `flush` solo vacía buffers de usuario: no se llama a `sync_all` y no se promete durabilidad ante pérdida de energía, fallo del kernel o caché del dispositivo. No se garantiza atomicidad frente a todos los tipos de fallo ni se coordinan múltiples instancias abiertas sobre el mismo archivo.

## Integridad y compatibilidad — punto 148

Los registros nuevos son v2 y almacenan CRC-32 IEEE sobre versión, opcode, longitudes y payload. Un checksum incorrecto en un registro completo devuelve `StorageError::InvalidFormat`; el replay no aplica ese registro. La longitud declarada se valida antes de calcular rangos, y un registro incompleto (incluido checksum parcial) se considera un sufijo truncado. Se conservan lectores de v1 sin checksum para no invalidar journals creados por la primera implementación. CRC-32 detecta daños accidentales, pero no es MAC, firma ni defensa frente a un atacante.

## Índices y estructuras — puntos 149–150

La decisión actual es un único índice primario `BTreeMap<Vec<u8>, Vec<u8>>` para lecturas O(log n), orden estable y escaneos ordenados O(n). `DiskStore` reconstruye el índice al abrir y las mutaciones actualizan el índice después de que el append y `flush` tengan éxito. `iter()` permite escanear por referencias sin clonar los valores. No se añade un índice secundario todavía: sin consultas por prefijo/campo de aplicación ni mediciones que justifiquen el coste, duplicar claves aumentaría memoria y complejidad de coherencia sin beneficio probado. No hay compactación del journal; el índice es en memoria y debe reconstruirse en cada apertura.

## Integración con Runtime — punto 151

`storage::runtime::StorageWorkQueue` acepta comandos `Put`/`Delete` propios y tiene capacidad fija no nula. Encolar no toca el store. `dispatch_next` quita un comando FIFO y lo aplica de forma síncrona mediante `Scheduler` en el hilo llamador; si la cola está llena, devuelve el comando rechazado para que el llamador lo conserve o reintente. Los errores de almacenamiento se conservan como resultado de tarea. No crea threads, no serializa automáticamente lecturas, no comparte el store y no aporta transacciones ni ejecución en segundo plano.

## Recuperación — punto 147

Al abrir, el store recorre los registros desde el principio y aplica PUT/DELETE en secuencia. Rechaza magic incorrecto, versión desconocida, opcode inválido, campos fuera de límite y DELETE con payload. Si el último registro está incompleto, recupera hasta el último límite válido y trunca el sufijo. Esta recuperación inicial no sustituye checksums, copias de seguridad, migraciones ni pruebas exhaustivas de corrupción.

## Ejemplo ejecutable

`examples/storage_basics.rs` demuestra el contrato común con `MemoryStore` y `DiskStore`: inserción y recorrido ordenado, reapertura del journal para recuperar valores y eliminación de una clave. Ejecútalo desde la raíz del repositorio con `cargo run --example storage_basics`. El ejemplo crea un nombre temporal único y elimina el journal al terminar. Este ejemplo es demostrativo; no cambia las garantías de durabilidad descritas arriba ni convierte el journal en una base de datos transaccional.

## Pruebas

`tests/storage.rs` cubre operaciones en memoria, persistencia/replay, recuperación de sufijos truncados, rechazo de cabecera malformada, corrupción detectada por checksum, truncado del checksum y lectura de registros v1. `tests/storage_runtime.rs` comprueba despacho FIFO y rechazo por capacidad; `tests/storage_concurrency.rs` verifica operaciones paralelas sobre journals independientes. No se prueba ni se promete acceso concurrente a un mismo archivo desde varias instancias.

## Siguiente trabajo

La suite `benches/storage.rs` mide operaciones agrupadas de MemoryStore y escaneo ordenado; CI ejecuta la suite en su matriz de cinco plataformas y archiva las mediciones por commit. Los resultados son específicos de cada máquina y no deben compararse entre plataformas como si fueran equivalentes. La implementación no se presenta como base de datos transaccional ni como garantía de persistencia fuerte. Antes de usar DiskStore para datos importantes, conserva copias de seguridad y asume que una pérdida de energía puede perder escrituras recientes. El formato del journal es interno; la lectura de v1 se conserva, pero no existe una herramienta de migración o compactación. Las garantías de concurrencia se limitan a la exclusión mutua que impone el ownership de Rust dentro de una instancia; abrir el mismo path varias veces no coordina escritores.
