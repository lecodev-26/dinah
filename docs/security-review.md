# DINAH — Modelo y revisión de seguridad (puntos 195–203, 205–206, 210)

Fecha de revisión: 2026-10-18. Alcance: superficie actual del crate, memoria, concurrencia, permisos, filesystem, networking, entrada del CLI, journal de Storage, salida JSON y ABI C escalar. Se añadieron pruebas unitarias de transiciones de Connection, replay de Storage, regresión del ABI y una carga paralela acotada sobre journals independientes. Se amplió el corpus determinista de entradas adversarias para replay y manifiestos; no se presenta como fuzzing coverage-guided. Esta revisión estática no es una auditoría externa ni demuestra ausencia de vulnerabilidades.

## 195 — Superficie de ataque

- El crate no declara dependencias de runtime en `[dependencies]`; la dependencia `wide` se usa solo como dependencia de desarrollo. Esto reduce la superficie de dependencias, pero no sustituye la auditoría periódica de lockfile/toolchain.
- `dinah init` usa `create_new(true)` y no reemplaza un manifiesto existente. Las operaciones de archivos se limitan al path que el usuario proporciona o al directorio actual; no se ejecutan comandos definidos en el manifiesto.
- `dinah run` solo valida el manifiesto. `dinah task` enumera claves; ninguna de las dos ejecuta las tareas declaradas.
- La entrada del manifiesto queda limitada a 1 MiB. No se afirma protección general frente a enlaces simbólicos, carreras del filesystem o paths maliciosos en todos los sistemas operativos.

## 196 — Entradas externas

- `DINAH.toml` se lee como UTF-8; errores de lectura se devuelven como errores de E/S. El parser soporta una sintaxis deliberadamente reducida y no es compatible con TOML completo.
- Se aplica un límite de 1 MiB antes de cargar el contenido completo. Los nombres de proyecto vacíos o no entrecomillados se rechazan.
- Las opciones desconocidas, formatos desconocidos y argumentos posicionales sobrantes se rechazan con salida no cero.
- Riesgo residual: el parser básico no valida todas las reglas TOML ni normaliza rutas; no debe usarse como validador de seguridad de configuraciones complejas.

## 197 — Serialización

- La salida JSON del CLI escapa comillas, barras inversas y saltos de línea; los controles restantes se sustituyen por espacios. La nueva exportación de métricas produce un array JSON determinista desde el snapshot ordenado y escapa nombres arbitrarios de métricas.
- No hay deserializador JSON ni decoder de wire format de networking declarado como implementado en esta revisión. No se debe asumir que un formato documentado ya tiene encoder/decoder.
- Riesgo residual: la exportación de métricas no fija un esquema de compatibilidad versionado; los consumidores deben tratarla como formato de diagnóstico local de esta versión.

## 198 — FFI

- `include/dinah.h` y `dinah::native::ffi` exponen solo consultas escalares de enteros de ancho fijo: versión ABI, flags y familia de plataforma. No aceptan punteros, buffers, callbacks ni ownership transferido.
- El crate mantiene `#![deny(unsafe_code)]`; el módulo FFI solo permite nombres de símbolos y no contiene bloques unsafe.
- El consumidor debe comprobar `dinah_abi_version()` antes de usar otras funciones. Los bits no definidos en `dinah_abi_features()` están reservados.
- Riesgo residual: sigue pendiente un smoke test con compilador y linker C reales; la revisión del header y pruebas Rust no sustituyen esa integración.

## Acciones pendientes

- Ejecutar fuzzing coverage-guided (libFuzzer) para parser de manifiesto y replay del journal; el corpus determinista actual solo es una primera barrera de regresión.
- Fuzzing y auditoría formal de dependencias siguen pendientes. Las pruebas de estrés/concurrencia actuales son acotadas; no incluyen model checking, escritores concurrentes sobre el mismo journal ni cargas prolongadas.
- Ejecutar el smoke test C/link en CI antes de declarar estable el ABI.
- Repetir esta revisión cuando se introduzcan ejecución de tareas, deserialización de red, acceso a archivos externos o APIs FFI basadas en punteros.


## 199 — Revisión de memoria

- `#![deny(unsafe_code)]` está habilitado a nivel de crate; el único `allow(unsafe_code)` se limita al módulo de nombres de símbolos C, que no contiene bloques unsafe ni punteros.
- `Buffer<T>` y `SoaBuffer<A, B>` exponen solo elementos inicializados a través de referencias de Rust. Sus capacidades son límites de elementos, no límites de bytes ni garantía de memoria residente.
- La mayoría de las reservas de `Buffer` son fallibles; otros contenedores estándar y clones de claves/valores pueden abortar o entrar en rutas de asignación infalibles según el allocator. No se afirma resistencia total a agotamiento de memoria.
- Riesgo residual: no existe fuzzing ni Miri en CI; las pruebas de invariantes y el sistema de tipos no sustituyen esas herramientas.

## 200 — Revisión de concurrencia

- Las APIs mutables de Storage, Connection y buffers requieren `&mut self`; no prometen uso concurrente de la misma instancia sin sincronización externa.
- `MetricsRegistry` usa `Mutex`; el poisoning se convierte en error. `TaskQueue` es deliberadamente local y no sincronizado. Los tests de Storage con threads abren journals independientes, no el mismo archivo.
- El scheduler usa threads acotados por la llamada, pero un panic de worker puede propagarse por los puntos de unión. No hay promesa de cancelación de tareas de sistema operativo ni aislamiento de procesos.
- Riesgo residual: faltan pruebas de estrés prolongadas, loom/model checking y una política para múltiples escritores que abran simultáneamente el mismo journal.

## 201 — Revisión de permisos

- DINAH no eleva privilegios, cambia permisos de archivos ni crea servicios privilegiados. El CLI usa los permisos efectivos del proceso y devuelve errores de E/S cuando el sistema operativo deniega una operación.
- El manifiesto nuevo se crea con `create_new(true)`, lo que evita reemplazar un archivo existente; los permisos finales dependen del sistema operativo y su umask.
- Las conexiones TCP pueden enlazar o conectar direcciones elegidas por el llamador. No existe autenticación, autorización, cifrado ni control de peers integrado.
- Recomendación operativa: ejecutar con privilegios mínimos, limitar los paths de salida y no exponer endpoints a redes no confiables sin controles en la aplicación.

## 202 — Revisión del filesystem

- El CLI acepta un path proporcionado por el usuario y usa `DINAH.toml` dentro de ese directorio. El límite de 1 MiB reduce entradas excesivas, pero `metadata` seguido de lectura no elimina carreras TOCTOU ni garantiza el comportamiento de symlinks en todos los sistemas.
- `DiskStore` abre el path indicado con lectura, append y create. No canonicaliza ni confina el path a un sandbox, no coordina instancias que abran el mismo journal y no llama a `sync_all`.
- El journal se lee completo a memoria al abrirlo; aunque cada campo está limitado a 16 MiB, el tamaño total del archivo no tiene un límite global en esta versión. Los journals deben proceder de una ubicación confiable.
- Riesgo residual: añadir límite global configurable, pruebas de permisos/symlinks por plataforma y política explícita para apertura de journal ya existente.

## 203 — Revisión de networking

- La capa actual es TCP bloqueante de bajo nivel. No añade framing, decoder de mensajes, autenticación, TLS, rate limiting, reconexión automática ni tareas de fondo.
- Los métodos de lectura pueden devolver datos parciales y el llamador debe definir límites de mensajes. `set_read_timeout` y `set_write_timeout` están disponibles, pero no se aplican por defecto.
- No aceptar bytes recibidos como objetos confiables ni exponer un listener a redes no confiables sin límites de tamaño, tiempo, identidad y frecuencia implementados por la aplicación.

## 205 — Pruebas de corrupción

- `tests/storage.rs` comprueba magic inválido, checksum v2 incorrecto, sufijos incompletos, checksum truncado y compatibilidad con v1. Se añade cobertura para longitudes declaradas por encima del límite y para valores de versión/opcode inválidos.
- CRC-32 sirve para detectar corrupción accidental de registros completos, no alteraciones deliberadas. Una corrupción completa debe fallar; solo el sufijo final incompleto se recupera como escritura interrumpida.

## 206 — Pruebas de estrés

- La suite añadida a `src/observability.rs` estresa incrementos concurrentes en el registro compartido con varios hilos y verifica el resultado agregado.
- Es una prueba de estrés acotada y determinista, no un benchmark de rendimiento ni una prueba de agotamiento de recursos. Las pruebas prolongadas, fuzzing y model checking siguen pendientes.

## 210 — Modelo de seguridad

### Activos

Integridad de memoria, archivos de configuración y journals, comportamiento determinista de las APIs, disponibilidad del proceso y límites de confianza entre la aplicación y conexiones externas.

### Límites de confianza

- Los archivos del usuario, datos de red y archivos de journal existentes se consideran entradas no confiables.
- El sistema operativo controla permisos, sockets y semántica del filesystem; DINAH no evita por sí solo rutas fuera de un directorio permitido.
- Las aplicaciones consumidoras validan payloads, definen framing y autenticación de red y gestionan copias de seguridad de Storage.

### Garantías actuales y no-garantías

Se priorizan APIs Rust seguras, límites de campo, escape JSON, detección de corrupción accidental y errores explícitos. No se garantiza seguridad frente a denegación de servicio, pérdida de energía, escritores concurrentes al mismo journal, pares de red maliciosos, vulnerabilidades de dependencias o errores del sistema operativo.

### Controles pendientes

- Fuzzing de parser de manifiesto, replay del journal y futuros codecs de red.
- Auditoría de dependencias/toolchain y revisión periódica de avisos de seguridad.
- Pruebas de estrés de larga duración, permisos y symlinks en las plataformas soportadas.
- Smoke test real de compilación y enlace C en CI antes de declarar estable la ABI.


## Cobertura adicional de pruebas (puntos 207–208, 217–219, 221–222 y 224)

- `src/network.rs` comprueba transiciones de estado Open/ReadClosed/WriteClosed/Closed sin crear sockets.
- `src/storage.rs` repite replay del mismo journal y valida el rechazo de un DELETE con payload.
- `src/native.rs` conserva tests unitarios para HostPlatform, PosixPlatform (en Unix) y los símbolos ABI escalares.
- `tests/robustness_regression.rs` comprueba invariantes ABI, orden y overwrite repetibles, y seis workers sobre archivos de journal separados con 128 actualizaciones cada uno y reapertura posterior.
- Estas pruebas mejoran regresión y concurrencia acotada; no sustituyen fuzzing, model checking, pruebas de múltiples escritores sobre el mismo archivo ni un smoke test de compilador/linker C.


## Trabajo de robustez del corpus (punto 204, en progreso)

- Storage ahora comprueba entradas binarias pseudoaleatorias deterministas de hasta 512 bytes, todas las truncaciones de un registro válido, mutaciones de cada byte y sufijos truncados tras registros concatenados. Los tests comprueban ausencia de panic, no que cada entrada deba ser aceptada.
- El CLI cubre nombre vacío/no entrecomillado, campos duplicados y asignaciones fuera de secciones conocidas. La semántica sigue siendo el subconjunto documentado, no TOML completo.
- Limitación explícita: estas pruebas son corpus determinista y no reemplazan libFuzzer, cobertura guiada, sanitizadores ni sesiones de larga duración.

## Controles automatizados de CI (2026-10-18)

- `.github/workflows/quality-security.yml` ejecuta `cargo audit` sobre el lockfile, genera un artefacto LCOV con `cargo llvm-cov` y compila/enlaza/ejecuta un consumidor C mínimo contra `libdinah.a`. La ejecución del commit `fcde532` pasó.
- `tests/ffi_smoke.c` comprueba los tres símbolos escalares públicos y las condiciones básicas de ABI; la compilación, el enlace y la ejecución pasaron en CI.
- El artefacto LCOV se generó y subió correctamente, pero aún no se ha fijado un umbral de cobertura. La auditoría automatizada no sustituye una revisión manual de licencias ni una auditoría de seguridad externa.


## Matriz de CI validada (2026-10-18)

- El workflow `.github/workflows/ci.yml` incluye una matriz explícita de Linux x86-64 y Linux ARM64. Para ambos runners, CI confirmó formato, tests de librería/integración, Clippy con warnings como errores y generación de rustdoc sin warnings en el commit `7a5e076`.
- El workflow de calidad/seguridad del commit anterior `fcde532` confirmó auditoría de dependencias, compilación/enlace/ejecución del consumidor C y generación/subida de LCOV. El LCOV se genera como artefacto, pero todavía no se aplica un umbral cuantitativo de cobertura.
- La matriz de pruebas no debe confundirse con la matriz de benchmarks: Windows y macOS se incluyen en benchmarks, pero no están cubiertos por este job de tests/Clippy/rustdoc.
