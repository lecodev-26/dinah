# dinah — Checklist maestro

Este archivo es el registro operativo vivo. No se renumeran ni reutilizan IDs. Para completar un punto, debe existir evidencia suficiente en código, pruebas, documentación o resultados según corresponda.

## Estado inicial

- Total de puntos numerados: 287
- Completados: 244
- En progreso: 17
- Pendientes: 26

Estados: [ ] pendiente · [>] en progreso · [x] completado · [~] obsoleto/sustituido (explicar motivo).

## FASE 0 — Fundación y reglas del proyecto

- [x] 001 — Definir identidad oficial de dinah
- [x] 002 — Definir propósito técnico
- [x] 003 — Definir alcance inicial
- [x] 004 — Definir qué NO es dinah
- [x] 005 — Mantener AXIOMSYSTEMS fuera
- [x] 006 — Mantener AXIOMSYSTEMS2 fuera
- [x] 007 — Mantener AXIOM-NEXUS-Q fuera
- [x] 008 — Definir Rust como lenguaje principal
- [x] 009 — Definir política para C/C++
- [x] 010 — Definir política para Assembly
- [x] 011 — Definir política de código externo reutilizado
- [x] 012 — Definir política de dependencias
- [x] 013 — Definir política de testing
- [x] 014 — Definir política de benchmarks
- [x] 015 — Definir política de documentación
- [x] 016 — Definir licencia dual MIT OR Apache-2.0, añadir textos de licencia y declararla en Cargo.toml
- [x] 017 — Crear identidad visual básica en SVG y documentar el uso de dinah en minúsculas
- [x] 018 — Crear README inicial
- [x] 019 — Crear roadmap
- [x] 020 — Crear checklist de puntos

## FASE 1 — Core

- [x] 021 — Crear crate principal
- [x] 022 — Definir types
- [x] 023 — Definir errores
- [x] 024 — Definir resultados internos
- [x] 025 — Definir configuración
- [x] 026 — Definir IDs internos
- [x] 027 — Definir estados internos
- [x] 028 — Definir abstracciones comunes
- [x] 029 — Definir traits fundamentales
- [x] 030 — Definir invariantes del sistema
- [x] 031 — Crear tests del Core
- [x] 032 — Crear documentación del Core
- [ ] 033 — APLAZADO: crear primeros benchmarks cuando exista una carga real del Core que se pueda medir sin claims sintéticos
- [x] 034 — Revisar API pública
- [ ] 035 — APLAZADO: estabilizar Core v0.1 cuando exista un consumidor real que justifique `HasCoreId` (auditoría inicial en `docs/core.md`)

## FASE 2 — Runtime

- [x] 036 — Diseñar modelo de ejecución
- [x] 037 — Definir Task
- [x] 038 — Definir TaskId
- [x] 039 — Definir estado de tarea
- [x] 040 — Definir Worker
- [x] 041 — Definir WorkerId
- [x] 042 — Crear executor
- [x] 043 — Crear scheduler
- [x] 044 — Crear cola de tareas
- [x] 045 — Crear asignación de tareas
- [x] 046 — Crear ejecución secuencial
- [x] 047 — Crear ejecución paralela
- [x] 048 — Crear sincronización
- [x] 049 — Gestionar errores de ejecución
- [x] 050 — Gestionar cancelación
- [x] 051 — Gestionar finalización
- [x] 052 — Añadir métricas
- [x] 053 — Añadir tracing
- [x] 054 — Testear concurrencia
- [x] 055 — Testear condiciones de carrera
- [x] 056 — Testear scheduler
- [x] 057 — Testear executor
- [x] 058 — Crear benchmarks
- [x] 059 — Optimizar scheduler
- [x] 060 — Estabilizar Runtime v0.1

## FASE 3 — Memory

- [x] 061 — Diseñar modelo de memoria
- [x] 062 — Definir buffers
- [x] 063 — Definir ownership interno
- [x] 064 — Definir allocation abstractions
- [x] 065 — Definir pools
- [x] 066 — Definir reutilización de memoria
- [x] 067 — Gestionar alineación
- [x] 068 — Investigar SIMD-friendly layouts
- [x] 069 — Crear estructuras contiguas
- [x] 070 — Crear estructuras data-oriented
- [x] 071 — Añadir sincronización
- [x] 072 — Testear seguridad de memoria
- [x] 073 — Testear concurrencia
- [x] 074 — Crear benchmarks
- [x] 075 — Optimizar allocations
- [x] 076 — Documentar Memory

## FASE 4 — Compute

- [x] 077 — Diseñar Compute API
- [x] 078 — Definir primitivas de procesamiento (API secuencial `for_each`, `map` y `try_reduce`; tests y docs; CI verde: https://github.com/lecodev-26/dinah/actions/runs/38123834702)
- [x] 079 — Crear procesamiento por lotes (`for_each_batch`, `BatchError`, tests de tamaño cero y último lote parcial; CI verde: https://github.com/lecodev-26/dinah/actions/runs/38124079211)
- [x] 080 — Crear parallel iteration (`for_each_parallel`, particiones disjuntas, integración con `Scheduler::run_parallel`, tests de cobertura/entrada vacía/cero workers; CI verde: https://github.com/lecodev-26/dinah/actions/runs/38153730624)
- [x] 081 — Crear reducción paralela (`try_reduce_parallel`, identidad por partición, combinación en orden, propagación del primer error por orden de entrada; CI verde: https://github.com/lecodev-26/dinah/actions/runs/38154040501)
- [x] 082 — Crear pipelines (`ComputePipeline` inmediato sobre slice prestado; etapas secuenciales, por lotes y paralelas; errores tipados y tests de orden/corte/ownership; CI verde: https://github.com/lecodev-26/dinah/actions/runs/38154774490)
- [x] 083 — Investigar SIMD (documentadas opciones, restricciones del crate y criterios de validación; se mantiene la ruta escalar hasta medir un caso real; CI verde: https://github.com/lecodev-26/dinah/actions/runs/38155044022)
- [ ] 084 — APLAZADO: implementar SIMD solo cuando un kernel de producción y benchmarks representativos justifiquen el cambio; evidencia mixta en `docs/simd-research.md`
- [ ] 085 — APLAZADO: optimizar estructuras data-oriented cuando exista una carga real; falta medir construcción, conversión y escrituras además de los benchmarks exploratorios
- [x] 086 — Añadir métricas de rendimiento (todas las suites benchmark reportan mínimo, mediana y máximo observados por caso, además de métricas normalizadas cuando aplica; el rango no es intervalo de confianza)
- [x] 087 — Crear benchmarks CPU (suite `benches/compute.rs`: bucle escalar, API secuencial, rutas paralelas de 2/4 workers y candidato `wide::f32x4`; 8 tamaños, 11 muestras por caso y mediana; ejecución multiplataforma archivada en el punto 088)
- [x] 088 — Comparar secuencial/paralelo (benchmark release archivado en cinco runners; salida escalar/paralela verificada; análisis documentado: en el kernel ligero, 2 workers tardan 4,0–10,3× más incluso con 262.144 elementos y 4 workers 6,3–16,6× más; no se justifica paralelizar automáticamente este kernel)
- [x] 089 — Testear determinismo (tests públicos repetidos en Core, Compute secuencial/paralelo/pipelines, Runtime —orden FIFO/resultados, cancelación, estados, métricas, trazas, `Shared<T>`, `TaskQueue` y `TaskFailure`— y Memory —secuencias de Buffer/SoaBuffer, errores sin mutación, alineación y capacidad cero—; CI y auditoría documental verdes para el commit `7171343`)
- [x] 090 — Documentar Compute (guía pública `docs/compute.md` enlazada desde README; ejemplos de uso y contratos de las APIs secuenciales, por lotes, paralelas, reducciones y pipeline; límites de ownership, errores, determinismo, pánicos y rendimiento documentados; CI/auditoría de la guía confirmadas en `7171343`)

## FASE 5 — Data / ECS

- [x] 091 — Diseñar modelo ECS (decisiones iniciales documentadas en `docs/ecs.md`: sparse sets tipados, Entity con generación, ownership de World, contratos de query, seguridad sin `unsafe`, invariantes y secuencia de integración; todavía no declara APIs implementadas)
- [x] 092 — Definir Entity (`dinah::ecs::Entity` con índice/generación privados, accesores públicos, identidad/hash sensible a generación y pruebas de tamaño/valores límite; la validación de handles vivos queda para World)
- [x] 093 — Definir Component (`dinah::ecs::Component` público con implementación blanket para todo tipo Rust `'static`; sin requisito prematuro `Send + Sync`, validado con tests unitarios y de integración)
- [x] 094 — Definir World (`World::spawn/despawn/contains/len/is_empty`, errores tipados, reutilización con generación incrementada y retirada segura ante overflow; tests unitarios y públicos)
- [x] 095 — Definir queries (contrato documentado: selección por componentes requeridos, semántica de resultados vacíos, préstamos/mutabilidad, orden no estable y casos de aceptación; queries públicas aún pendientes)
- [x] 096 — Crear almacenamiento contiguo (`ComponentStorage<T>` interno con columnas contiguas alineadas, sparse index validado por generación, reemplazo, eliminación `swap_remove` y reservas previas para evitar desalineación por fallo de asignación; aún no integrado con `World`)
- [x] 097 — Crear iteración eficiente (`ComponentStorage::iter/iter_mut` recorren simultáneamente las columnas contiguas, devuelven pares entidad/componente alineados y permiten mutación segura de filas sin cambio estructural; tests para orden físico, storage vacío y actualización mutable)
- [x] 098 — Integrar ECS con Runtime (`World::run_system` despacha una closure con préstamo exclusivo de `World` mediante `Scheduler::schedule`; conserva separados el error del scheduler y el del sistema, con tests unitarios y públicos; integración síncrona, sin prometer paralelismo)
- [x] 099 — Integrar ECS con Compute (`ComponentStorage::for_each_mut` delega en `compute::for_each`, preserva el emparejamiento entidad/componente y no permite cambios estructurales; tests de alineación, mutación y storage vacío; integración interna secuencial, sin paralelismo ni API pública de World)
- [ ] 100 — APLAZADO: crear sistemas paralelos tras disponer de una API segura de ejecución/query que valide declaraciones y aísle cambios estructurales; metadatos y planificador determinista ya existen
- [x] 101 — Testear consistencia (tests de invariantes de `World` para contador de entidades vivas, slots retirados y lista libre sin duplicados; tests de `ComponentStorage` para alineación de columnas, índices sparse y generaciones obsoletas tras inserciones, reemplazos, `swap_remove` y reutilización)
- [x] 102 — Benchmark ECS (benchmarks públicos de ciclo de vida de `World` para spawn, despawn, reutilización de slots y `contains`; CI y resultados de cinco plataformas registrados en `docs/ecs-benchmarks.md`)
- [ ] 103 — APLAZADO: optimizar acceso a memoria después de implementar iteración pública/queries y obtener resultados multiplataforma con cargas representativas
- [x] 104 — Documentar arquitectura ECS (`docs/ecs.md` auditada contra la API actual: almacenamiento registrado por `TypeId`, operaciones públicas de componentes, limpieza en `despawn`, errores, ejemplo de uso y límites explícitos de queries/paralelismo)

## FASE 6 — Simulation

- [x] 105 — Diseñar Simulation API (`docs/simulation.md` fija responsabilidades, contrato temporal propuesto, errores, determinismo, integración por etapas y criterios de aceptación; las firmas se identifican explícitamente como conceptuales)
- [x] 106 — Integrar física ASCII: `simulation::ascii_physics` ofrece rejilla acotada, partículas discretas, obstáculos, rebote y renderizado; tests de límites y colisiones. CI, tests, Clippy y documentación pública verificados en `f7591b2`.
- [x] 107 — Integrar simulación de fluidos acotada: `simulation::fluid::ScalarFluid` implementa difusión escalar 2D explícita, buffer doble y borde sin flujo; tests de estabilidad, finitud y conservación aproximada. No es CFD general. CI, tests, Clippy y documentación pública verificados en `f7591b2`.
- [x] 108 — Integrar autómatas celulares: `simulation::cellular::CellularGrid` implementa Conway Life, vecindad Moore, bordes muertos o wrap y actualización síncrona con buffer doble; tests de blinker, límites y validación. CI, tests, Clippy y documentación pública verificados en `f7591b2`.
- [x] 109 — Integrar raycasting 2D: `simulation::raycast` ofrece rayos y AABB con intersección por slabs, distancia paramétrica, origen interior y miss; tests de impacto, paralelo, miss y dirección cero. CI, tests, Clippy y documentación pública verificados en `f7591b2`.
- [x] 110 — Integrar enjambres: `simulation::swarm::Flock2D` implementa flocking 2D acotado con vecindad por radio, actualización desde snapshot y límite de velocidad; tests de replay, velocidad y valores finitos. CI, tests, Clippy y documentación pública verificados en `f7591b2`.
- [x] 111 — Definir timestep (`src/simulation.rs`: `SimulationClock` con delta fijo validado, tick y elapsed explícitos, overflow comprobado y tests de avance atómico)
- [x] 112 — Definir actualización determinista (`Simulation<S>::step` ejecuta una actualización síncrona por tick; conserva el reloj ante error y documenta que no revierte mutaciones del estado)
- [x] 113 — Crear sistema de entidades simuladas (`SimulationEntities<T>` con IDs monotónicos, acceso/mutación/eliminación, orden de iteración estable y error de agotamiento)
- [x] 114 — Integrar Simulation con ECS (`SimulationEntityBindings` conecta IDs simulados con `ecs::Entity`, valida entidades vivas, evita asociaciones duplicadas y rechaza handles obsoletos al resolver)
- [x] 115 — Integrar Simulation con Compute (`SimulationEntities::for_each_mut` delega en `compute::for_each`, conserva el orden de inserción y mantiene ejecución secuencial explícita)
- [x] 116 — Crear pruebas deterministas (replay de una secuencia explícita de entradas en dos ejecuciones, compara la traza de ticks y el estado por ID; sin reloj real ni aleatoriedad implícita)
- [x] 117 — Crear benchmarks (suite ejecutada en CI y resultados/entornos archivados para Linux x86-64, Linux ARM64, Windows x86-64, macOS x86-64 y macOS ARM64)
- [x] 118 — Crear ejemplos (`examples/simulation_basics.rs`, ejemplo ejecutable de timestep fijo y entidades; CI pasó en `41a2f6f`)
- [x] 119 — Documentar Simulation (API real, límites, semántica de error, ejemplo ejecutable y resultados medidos en `docs/simulation.md`)

## FASE 7 — Networking

- [x] 120 — Diseñar Network API (`docs/network.md` fija capas, ownership, framing, límites de mensajes, errores, tiempo, concurrencia, cierre, seguridad, determinismo y decisiones aplazadas; identifica propuestas sin presentarlas como API implementada)
- [x] 121 — Definir transporte (contrato bloqueante de byte stream: lectura parcial/EOF, escritura completa y fallo parcial, bloqueo, timeout explícito, cierre, ownership y límites documentados en `docs/network.md`; sin afirmar sockets implementados)
- [x] 122 — Definir mensajes (envelope conceptual con versión de protocolo, tipo estable, versión de mensaje y payload opaco; reglas de validación/evolución y límites documentados en `docs/network.md`; la codificación binaria se concreta en el punto 123, sin afirmar implementación)
- [x] 123 — Definir serialización (wire format v1 documentado: prefijo de longitud u32 big-endian, cabecera de 6 bytes, frame máximo de 1 MiB, límites/errores y evolución versionada; solo diseño, sin afirmar encoder/decoder implementados)
- [x] 124 — Crear conexiones (`dinah::network::Connection` implementa TCP bloqueante sobre `TcpStream`, conexión con timeout, lectura/escritura, direcciones y shutdown; pruebas loopback en `tests/network_connection.rs`; sin framing integrado ni reconexión automática)
- [x] 125 — Crear endpoints (`dinah::network::Endpoint` implementa bind TCP, consulta de dirección local, accept que devuelve `Connection` y modo no bloqueante explícito; pruebas loopback para aceptación/intercambio y `WouldBlock`; sin gestión de estado ni tareas de fondo)
- [x] 126 — Crear gestión de estado (ConnectionState local para estados Open, ReadClosed, WriteClosed y Closed)
- [x] 127 — Investigar client-side prediction (modelo autoritativo, inputs numerados, reconciliación conceptual, límites y plan 128–131 documentados en `docs/client-side-prediction.md`; sin implementación de prediction)
- [x] 128 — Implementar prediction (`network::prediction::PredictionBuffer` aplica inputs localmente con secuencias monotónicas y ticks explícitos)
- [x] 129 — Implementar reconciliation (snapshot autoritativo, reconocimiento de secuencia, rechazo de ACK futuro e ignorado de snapshots obsoletos)
- [x] 130 — Implementar history/state buffers (historial de inputs pendientes acotado; capacidad cero y overflow de historial rechazados sin descartar inputs silenciosamente)
- [x] 131 — Implementar deterministic resimulation (replay en orden con el callback de transición suministrado; tests de repetibilidad; no garantiza determinismo multiplataforma)
- [x] 132 — Gestionar pérdida de paquetes (outbox acotado de mensajes con secuencia, ACK acumulativo y reintentos por tick; política de nivel de aplicación, no sustituye la retransmisión de segmentos TCP)
- [x] 133 — Gestionar latencia (`LatencyEstimator` con RTT suavizado entero, mínimo, máximo y contador; muestras aportadas por el llamador, sin reloj oculto)
- [x] 134 — Gestionar desconexiones (`ConnectionMonitor` con timeout lógico y `ReconnectBackoff` exponencial acotado; no reconecta sockets automáticamente)
- [x] 135 — Integrar Network con Runtime (`NetworkWorkQueue` acotada y despacho síncrono explícito mediante `Scheduler`; no crea tareas de fondo)
- [x] 136 — Integrar Network con Simulation (el simulador puede liberar entregas según el tick lógico de `Simulation`; el consumidor aplica los mensajes explícitamente)
- [x] 137 — Crear tests de red (`tests/network_advanced.rs` cubre ACK/reintentos, saturación, RTT, liveness, backoff, Runtime y escenarios deterministas de pérdida/latencia)
- [x] 138 — Crear simulador de latencia (`NetworkSimulator` con latencia en ticks, pérdida scripted y orden estable por tick/secuencia; sin aleatoriedad implícita)
- [x] 139 — Crear benchmarks (`benches/network.rs` mide RTT estimator y outbox; incluido en el runner multiplataforma, resultados pendientes de CI)
- [x] 140 — Documentar Network (estado real, límites, piezas implementadas y capacidades aún pendientes en `docs/network.md`)

## FASE 8 — Storage

- [x] 141 — Diseñar Storage API (`Storage` con `get`/`put`/`delete`, bytes opacos y errores tipados)
- [x] 142 — Definir almacenamiento en memoria (`MemoryStore` basado en `BTreeMap`, ownership explícito y operaciones básicas)
- [x] 143 — Definir almacenamiento en disco (`DiskStore` local con journal append-only; sin prometer durabilidad fuerte)
- [x] 144 — Crear formato interno (registros versionados DNST, opcode y longitudes little-endian validadas)
- [x] 145 — Crear persistencia (PUT/DELETE en journal, flush de buffers y replay al abrir)
- [x] 146 — Crear journal (registros secuenciales; sin compactación ni coordinación entre procesos)
- [x] 147 — Crear recuperación (replay ordenado, rechazo de registros completos inválidos y truncado de sufijo incompleto)
- [x] 148 — Crear checksums/integridad (journal v2 añade CRC-32 IEEE por registro; replay rechaza checksum incorrecto y conserva lectura de registros v1)
- [x] 149 — Diseñar índices (decisión documentada: BTreeMap como índice primario ordenado; no se añade índice secundario sin patrón de consulta y evidencia de rendimiento)
- [x] 150 — Crear estructuras eficientes (`MemoryStore` y `DiskStore` exponen iteración ordenada por slices prestados sin clonar valores; índice BTreeMap reconstruido en replay)
- [x] 151 — Integrar Storage con Runtime (`StorageWorkQueue` de capacidad acotada acepta Put/Delete y despacha FIFO mediante Scheduler en el hilo llamador)
- [x] 152 — Testear corrupción (prueba de alteración de payload detectada por CRC-32 y rechazo de registros completos corruptos)
- [x] 153 — Testear recuperación (sufijo truncado incluido checksum parcial, truncado previo a nuevos appends y compatibilidad de replay v1)
- [x] 154 — Testear concurrencia (prueba paralela con journals independientes; no implica ni promete coordinación concurrente sobre un mismo archivo)
- [x] 155 — Crear benchmarks (`benches/storage.rs` mide operaciones agrupadas de MemoryStore y escaneo ordenado; añadido a CI benchmark multiplataforma y archivado)
- [x] 156 — Documentar Storage (guía final de API, formato v1/v2, CRC-32, recuperación, Runtime, benchmarks, límites de durabilidad y concurrencia)

## FASE 9 — Native / Hardware

- [x] 157 — Definir HAL (trait `native::Hal` para metadatos del host y tiempo monotónico; sin prometer acceso directo al hardware)
- [x] 158 — Definir abstracciones de plataforma (trait `Platform` y proveedor estándar `HostPlatform`, con límites documentados)
- [x] 159 — Crear capa POSIX/Linux (adaptador `native::posix` bajo `cfg(unix)` mediante APIs seguras de std; no declara soporte de syscalls Linux directas)
- [x] 160 — Diseñar FFI (contrato escalar versionado, ownership explícito y criterios para futuras APIs de punteros documentados)
- [x] 161 — Crear C ABI (consultas escalares versionadas, símbolos exportados, `include/dinah.h` y crate-type rlib/cdylib/staticlib; consumidor C compilado, enlazado y ejecutado correctamente en CI del commit `fcde532`)
- [x] 162 — Investigar integración C++ (recomendado consumir el header C desde C++; sin exponer clases, excepciones ni ownership Rust)
- [x] 163 — Investigar Assembly (criterios de evidencia, fallback, revisión ABI y CI por arquitectura; no se añadió assembly sin hotspots medidos)
- [x] 164 — Identificar hotspots reales (auditoría de los siete benchmarks y criterio de evidencia; no se declara hotspot sin resultados comparables)
- [ ] 165 — APLAZADO: optimizar hotspots cuando perfiles y mediciones comparables identifiquen un cuello de botella real
- [x] 166 — Evaluar SIMD nativo donde corresponda (el benchmark compara scalar y candidato portable `wide::f32x4`; no se activa SIMD específico sin mejora medida)
- [x] 167 — Evaluar instrucciones específicas cuando aporten beneficio (umbral documentado; no se añade assembly sin mejora repetible)
- [x] 168 — Testear distintas arquitecturas (matriz Linux x86-64/ARM64, Windows x86-64 y macOS x86-64/ARM64)
- [x] 169 — Documentar Native (límites, matriz, hotspots, SIMD/assembly y ABI)

## FASE 10 — CLI

- [x] 170 — Crear comando dinah (binario Rust con ayuda, versión, errores y códigos de salida)
- [x] 171 — Crear dinah init (crea `DINAH.toml` y no sobrescribe un manifiesto existente)
- [x] 172 — Crear dinah run (valida manifiesto por defecto; `--task NAME` ejecuta explícitamente el comando declarado, captura stdout/stderr en JSON y propaga el código de salida; sin sandbox)
- [x] 173 — Crear dinah task (enumera las claves de [tasks]; la ejecución requiere selección explícita mediante `--task NAME run`)
- [x] 174 — Crear dinah inspect (muestra metadatos básicos y número de tareas)
- [x] 175 — Crear dinah benchmark (enumera las siete suites; no ejecuta benchmarks)
- [x] 176 — Crear dinah doctor (validación básica de directorio y manifiesto)
- [x] 177 — Crear configuración CLI (--format text/json y --verbose)
- [x] 178 — Crear output estructurado (JSON para resultados de comandos)
- [x] 179 — Crear logs (diagnóstico verbose en stderr)
- [x] 180 — Crear errores amigables (mensajes accionables, errores de manifiesto/tarea y propagación del código de salida de la tarea)
- [x] 181 — Crear documentación CLI (comandos, límites y ejemplos)
- [x] 182 — Testear comandos (parsing, init, metadatos, manifiestos inválidos, selección de tareas, salida JSON y códigos de salida; CI pendiente para la última ampliación)

## FASE 11 — Observabilidad

- [x] 183 — Diseñar sistema de tracing (API opt-in documentada; reutiliza el buffer acotado de eventos del Runtime)
- [x] 184 — Añadir métricas Runtime (registro nombrado y explícito de contadores)
- [x] 185 — Añadir métricas Memory (registro nombrado y explícito de contadores/gauges)
- [x] 186 — Añadir métricas Compute (registro nombrado y explícito de contadores/gauges)
- [x] 187 — Añadir métricas Network (registro nombrado y explícito de contadores/gauges)
- [x] 188 — Añadir métricas Storage (registro nombrado y explícito de contadores/gauges)
- [x] 189 — Crear profiler hooks (medición explícita de operaciones y hook consumidor de muestras)
- [x] 190 — Crear diagnóstico (resumen de métricas, muestras de profiling y capacidades de buffers)
- [x] 191 — Crear modo debug (perfil de desarrollo Cargo documentado y separado del perfil optimizado; no es un modo runtime del CLI)
- [x] 192 — Crear modo release (uso de `cargo build --release` y perfil release de benchmarks documentado)
- [x] 193 — Crear exportación de métricas (`MetricsRegistry::export_json`, snapshot ordenado y escape de caracteres)
- [x] 194 — Crear benchmarks reproducibles (runner de siete suites registra entorno, toolchain, commit y timestamp; resultados específicos del host)

## FASE 12 — Seguridad y robustez

- [x] 195 — Revisar superficie de ataque (revisión inicial documentada; quedan pendientes fuzzing y auditorías formales)
- [x] 196 — Revisar entradas externas (límite de manifiesto de 1 MiB y pruebas; parser sigue siendo subconjunto TOML)
- [x] 197 — Revisar serialización (escape de exportación JSON y límites documentados; no implica decoder de red implementado)
- [x] 198 — Revisar FFI (revisión de ABI escalar/header más smoke test C real compilado, enlazado y ejecutado en CI del commit `fcde532`)
- [x] 199 — Revisar memoria (invariantes de buffers y uso de unsafe revisados; límites y riesgos de asignación documentados)
- [x] 200 — Revisar concurrencia (ownership, Mutex/poisoning, colas y límites de concurrencia documentados)
- [x] 201 — Revisar permisos (privilegios efectivos, create_new, sockets y recomendaciones de mínimo privilegio documentados)
- [x] 202 — Revisar filesystem (límites, TOCTOU, symlinks, journals sin límite global y falta de sync_all documentados)
- [x] 203 — Revisar networking (TCP bloqueante, ausencia de framing/TLS/auth/rate limit y responsabilidades del consumidor documentadas)
- [ ] 204 — APLAZADO: añadir fuzzing coverage-guided con libFuzzer cuando el parser del manifiesto tenga un punto de entrada reutilizable por el harness; el corpus determinista actual sigue siendo regresión, no fuzzing
- [x] 205 — Añadir tests de corrupción (longitudes excesivas, versión desconocida y opcode inválido; checksum y truncado ya cubiertos)
- [x] 206 — Añadir tests de estrés (8 hilos × 2000 incrementos sobre el registro compartido; estrés acotado, no benchmark)
- [x] 207 — Añadir tests de concurrencia (seis hilos operan sobre journals independientes y verifican actualizaciones/reapertura; no prueba multiwriter sobre un mismo archivo)
- [x] 208 — Añadir tests de determinismo (replay de bytes idénticos y orden/overwrite estable de MemoryStore; no garantiza determinismo temporal entre plataformas)
- [x] 209 — Auditoría de dependencias (el workflow `quality-security.yml` ejecuta `cargo audit` sobre el lockfile; ejecución CI correcta en el commit `fcde532`, sin afirmar que no existan avisos futuros)
- [x] 210 — Documentar modelo de seguridad (activos, límites de confianza, garantías, no-garantías y controles pendientes)

## FASE 13 — Testing

- [x] 211 — Unit tests Core (`src/core/tests.rs`: 15 tests unitarios; complementados por `tests/core_determinism.rs`; CI verde en `320c5d7`)
- [x] 212 — Unit tests Runtime (tests en `src/runtime.rs` y submódulos: executor, scheduler, queue, cancellation, estados, sincronización, métricas y tracing; complementados por tests públicos de contrato/determinismo; CI verde en `320c5d7`)
- [x] 213 — Unit tests Memory (`src/memory/buffer.rs` y `src/memory/soa.rs`: 26 tests unitarios; complementados por `tests/memory_determinism.rs`; CI verde en `320c5d7`)
- [x] 214 — Unit tests Compute (`src/compute.rs`: 22 tests unitarios; complementados por `tests/compute_determinism.rs`; CI verde en `320c5d7`)
- [x] 215 — Unit tests ECS (tests unitarios de access, component, entity, storage y world; complementados por `tests/ecs_contract.rs`; CI verde en `320c5d7`)
- [x] 216 — Unit tests Simulation (`src/simulation.rs`: 14 tests unitarios, incluidos timestep, errores, entidades, bindings y replay determinista; CI verde en `320c5d7`)
- [x] 217 — Unit tests Network (`src/network.rs`: transiciones half-close y consistencia de direcciones cerradas)
- [x] 218 — Unit tests Storage (`src/storage.rs`: replay determinista y rechazo de DELETE con payload)
- [x] 219 — Unit tests Native (`src/native.rs`: metadatos host, tiempo monotónico, proveedor Unix y contrato ABI escalar)
- [x] 220 — Integration tests (`tests/system_pipeline.rs` cubre Core/Runtime/Memory/Compute/ECS/Simulation/Storage/observabilidad, reconciliación de predicción, entrega simulada por tick y TCP loopback real hacia Simulation y persistencia/reapertura de DiskStore mediante Runtime; `tests/system_integration.rs` añade Network, Native y CLI real. Cierra la integración de las APIs existentes, sin afirmar framing, autenticación, autoridad de servidor ni una aplicación de producción; esas capacidades no están implementadas)
- [x] 221 — Concurrency tests (`tests/robustness_regression.rs`: seis workers aislados actualizan y reabren journals independientes; no comparte una instancia mutable)
- [x] 222 — Stress tests (6 workers × 128 actualizaciones, comprobación de 8 claves y replay al reabrir; stress acotado, no benchmark prolongado)
- [x] 223 — Determinism tests (contratos repetidos públicos de Core, Runtime, Memory y Compute, más replay de Simulation; no implica determinismo temporal ni equivalencia bit a bit entre arquitecturas)
- [x] 224 — Regression tests (`tests/robustness_regression.rs`: ABI escalar y orden/overwrite repetibles de MemoryStore)
- [x] 225 — Benchmark suite (siete suites release —Runtime, Memory, Compute, ECS, Simulation, Network y Storage— ejecutadas por `scripts/run_benchmarks.py` y archivadas por CI en cinco runners; Native no tiene suite independiente porque no hay hotspot medido que la justifique)
- [x] 226 — CI completa (CI principal, auditoría documental, archivo de artefactos y workflow de calidad/seguridad —auditoría de dependencias, smoke test C/link y LCOV— completados correctamente en `fcde532`; la matriz Linux queda separada en 228–229)
- [x] 227 — Coverage (el workflow de calidad aplica `cargo llvm-cov --fail-under-lines 85`; LCOV de `9debb1b`: 4.814/5.297 líneas cubiertas, 90,88 %, y 670/763 funciones, 87,81 %. Se fija 85 % de líneas como guardrail inicial con ~5,9 puntos de margen; no mide branches en este reporte ni sustituye revisión de rutas críticas)
- [x] 228 — Test matrix Linux (matriz de formato, tests, Clippy y rustdoc validada correctamente en Linux x86-64 y ARM64 por CI del commit `7a5e076`)
- [x] 229 — Test matrix arquitecturas soportadas (matriz de tests/Clippy/rustdoc validada en Linux x86-64 y ARM64; Windows/macOS siguen cubiertos por benchmarks, no por esta matriz de tests)

## FASE 14 — Optimización

- [x] 230 — Crear baseline (el runner de benchmarks genera `baseline.json` y `baseline-summary.txt` con commit, timestamp, entorno, toolchain, casos medidos y duración por suite; baseline específico del runner, sin mezclar plataformas)
- [x] 231 — Perfilar Runtime (perfilado basado en benchmarks release: executor, scheduler, colas, costes de arranque y comparación con pool persistente; no es un perfilador CPU muestreado)
- [x] 232 — Perfilar Memory (perfilado basado en benchmarks de Buffer/SoA, escritura, escaneo AoS/SoA y lock compartido; no mide conteo de allocations ni cache misses)
- [x] 233 — Perfilar Compute (suite release registra kernels escalares, lotes, rutas paralelas y candidato SIMD; resultados dependientes de workload/runner)
- [x] 234 — Perfilar ECS (suite release registra ciclo de vida, acceso denso/disperso, generaciones obsoletas y reemplazos)
- [x] 235 — Perfilar Simulation (suite release registra los kernels y tamaños definidos en `benches/simulation.rs`)
- [x] 236 — Perfilar Network (suite release registra estimación de latencia y operaciones de outbox/reliability)
- [x] 237 — Perfilar Storage (suite release registra operaciones de MemoryStore y escaneo ordenado)
- [>] 238 — Identificar hotspots (artefactos de `2d7e8b4` revisados en cinco runners y tres procesos Runtime por runner; coste de worker lifecycle visible en no-op, pero workload sintético y sin profiling de aplicación; falta workload representativo y perfil CPU real)
- [>] 239 — Reducir allocations innecesarias (se reutiliza capacidad del vector batch; benchmark de `74ad2f0` no mide allocations y no demuestra mejora consistente)
- [>] 240 — Reducir copias (se conserva el lote completo antes de despachar y el orden FIFO; benchmark comparativo no mide copias directamente)
- [>] 241 — Optimizar cache locality (sin mediciones de cache misses; las medianas y sus rangos no permiten inferir locality)
- [>] 242 — Optimizar paralelismo (en `2d7e8b4`, scheduler/no-op frente al pool persistente de benchmark: 10,5×–63,9× más lento según runner y número de workers; CPU-heavy muestra ratios menores y variables; comparación no equivalente, falta carga real y evaluar lifecycle/errores antes de diseñar un pool persistente)
- [>] 243 — Optimizar scheduling (canal por worker elimina receiver compartido con `Mutex`; comparación histórica `2aeee49`→`74ad2f0` mixta; tres procesos Runtime en cinco runners confirman coste elevado de worker lifecycle para no-op, sin aislar el coste de cada etapa ni demostrar mejora del cambio de canal)
- [>] 244 — Optimizar SIMD (suite Compute ofrece casos escalares y `wide::f32x4`; el nuevo rango min/mediana/max ayuda a ver dispersión, pero sigue sin justificar activar SIMD sin evidencia por runner y semántica numérica)
- [>] 245 — Benchmark antes/después (comparados `2aeee49` y `74ad2f0`; los artefactos de `2d7e8b4` ya tienen tres procesos por runner y resumen automático de medianas/rangos; falta comparar una modificación de código concreta con el mismo harness en ejecuciones emparejadas/controladas)
- [x] 246 — Documentar resultados (`docs/performance-baseline.md` registra comparación antes/después de `2aeee49`, ejemplos de medianas por runner y límites de inferencia; no afirma una mejora general)

## FASE 15 — Integración total

- [x] 247 — Integrar Core + Runtime (smoke test público valida límites de Core antes de orquestar el pipeline con Scheduler; `tests/system_pipeline.rs`)
- [x] 248 — Integrar Runtime + Memory (Scheduler procesa una entrada desde el buffer acotado; `tests/system_pipeline.rs`)
- [x] 249 — Integrar Runtime + Compute (Scheduler ejecuta una transformación mediante Compute; `tests/system_pipeline.rs`)
- [x] 250 — Integrar Compute + ECS (resultado transformado almacenado como componente ECS; `tests/system_pipeline.rs`)
- [x] 251 — Integrar ECS + Simulation (Simulation avanza el World con timestep fijo y actualiza el componente; `tests/system_pipeline.rs`)
- [x] 252 — Integrar Runtime + Network (`tests/system_integration.rs` valida cola acotada, despacho síncrono por Scheduler, ejecución diferida hasta dispatch y métrica explícita; no crea sockets ni hilos de fondo)
- [x] 253 — Integrar Runtime + Storage (cola de trabajo despachada por Scheduler aplica snapshot a MemoryStore; `tests/system_pipeline.rs`)
- [x] 254 — Integrar Native (`tests/system_integration.rs` consume metadatos de `HostPlatform` dentro de una operación del Scheduler; integración de capacidades host, no acceso directo al hardware)
- [x] 255 — Integrar CLI (`tests/system_integration.rs` invoca el binario real para `init` e `inspect --format json` y comprueba que `init` no sobrescribe el manifiesto)
- [x] 256 — Integrar observabilidad (el test reporta métricas explícitas de Runtime y Compute y verifica exportación JSON; `tests/system_pipeline.rs`)
- [>] 257 — Ejecutar sistema completo (smoke tests públicos integran Runtime/Network, Native, CLI, Storage, Simulation y observabilidad; el CLI ejecuta una tarea seleccionada y la integración comprueba escapes Unicode, tarea inexistente y exit code no cero; CI del commit `02e30b0` verde. No constituye una aplicación completa ni integra framing/auth de red)
- [x] 258 — Crear ejemplos completos (`examples/system_integration.rs` demuestra NetworkWorkQueue + Scheduler + Native + métricas; ejemplo intencionalmente acotado, sin prometer socket real ni integración de todos los subsistemas)
- [x] 259 — Crear tests end-to-end (smoke test público cubre un pipeline Core/Runtime/Memory/Compute/ECS/Simulation/Storage/observabilidad; no representa todavía la arquitectura completa ni subsistemas sin implementar)
- [x] 260 — Resolver dependencias circulares (CI ejecuta `scripts/check_module_cycles.py`, que inspecciona referencias fuente `crate::` entre módulos raíz y no detecta ciclos en el grafo actual; tests Python verifican grafos cíclicos/acíclicos. Es un guardrail de referencias, no prueba ausencia de dependencias conceptuales o de runtime.)
- [>] 261 — Revisar API pública (inventario inicial en `docs/public-api-review.md`: diez módulos públicos y CLI, contratos/límites por subsistema, riesgos de estabilidad y gates previos a freeze; `README.md` corrige contadores y claims obsoletos. Sigue abierto hasta acordar el surface v0.1 soportado, política de compatibilidad y revisión final de errores/ownership/pánicos)
- [>] 262 — Revisar arquitectura completa (matriz en `docs/integration-matrix.md` y guardrail CI del grafo de referencias entre módulos raíz; siguen pendientes framing de red, límites de integración y revisión global antes de congelar arquitectura)

## FASE 16 — dinah v0.1

- [ ] 263 — Congelar arquitectura v0.1
- [ ] 264 — Congelar API v0.1
- [>] 265 — Completar documentación (`docs/cli.md` documenta escapes Unicode, whitespace, límites del parser y ejecución shell; `docs/release-checklist.md` registra los requisitos de publicación; falta completar y revisar toda la documentación pública)
- [>] 266 — Completar ejemplos (añadidos ejemplos ejecutables de Compute, Runtime, buffers, ECS, Simulation, Storage, Networking e integración; el CI verde de `d5b6ce6` ejecuta Clippy con `--all-targets`, que compila los targets de ejemplo; falta ampliar cobertura de APIs/subsistemas y revisar cada ejemplo frente al contrato soportado)
- [>] 267 — Completar CLI (parser del subconjunto starter con escapes Unicode `\u`/`\U` validados y whitespace preservado; ejecución opt-in de tareas con salida/código propagados; regresiones para Unicode inválido y selección de tarea verificadas por CI verde en `02e30b0`, más test añadido para comandos vacíos/al solo whitespace; parser TOML completo y compatibilidad siguen pendientes)
- [>] 268 — Completar testing (tests para sintaxis malformada, duplicados, escapes Unicode válidos/inválidos, whitespace, selección de tarea, salida capturada, comandos vacíos y exit codes; umbral inicial del 85 % de líneas ya aplicado en CI y LCOV revisado en `9debb1b`; faltan cobertura de ramas/rutas críticas, escenarios de fuzzing coverage-guided y revisión completa del conjunto de pruebas)
- [ ] 269 — Completar benchmarks
- [>] 270 — Auditoría de seguridad (pruebas de escapes JSON, comentarios, sintaxis inválida, duplicados, validación de codepoints Unicode, corpus adversario y regresiones CLI verificadas por CI verde en `02e30b0`; no equivale a fuzzing coverage-guided ni auditoría externa)
- [x] 271 — Auditoría de dependencias (auditoría automatizada del lockfile ejecutada correctamente en CI del commit `fcde532`; no equivale a auditoría manual de licencias ni garantiza ausencia futura de vulnerabilidades)
- [>] 272 — Revisar licencia (metadatos de licencia dual y smoke test C del ABI revisados; falta comprobar formalmente notices, procedencia/licencias de terceros y contenido final del paquete)
- [ ] 273 — Preparar release (`docs/release-checklist.md` define bloqueos, comandos de inspección de paquete y verificación posterior; sigue sin autorizar ni ejecutar una publicación)
- [ ] 274 — Crear changelog (`CHANGELOG.md` iniciado como Unreleased; debe actualizarse con los cambios finales de la versión y la fecha real de publicación)
- [ ] 275 — Crear documentación pública (README enlaza el changelog y checklist; la guía de CLI enlaza los límites de release; falta revisión final de la documentación pública)
- [ ] 276 — Crear paquetes/binarios
- [ ] 277 — Crear release v0.1.0

## FASE 17 — dinah v0.2+

- [ ] 278 — Evaluar arquitectura real
- [ ] 279 — Evaluar rendimiento
- [ ] 280 — Evaluar limitaciones
- [ ] 281 — Recoger necesidades reales
- [ ] 282 — Diseñar v0.2
- [ ] 283 — Implementar mejoras
- [ ] 284 — Repetir ciclo de testing
- [ ] 285 — Repetir benchmarking
- [ ] 286 — Repetir auditoría
- [ ] 287 — Preparar siguiente release

---

## Registro de cambios de estado

Añadir aquí la fecha, el ID y la evidencia cuando un punto cambie a completado, en progreso u obsoleto. No borrar ni reutilizar IDs.

- 2026-10-11 — Punto 075 completado: `SoaBuffer::try_extend_from_slices`, pruebas de errores/límites, benchmark comparable y resultados archivados en `artifacts/benchmarks/memory-soa-batch-2026-10-11.txt`; CI: https://github.com/lecodev-26/dinah/actions/runs/38096865096.
- 2026-10-11 — Punto 076 completado: guía práctica `docs/memory.md`, enlaces desde `README.md` y `docs/memory-model.md`; CI verde: https://github.com/lecodev-26/dinah/actions/runs/38097226307.
- 2026-10-11 — Punto 077 completado: contrato inicial de Compute y secuencia de implementación en `docs/compute.md`; CI verde: https://github.com/lecodev-26/dinah/actions/runs/38097443705.
- 2026-10-12 — Punto 085 iniciado: añadido benchmark comparativo AoS/SoA para recorrer un campo o ambos campos con datos equivalentes; los resultados reproducibles quedan pendientes de CI multiplataforma.
2026-10-12 — Punto 085: CI multiplataforma completada; resultados AoS/SoA archivados para cinco runners y documentados en docs/data-oriented-research.md. La señal es mixta y se mantiene en progreso hasta repetir mediciones y valorar costes de construcción/conversión.
- 2026-10-12 — Punto 085: segunda ejecución de benchmarks completada y archivada (run 38207649616; CI: https://github.com/lecodev-26/dinah/actions/runs/38207649616). Se compararon ambas ejecuciones en cinco runners; cambió la dirección en macOS ARM64 y se mantiene evidencia mixta. Documentación actualizada; el punto sigue en progreso por falta de evidencia estadística y medición de construcción/conversión/escrituras.

- 2026-10-12 — Punto 085: tercera ejecución completa de AoS/SoA archivada (run 38208067774; CI de benchmarks y archivado verdes). Se añadieron resultados de los cinco runners; se mantiene en progreso porque el microbenchmark no mide construcción/conversión ni escrituras de una carga real.

- 2026-10-12 — Punto 035: auditoría de Core documentada en `docs/core.md`; se verificaron contratos, errores, matriz completa de transiciones y CI verde. Se mantiene en progreso porque `HasCoreId` aún no tiene un consumidor real que permita validar si merece formar parte del contrato estable.

- 2026-10-12 — Punto 089 iniciado: añadidos tests de determinismo de Compute en la API pública con ejecuciones repetidas y distintos recuentos de workers; CI pendiente. El punto permanece en progreso porque el determinismo transversal de Runtime/Memory y otros subsistemas aún no está cubierto.

- 2026-10-12 — Punto 088 completado: benchmark release y CI multiplataforma verdes en cinco runners (run 38208860456; CI https://github.com/lecodev-26/dinah/actions/runs/38208860464; benchmarks https://github.com/lecodev-26/dinah/actions/runs/38208860456; auditoría documental https://github.com/lecodev-26/dinah/actions/runs/38208860468). Resultados archivados en `artifacts/benchmarks/f728dba482d00450bb06a7e7b5f994e3f82152dc/run-38208860456/`. En 262.144 elementos, 2 workers tardaron 4,0–10,3× y 4 workers 6,3–16,6× que `for_each`; el kernel ligero no compensa el coste de coordinación. Esto cierra la comparación, no demuestra que todo workload paralelo sea más lento.

- 2026-10-12 — Punto 089 ampliado: `tests/runtime_determinism.rs` repite 64 tareas con carga determinista variable, cinco límites de workers y errores intercalados; verifica que Runtime devuelve resultados/errores en orden FIFO y vacía la cola. Documentado en `docs/runtime.md`; CI pendiente. El punto sigue en progreso porque Memory/Core y otros contratos aún no están cubiertos.
- 2026-10-12 — Punto 089 ampliado a Memory: `tests/memory_determinism.rs` repite secuencias de inserción, rechazo por capacidad, pop, clear/reutilización y alineación SoA; documentación actualizada en `docs/memory.md`. CI pendiente.

- 2026-10-12 — Punto 089 ampliado a Core: pruebas de API pública repetidas para configuración/errores, Validate, IDs suministrados por el llamador y matriz completa de transiciones. Documentado en `docs/core.md`; validación CI pendiente.

- 2026-10-12 — Punto 089 ampliado en Compute secuencial: tests públicos repetidos de orden de visita/salida, lotes parciales, validación antes de mutar, primer error, entradas vacías y detención de pipeline tras fallo; documentación actualizada. CI pendiente.

- 2026-10-12 — Punto 089 ampliado en Runtime: pruebas repetidas de idempotencia de cancelación, errores estables de transición de estado, contadores independientes y orden/capacidad/drenaje/reutilización de TraceBuffer; documentación actualizada. CI pendiente.

- 2026-10-12 — Punto 089 ampliado en Memory: pruebas repetidas de errores de extensión de Buffer/SoaBuffer (longitud/capacidad sin mutación ni desalineación) y capacidad cero; actualizada la guía pública. Pendiente CI del commit.

- 2026-10-12 — Punto 089 ampliado en Runtime: repetidas actualizaciones concurrentes mediante `Shared<T>` y comprobaciones de totales de métricas/capacidad, drenaje y vaciado de trazas bajo productores concurrentes. El orden entre eventos de hilos distintos queda explícitamente fuera de la garantía. Pendiente CI.

- 2026-10-12 — Punto 089 ampliado en Runtime: se añaden pruebas repetidas de la API pública `TaskQueue` (FIFO, rechazo sin mutación, capacidad y reutilización) y `TaskFailure` (clasificación estable y mapeo del error); documentación actualizada. Pendiente CI.

- 2026-10-12 — Punto 089 completado: cobertura repetida de contratos públicos de Core, Compute, Runtime y Memory, incluyendo resultados paralelos en orden de entrada, errores deterministas, estructuras acotadas, estado compartido, métricas y trazas concurrentes sin afirmar orden entre eventos concurrentes. Validación local de formato/diff/checklist y CI + auditoría documental + archivo de artefactos verdes para `7171343c90304a500f8944e3b2871c59bb3479ce` (CI: https://github.com/lecodev-26/dinah/actions/runs/38210712872; auditoría: https://github.com/lecodev-26/dinah/actions/runs/38210712967; archivo: https://github.com/lecodev-26/dinah/actions/runs/38210745109).

- 2026-10-12 — Punto 090 completado: auditada la guía `docs/compute.md` contra la API implementada y enlazada desde `README.md`; incluye ejemplos ejecutables de operaciones públicas, contratos de lotes y paralelismo, reducción, pipeline, errores, ownership, pánicos, determinismo, rendimiento y límites de lo aún no implementado. La auditoría documental y CI de la guía pasaron en el commit `7171343c90304a500f8944e3b2871c59bb3479ce` (CI: https://github.com/lecodev-26/dinah/actions/runs/38210712872; auditoría: https://github.com/lecodev-26/dinah/actions/runs/38210712967).

- 2026-10-12 — Punto 093 completado: se define `dinah::ecs::Component` como marcador público con implementación blanket para todo tipo `'static`, sin imponer `Send + Sync` mientras `World` sea single-owner. Añadidos tests unitarios/de integración y checks de presencia en CI; checklist validado localmente, pendiente CI remoto.

- 2026-10-12 — Punto 094 completado: `World` administra liveness, creación/destrucción, reutilización de índices y rechazo de handles obsoletos; los slots no reutilizables se retiran al desbordar la generación. Tests unitarios/de integración, documentación y checks CI añadidos; pendiente validación CI remota.

- 2026-10-12 — Punto 095 completado como diseño: `docs/ecs.md` define el contrato de selección de queries, entidades vivas, resultados vacíos, préstamos, límites de mutabilidad, orden no garantizado, límites de rendimiento y casos de aceptación. No se declara implementada la API; almacenamiento e iteración siguen pendientes en 096–097. CI del punto 094 y auditoría documental previos verificados verdes.

- 2026-10-12 — Punto 096 completado: añadido `src/ecs/storage.rs` con `ComponentStorage<T>` interno, columnas `Vec<Entity>`/`Vec<T>` alineadas, lookup sparse por índice y validación de generación, reemplazo, `swap_remove` con reparación del índice movido y reserva fallible previa a mutaciones. Tests unitarios cubren alineación, reemplazo, borrado/movimiento, handles obsoletos y acceso mutable. La API sigue interna y no se declara integrado el storage en `World`; punto 097 mantiene la iteración pendiente.

- 2026-10-12 — Punto 097 completado: iteradores secuenciales `iter()`/`iter_mut()` sobre las columnas contiguas del storage, pruebas de alineación, vacío y mutación; corregida documentación obsoleta sobre `World` y aclarado que el orden físico no es estable. CI, auditoría documental y archivado del punto 096 verificados verdes antes de estos cambios; el nuevo commit requiere su propia validación CI.

- 2026-10-13 — Punto 098 completado: `World::run_system` integra explícitamente una operación de sistema con `runtime::Scheduler`; el adaptador conserva la ejecución síncrona actual y diferencia el resultado del scheduler del error del sistema. Tests unitarios y de integración pública añadidos; documentación ECS/Runtime actualizada. Checks locales de formato, diff y checklist pendientes de CI remota.


### Registro de progreso — 2026-10-13

- Punto 099 completado en código y documentación: puente secuencial interno entre `ComponentStorage` y `compute::for_each`, con tests de alineación de entidades, mutación de valores y storage vacío. El storage sigue sin estar conectado a `World`; no se habilita ejecución paralela.

- 2026-10-13 — Punto 100 iniciado: documentados los requisitos de seguridad para sistemas ECS paralelos, incluidos conflictos read/write y write/write, bounds Send/Sync por API concurrente, aislamiento de mutaciones estructurales y criterios de aceptación. No se marca completado: `World` sigue siendo single-owner, el storage no está integrado y aún no existe una API paralela ECS segura.

- 2026-10-14 — Punto 100 avanzado: añadidos `SystemAccess` y `AccessMode` públicos con detección de conflictos por `TypeId`, deduplicación de declaraciones y precedencia conservadora de escritura; tests unitarios y de contrato público. No ejecuta sistemas ni habilita workers; punto sigue en progreso.

- 2026-10-14 — Punto 100 avanzado: `SystemPlan` construye lotes contiguos deterministas a partir de `SystemAccess`, conserva el orden de sistemas conflictivos y acepta entrada vacía. Añadidos tests unitarios y públicos. El planificador solo organiza metadata y no ejecuta closures ni habilita paralelismo seguro; punto sigue en progreso.

- 2026-10-14 — Punto 101 completado en pruebas y documentación: se verifican invariantes de `World` y `ComponentStorage` tras secuencias repetidas de mutaciones, incluyendo lista libre sin duplicados, coherencia del contador de entidades vivas, slots retirados, alineación de columnas y reparación del índice sparse. El storage continúa sin estar integrado con `World`; no se afirma eliminación automática de componentes en `despawn`. Checks locales de formato, diff y checklist ejecutados; pendiente CI remoto.


### Registro de progreso — 2026-10-14

- Punto 103 iniciado: revisión de las estructuras actuales y de la evidencia disponible. `ComponentStorage<T>` ya usa columnas contiguas y un índice sparse validado por generación, pero aún no está integrado con `World`; los benchmarks existentes solo cubren ciclo de vida de entidades y layouts genéricos. Documentados los casos necesarios para medir accesos densos/dispersos, generaciones obsoletas, iteración de uno/dos componentes, mutaciones estructurales y coste de preparación por separado. No se introduce una optimización especulativa ni se marca completado sin resultados representativos.


- 2026-10-14 — Punto 103 avanzado: conectado el registro de storages tipados a `World` mediante `TypeId` y borrado de tipo seguro (`Any`), añadidos `insert/get/get_mut/remove/has`, rechazo de handles muertos/obsoletos y limpieza de componentes durante `despawn`. Se añaden pruebas unitarias y de contrato público, incluyendo componente `!Send`/`!Sync` y reutilización de índice. El punto sigue en progreso: faltan benchmarks reales de acceso y queries antes de optimizar la representación. Pendiente CI remoto.


- 2026-10-14 — Punto 104 completado: auditada y actualizada `docs/ecs.md` contra el código real tras la integración de storage con `World`. Se corrigieron afirmaciones obsoletas sobre almacenamiento aún no integrado y se añadieron API/errores actuales, ejemplo público compilable conceptualmente y límites claros de queries y paralelismo. CI remoto del cambio de implementación y auditoría documental en curso.

- 2026-10-14 — Punto 103 avanzado: tras validar el CI de `1df4753`, ampliado `benches/ecs.rs` con cargas públicas de `World::get` para índices densos/dispersos, rechazo de generaciones obsoletas y un ciclo de insert/replace/remove. `docs/ecs-benchmarks.md` distingue estas mediciones de las queries e iteración pública aún inexistentes. Pendiente CI/benchmarks multiplataforma para revisar resultados; el punto permanece en progreso y no se declara optimización demostrada.


- 2026-10-14 — Punto 105 completado como diseño: `docs/simulation.md` delimita Simulation frente a Core/ECS/Compute/Runtime, propone contratos conceptuales de tiempo/actualización, define límites de determinismo, errores, pánicos, integración y criterios de aceptación. No se declara implementado ningún módulo ni API; timestep y actualización determinista siguen pendientes en 111–112.

- 2026-10-14 — Punto 103: archivados y revisados los benchmarks de acceso ECS en cinco runners (run 38478287951). Se documentan medianas de get denso/disperso, miss de generación obsoleta y ciclo combinado insert/replace/remove; la variación por runner y la falta de iteración/query pública no justifican cambiar la representación. Punto 103 sigue en progreso.

- 2026-10-14 — Punto 111 completado: se añadió `dinah::simulation::{SimulationClock, SimulationTime, SimulationTimeError}`. El reloj no consulta wall-clock; rechaza delta cero y calcula tick/duración siguientes antes de mutar para preservar el estado si hay overflow. Tests específicos cubren progresión y errores. Los puntos 112+ permanecen pendientes.

- 2026-10-14 — Punto 112 completado: añadido `Simulation<S>` con callback síncrono y tiempo explícito. El reloj solo confirma el tick cuando la actualización termina correctamente; overflow temporal evita invocar el callback. Tests cubren orden de ticks, éxito, error y overflow. Se documenta que el estado de dominio no se revierte automáticamente si el callback muta y falla.

- 2026-10-14 — Punto 113 completado: añadida `SimulationEntities<T>` como colección de estado simulado independiente de ECS, con `SimulationEntityId` monotónico, inserción/acceso/mutación/eliminación, iteración en orden de inserción y error explícito de agotamiento sin mutación. Tests cubren orden, IDs no reutilizados, borrado y agotamiento; documentados los costes lineales y el límite frente a `ecs::Entity`. CI remoto correcto en `fdcfbb2`.

- 2026-10-14 — Punto 114 implementado: añadida `SimulationEntityBindings` como puente explícito entre `SimulationEntityId` y `ecs::Entity`. Solo enlaza handles vivos, impide asociar dos veces el mismo handle, revalida liveness/generación en `resolve` y permite retirar la asociación sin despawn implícito. Tests cubren enlaces válidos, duplicados, handles obsoletos y rechazo sin mutación. El puente no posee `World`; el llamador debe usar consistentemente el mundo propietario y gestionar `unbind`. Pendiente CI remoto.

- 2026-10-14 — Punto 114 validado por CI en `148dcbe`; punto 115 implementado: `SimulationEntities::for_each_mut` delega el recorrido mutable a `compute::for_each`, con orden de inserción determinista y sin workers implícitos. Test cubre orden de visita y mutación de cada estado. Pendiente CI remoto del nuevo commit.


- 2026-10-14 — Punto 115 validado por CI (`38de8bf`): DINAH CI y auditoría documental terminaron correctamente. Punto 116 implementado: test de replay ejecuta dos veces la misma secuencia de entradas explícitas sobre Simulation y SimulationEntities, y compara traza por tick, estado por ID y estado final. Sin dependencia de wall-clock ni aleatoriedad implícita. Checks locales de formato, diff y checklist; CI remoto pendiente.

- 2026-10-14 — Punto 116 validado por CI en `cf19fde` (DINAH CI, auditoría documental y archivo de artefactos correctos). El ajuste de alias de tipos resolvió `clippy::type_complexity`. PR #7 del bot documental revisado y cerrado sin merge: solo contenía una lista generada; la documentación de Simulation ya cubría el cambio y el arreglo era interno al test. Punto 117 iniciado: añadido `benches/simulation.rs` con casos de reloj, paso de simulación y recorrido/búsqueda de entidades. La primera ejecución CI pasó, pero la revisión de artefactos detectó que el runner no invocaba el nuevo binario; se corrige el script de benchmarks, el disparador del workflow y se limita la muestra de búsquedas lineales a 16 IDs para mantener un coste acotado. La nueva ejecución `38484605568` pasó en los cinco runners; el workflow de archivo `38484770738` archivó las salidas y entornos en `artifacts/benchmarks/83cbcc3f3a36bd38ee661ae12075a0e27b654414/run-38484605568/`. Punto 117 completado: `docs/simulation.md` registra medianas de referencia y aclara sus límites; no se infieren mejoras ni se cambia la representación basándose solo en estos microbenchmarks.

- 2026-10-14 — Punto 117 validado: DINAH CI, auditoría documental y benchmarks de `83cbcc3` pasaron. Se inspeccionaron los cinco artefactos de Simulation y se verificó su archivo en el commit automático `c860fc7`; se documentaron las medianas medidas y se marcó el punto 117 como completado. Checks locales de formato, diff y checklist pendientes tras esta actualización documental.

- 2026-10-14 — CI principal y auditoría documental pasaron para `41a2f6f`; el archivo automático `38485049766` también terminó correctamente. Punto 118 validado: el ejemplo ejecutable está incluido en CI y usa la API real. Punto 119 actualizado para alinear la guía con el estado implementado, eliminar referencias a benchmarks pendientes y describir explícitamente las limitaciones actuales. Checks locales de formato/diff/checklist ejecutados después del cambio documental.


### Auditoría de puntos no marcados — 2026-10-14

- Se verificó que la CI del commit `320c5d7` terminó correctamente (https://github.com/lecodev-26/dinah/actions/runs/38485213862) y que el archivado automático también terminó correctamente (https://github.com/lecodev-26/dinah/actions/runs/38485269268).
- Se revisó la lista completa de puntos pendientes/en progreso, el inventario de módulos, tests, benchmarks y workflows antes de iniciar Networking.
- Se cerraron 087 y 211–216 porque hay implementaciones de benchmarks CPU y suites unitarias por módulo con CI verde. Esto no cierra la cobertura global, pruebas de estrés, fuzzing ni la matriz de arquitecturas, que siguen siendo tareas distintas.
- Se cerró 223 por la cobertura repetible observable ya existente en Core, Runtime, Memory, Compute y Simulation; no se afirma determinismo temporal ni bit a bit entre arquitecturas.
- 220 y 225 pasan a en progreso, no a completados: ya existen pruebas de contrato/determinismo y benchmarks para los módulos implementados, pero falta la integración end-to-end completa y quedan subsistemas futuros sin pruebas/benchmarks.
- Se mantienen abiertos 033 (no hay carga representativa propia de Core), 035 (estabilidad de Core aún depende de justificar `HasCoreId`), 084 (sin kernel SIMD de producción que demuestre beneficio), 085 (AoS/SoA no justifica un cambio general), 086 (no existe todavía un sistema transversal de métricas de rendimiento), 100 (sin ejecución ECS paralela segura) y 103 (faltan iteración/query pública y benchmarks representativos para optimizar).
- No se inició Networking ni se modificó código de producto en esta auditoría. Los puntos de Networking, Storage, Native, CLI, observabilidad, seguridad, cobertura, profiling, integración total y release siguen pendientes hasta que haya evidencia específica.


### Implementación inicial de Simulation 106–110 — 2026-10-14

- Confirmada CI verde para la auditoría previa `e87549d`: CI https://github.com/lecodev-26/dinah/actions/runs/38485862573 y archivo de artefactos https://github.com/lecodev-26/dinah/actions/runs/38485917858.
- Añadidos cinco módulos sin dependencias externas: `simulation::ascii_physics`, `simulation::fluid`, `simulation::cellular`, `simulation::raycast` y `simulation::swarm`. Cada módulo contiene tests de dominio y valida entradas básicas.
- 106 es una física discreta de rejilla (no un motor rígido general); 107 es difusión escalar explícita (no CFD); 108 es Conway Life con buffer doble; 109 es raycasting 2D contra AABB; 110 es flocking 2D basado en snapshot. Las limitaciones están documentadas en `docs/simulation.md`.
- CI confirmada verde en `f7591b2`: tests, Clippy, documentación pública y checklist pasaron (https://github.com/lecodev-26/dinah/actions/runs/38486267990); auditoría documental pasó (https://github.com/lecodev-26/dinah/actions/runs/38486267997). Los cinco puntos se marcan completados con el alcance acotado descrito arriba.
- No se inició Networking ni la siguiente fase.


### Networking — diseño inicial (punto 120)

- Tras completar y validar Simulation 106–110, se inicia Fase 7 con el diseño de API. `docs/network.md` separa mensaje lógico, serialización, transporte, conexión e integración; fija contratos de seguridad y límites, y documenta decisiones aplazadas. No se implementaron sockets ni dependencias en este punto.
- La fase sigue abierta: los puntos 121–140 no se consideran iniciados ni completados por el documento de diseño.


### Identidad visual del README — 2026-10-14

- Actualizada la identidad visual de DINAH a una paleta naranja coherente (`#e87524`, `#ffb15c`, `#c65312`) en las marcas, botones, cabeceras, diagramas, gráficos y variantes light/dark del README. Se recolorearon los recursos raster de marca existentes y se corrigió el README para describir el estado real del repositorio: un único crate Rust, cinco suites de benchmarks y un ejemplo ejecutable. No se modificó código de producto ni se alteró el estado de los 287 puntos.


### Networking — contrato de transporte (punto 121)

- Se fija el primer contrato como byte stream bloqueante, separado del proveedor de sockets: lectura parcial permitida, `0` como EOF, escritura completa con errores parciales explícitos, timeout configurado sin reloj oculto, ownership prestado, límites en framing y ninguna reconexión/reintento automático. No se introduce UDP ni se afirma implementación de sockets.
- El punto 121 queda documentado y validado por CI en el commit `8be9e28` (ejecución 38491633487, todos los trabajos correctos).


### Networking — contrato de mensajes (punto 122)

- Se define el envelope lógico de mensaje: versión de protocolo, identificador de tipo estable, versión del mensaje y payload opaco. Se documentan rechazo de versiones/tipos desconocidos por defecto, compatibilidad explícita, límites y errores, separando el contrato semántico del formato binario/framing. El ejemplo Rust es conceptual; no se han implementado tipos, serialización ni sockets.
- El punto 122 queda documentado; el punto 123 (serialización) es el siguiente.


### Networking — investigación de client-side prediction (punto 127)

- 2026-10-18 — Investigación documentada en `docs/client-side-prediction.md`: servidor autoritativo, inputs con secuencia/tick, snapshot con última secuencia procesada, historial acotado, replay de inputs pendientes, separación entre estado simulado y suavizado visual, límites de determinismo, seguridad y memoria. Se documentan referencias técnicas y criterios incrementales para 128–131. No se afirma que prediction, reconciliation, history buffers ni resimulation estén implementadas. CI de este cambio pendiente.

- 2026-10-18 — Bloque 128–131 implementado en `src/network/prediction.rs`, exportado como `dinah::network::prediction`. Incluye secuencias monotónicas, historial acotado, aplicación local inmediata, reconciliación con snapshots autoritativos, replay de inputs no confirmados y manejo de snapshots obsoletos/ACK futuros. Tests de contrato añadidos en `tests/network_prediction.rs`; CI pendiente. La transición de estado la aporta el consumidor y debe ser reproducible; no se declara determinismo universal.

- 2026-10-18 — Puntos 132–139 completados en bloque: outbox fiable de nivel de aplicación, estimación RTT, política explícita de liveness/backoff, cola de trabajo Runtime, entrega por tick de Simulation, simulador determinista de pérdida/latencia, tests y benchmark de Networking. Los retries no sustituyen la fiabilidad TCP ni implican UDP; no hay reconexión automática. Formato, diff y checklist local validados; pendiente CI/auditoría/benchmarks del commit de este bloque.

- 2026-10-18 — Puntos 140–147 completados: documentación de Networking actualizada y primer módulo Storage con API de bytes, MemoryStore, journal local versionado y recuperación básica. Checksums/integridad, transacciones y durabilidad fuerte siguen pendientes.


### CI, seguridad y matriz Linux — 2026-10-18

- El commit `fcde532` pasó `dinah CI`, `dinah artifact archive` y `dinah quality and security`. El workflow de calidad confirmó `cargo audit`, el consumidor C compilado/enlazado/ejecutado y la generación/subida del artefacto LCOV. Esto permite cerrar 209, 226 y 271, y actualizar la evidencia de 161/198; no significa que la auditoría detecte o descarte todos los riesgos, ni fija un umbral de cobertura.
- Se añadió a `.github/workflows/ci.yml` una matriz Linux x86-64/ARM64 con formato, tests de librería e integración, Clippy y rustdoc. Los puntos 228–229 permanecen en progreso hasta que la nueva matriz pase CI; Windows/macOS no se presentan como cubiertos por esta matriz de tests.
- El punto 270 sigue en progreso porque los controles automatizados no sustituyen una auditoría externa. El punto 272 sigue en progreso hasta completar una revisión formal de licencias.

- 2026-10-18 — CI de `7a5e076` verificada: pasaron los cuatro jobs de `dinah CI`, incluidos los runners `ubuntu-latest` (x86-64) y `ubuntu-24.04-arm` (ARM64), además del workflow de archivo de artefactos. Se cierran los puntos 228–229; la matriz de pruebas explícita sigue limitada a Linux.


### Integración pública entre subsistemas — 2026-10-18

- Añadido `tests/system_pipeline.rs`: valida configuración Core, buffer acotado, transformación Compute programada por Runtime, persistencia del resultado como componente ECS, avance de Simulation con tiempo lógico fijo, despacho de StorageWorkQueue a MemoryStore y métricas opt-in exportadas a JSON.
- Cerrados los puntos 247–251, 253, 256 y 259 con evidencia en el test. El punto 220 permanece en progreso porque un smoke test no demuestra integración completa ni cubre Networking, Native o CLI.
- Validaciones locales permitidas: `cargo fmt --all -- --check` y `git diff --check`; la compilación y ejecución de tests queda a cargo de CI.


### Scheduler: medición antes/después y siguiente experimento — 2026-10-18

- CI del commit `2aeee49` verificada: CI principal, benchmarks, calidad/seguridad, auditoría documental y archivo de artefactos terminaron correctamente. Se descargaron los cinco artefactos de benchmarks y se compararon los casos no-op con el baseline `3a36b73`.
- La reutilización de la capacidad del batch no muestra una mejora consistente en los cinco runners. Se documentaron valores concretos y límites en `docs/performance-baseline.md`; se completa el punto 246, mientras 238–245 siguen abiertos por falta de evidencia representativa y de mediciones específicas.
- Nuevo experimento en `src/runtime/scheduler.rs`: sustituir el receptor de tareas compartido (`Arc<Mutex<Receiver<_>>>`) por un canal dedicado por worker scoped. Cada lote sigue retirándose por completo de la cola antes del envío, no asigna más de una tarea a cada worker por lote, recoge resultados por índice para conservar orden FIFO y envía shutdown a todos los workers. Se reutilizan los tests existentes de concurrencia, errores y pánicos; CI y benchmarks de este nuevo cambio quedan pendientes.


### Registro de progreso — 2026-10-18

- Puntos 204, 261, 262, 265, 267, 268 y 270 avanzados: el parser de `DINAH.toml` sigue siendo deliberadamente un subconjunto, pero ahora rechaza errores sintácticos comunes, tablas conocidas/claves duplicadas y cadenas básicas con escapes no admitidos, e informa la línea del error. Se añaden cuatro grupos de pruebas de regresión y se alinean `docs/cli.md`, `docs/security-review.md` y el resumen de estado de `README.md`. No se marca como parser TOML completo; la ejecución de tareas se habilita después con selección explícita en el bloque CLI siguiente, y no se cierra fuzzing coverage-guided, umbral de cobertura, congelación API/arquitectura ni revisión legal. Checks locales de formato, diff y checklist correctos; CI del commit nuevo pendiente.

- 2026-10-18 — Bloque CLI: `dinah run` conserva la validación por defecto y añade ejecución explícita con `--task NAME`; el comando declarado se ejecuta en el directorio del proyecto, JSON captura stdout/stderr, y el código de salida se propaga. Se amplían pruebas unitarias e integración, `docs/cli.md` y revisión de seguridad; se documenta claramente que no hay sandbox. CI de este bloque pendiente; los puntos 220, 257, 267 y 268 siguen en progreso.


### Ejemplos públicos — 2026-10-18

- Avanzado el punto 266 con ejemplos ejecutables para Compute, Buffer, ECS y Storage. `examples/storage_basics.rs` cubre MemoryStore, DiskStore, replay al reabrir y eliminación; `docs/storage.md` enlaza ahora el comando reproducible `cargo run --example storage_basics` y recuerda los límites de durabilidad. La CI del commit `54e9a9b` confirmó la compilación de ejemplos; este punto sigue en progreso hasta revisar cobertura de los subsistemas restantes. Checks locales permitidos ejecutados; no se ha ejecutado `cargo test` localmente.

- 2026-10-18 — Avanzado el punto 266 con `examples/network_basics.rs`: intercambio TCP de loopback usando Endpoint/Connection, lecturas parciales seguras, cierre de escritura y detección de EOF; `docs/network.md` incluye comando y límites explícitos. Se mantiene en progreso hasta validar este ejemplo por CI y revisar subsistemas restantes.

- 2026-10-18 — Revisión de orden: se aplazan 033, 035, 084, 085, 100, 103, 165 y 204 con su dependencia concreta descrita en la checklist; no se declaran completados ni se inventan benchmarks/consumidores. Se continúa por el siguiente trabajo que pueda avanzar con evidencia real.


- 2026-10-18 — Punto 220 ampliado: `tests/system_pipeline.rs` ahora valida el límite Runtime → Storage → DiskStore con comandos FIFO PUT/DELETE, confirma que encolar no muta el almacén y comprueba la recuperación del snapshot tras cerrar y reabrir el journal. `docs/integration-matrix.md` describe la evidencia y aclara que el replay no equivale a durabilidad ante pérdida de energía. Punto 220 sigue en progreso; CI de este cambio pendiente.

- 2026-10-18 — Punto 220 ampliado otra vez: se añadió un escenario transversal donde `NetworkSimulator` retiene una entrada hasta el tick lógico previsto, el consumidor la aplica explícitamente a `Simulation` y el estado final se escribe vía `StorageWorkQueue` en `DiskStore` y se recupera tras reabrir. La matriz de integración delimita que es transporte simulado y no protocolo de red real. Checks locales de formato/diff/checklist/auditoría documental ejecutados; CI pendiente.

- 2026-10-18 — Punto 220 ampliado con un pipeline TCP loopback real: `Endpoint` acepta una `Connection`, recibe un valor fijo de 8 bytes, el consumidor lo aplica explícitamente a `Simulation` en un tick lógico y persiste/recupera el estado mediante `StorageWorkQueue` y `DiskStore`. `docs/integration-matrix.md` delimita que no hay framing ni autenticación y que un valor fijo no representa un protocolo general. Checks locales permitidos y CI pendiente.


### Cierre de integración y umbral inicial de cobertura — 2026-10-18

- Tras verificar los cuatro workflows verdes de `9debb1b`, se revisaron `tests/system_pipeline.rs`, `tests/system_integration.rs` y `docs/integration-matrix.md`. La cobertura de integración incluye las APIs actualmente implementadas de Core, Runtime, Memory, Compute, ECS, Simulation, Network, Native, Storage, observabilidad y CLI; se cierra 220 con límites explícitos para framing/autenticación y sin declarar una aplicación de producción.
- Se descargó y revisó el artefacto LCOV de CI `dinah-lcov-9debb1b2ad13ddf00025e27d6210b789c238ce67`: 4.814 de 5.297 líneas cubiertas (90,88 %) y 670 de 763 funciones (87,81 %). El reporte no contiene contadores de ramas (`BRF:0`), por lo que no se hace ninguna afirmación de cobertura de ramas.
- Se fija un umbral inicial y explícito del 85 % de líneas con `cargo llvm-cov --fail-under-lines 85`. La distancia de ~5,9 puntos respecto a la medición actual permite cierta variación mientras impide regresiones grandes; se revisará al ampliar el conjunto de pruebas y antes de release. Este umbral agregado no garantiza cobertura de rutas críticas ni sustituye pruebas de seguridad específicas.

- 2026-10-18 — Punto 261 avanzado con inventario de API pública en `docs/public-api-review.md`. Documenta módulos y límites reales, identifica los contratos aún provisionales (HasCoreId, ECS queries/parallel systems, wire protocol, durabilidad y CLI sin sandbox) y enumera gates explícitos antes de congelar v0.1. Se corrigieron contadores y estado de integración/cobertura en README; el punto permanece en progreso hasta que se apruebe formalmente la superficie soportada.

- 2026-10-18 — Tras el CI verde de `d5b6ce6`, se actualiza el punto 266: Clippy `--all-targets` compila los targets de ejemplo, aunque la revisión funcional/documental de todos los ejemplos sigue abierta. Punto 268 registra el umbral de líneas del 85 % ya configurado y deja explícitas las brechas restantes de ramas, rutas críticas y fuzzing. Punto 272 distingue el smoke test ABI completado de la revisión legal aún pendiente.


### Repetibilidad de benchmarks Runtime — 2026-10-18

- El runner de benchmarks ejecutará la suite Runtime en tres procesos independientes por runner (configurable entre 1 y 5 con `DINAH_RUNTIME_BENCH_REPETITIONS`). Cada ejecución se conserva en un archivo separado y en `baseline.json`; el archivo y campo `cases` históricos siguen representando la primera ejecución para compatibilidad. Las otras seis suites se ejecutan una vez para no multiplicar innecesariamente el coste de CI.
- La repetición independiente ayuda a observar variación entre procesos en los casos del Scheduler, pero no es un estudio estadístico ni elimina efectos de orden, temperatura, caché o planificación del SO. No se ha declarado ninguna mejora de rendimiento; los puntos 238, 243 y 245 siguen abiertos hasta revisar los artefactos de CI y disponer de comparaciones controladas.


- 2026-10-18 — CI de `2d7e8b4` verificada: CI principal, Quality/Security, auditoría documental, benchmarks multiplataforma y archivo de artefactos terminaron correctamente. Se revisaron los cinco artefactos de benchmarks: cada uno contiene tres ejecuciones independientes de Runtime. Para los 256 no-op, el scheduler quedó entre 10,5× y 63,9× más lento que el prototipo de pool persistente, según runner y número de workers; el prototipo excluye el coste de crear workers por llamada y no equivale semánticamente al Scheduler. Se amplió `scripts/run_benchmarks.py` para resumir automáticamente mediana de medianas, rango y ratio por workload, y `docs/performance-baseline.md` registra los valores por runner y sus límites. Puntos 238, 242, 243 y 245 permanecen abiertos: la evidencia señala una hipótesis de coste de lifecycle en una carga sintética, no prueba un cuello de botella de producción ni justifica todavía cambiar la arquitectura pública.
