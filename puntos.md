# dinah — Checklist maestro

Este archivo es el registro operativo vivo. No se renumeran ni reutilizan IDs. Para completar un punto, debe existir evidencia suficiente en código, pruebas, documentación o resultados según corresponda.

## Estado inicial

- Total de puntos numerados: 287
- Completados: 90
- En progreso: 4
- Pendientes: 193

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
- [>] 033 — Crear primeros benchmarks
- [x] 034 — Revisar API pública
- [>] 035 — Estabilizar Core v0.1 (auditoría inicial de API/documentación/tests registrada en `docs/core.md`; siguen abiertos el primer consumidor real de `HasCoreId` y la decisión de estabilidad que depende de él)

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
- [>] 084 — Implementar SIMD donde aporte beneficio real (benchmark directo/API y candidato seguro `wide::f32x4`; matriz repetida: mejora en Linux x86-64, regresión en ARM64 y resultados mixtos en otros runners; sin kernel de producción que justifique SIMD; detalles en `docs/simd-research.md`)
- [>] 085 — Optimizar estructuras data-oriented (benchmark AoS/SoA repetido tres veces en cinco runners; SoA pierde de forma consistente al leer un campo en Windows y al leer ambos en Linux ARM64, pero el resto de diferencias son pequeñas o variables; falta una carga real y medir construcción/conversión/escrituras)
- [ ] 086 — Añadir métricas de rendimiento
- [ ] 087 — Crear benchmarks CPU
- [x] 088 — Comparar secuencial/paralelo (benchmark release archivado en cinco runners; salida escalar/paralela verificada; análisis documentado: en el kernel ligero, 2 workers tardan 4,0–10,3× más incluso con 262.144 elementos y 4 workers 6,3–16,6× más; no se justifica paralelizar automáticamente este kernel)
- [x] 089 — Testear determinismo (tests públicos repetidos en Core, Compute secuencial/paralelo/pipelines, Runtime —orden FIFO/resultados, cancelación, estados, métricas, trazas, `Shared<T>`, `TaskQueue` y `TaskFailure`— y Memory —secuencias de Buffer/SoaBuffer, errores sin mutación, alineación y capacidad cero—; CI y auditoría documental verdes para el commit `7171343`)
- [x] 090 — Documentar Compute (guía pública `docs/compute.md` enlazada desde README; ejemplos de uso y contratos de las APIs secuenciales, por lotes, paralelas, reducciones y pipeline; límites de ownership, errores, determinismo, pánicos y rendimiento documentados; CI/auditoría de la guía confirmadas en `7171343`)

## FASE 5 — Data / ECS

- [x] 091 — Diseñar modelo ECS (decisiones iniciales documentadas en `docs/ecs.md`: sparse sets tipados, Entity con generación, ownership de World, contratos de query, seguridad sin `unsafe`, invariantes y secuencia de integración; todavía no declara APIs implementadas)
- [x] 092 — Definir Entity (`dinah::ecs::Entity` con índice/generación privados, accesores públicos, identidad/hash sensible a generación y pruebas de tamaño/valores límite; la validación de handles vivos queda para World)
- [x] 093 — Definir Component (`dinah::ecs::Component` público con implementación blanket para todo tipo Rust `'static`; sin requisito prematuro `Send + Sync`, validado con tests unitarios y de integración)
- [x] 094 — Definir World (`World::spawn/despawn/contains/len/is_empty`, errores tipados, reutilización con generación incrementada y retirada segura ante overflow; tests unitarios y públicos)
- [x] 095 — Definir queries (contrato documentado: selección por componentes requeridos, semántica de resultados vacíos, préstamos/mutabilidad, orden no estable y casos de aceptación; implementación pendiente en 096–097)
- [x] 096 — Crear almacenamiento contiguo (`ComponentStorage<T>` interno con columnas contiguas alineadas, sparse index validado por generación, reemplazo, eliminación `swap_remove` y reservas previas para evitar desalineación por fallo de asignación; aún no integrado con `World`)
- [ ] 097 — Crear iteración eficiente
- [ ] 098 — Integrar ECS con Runtime
- [ ] 099 — Integrar ECS con Compute
- [ ] 100 — Crear sistemas paralelos
- [ ] 101 — Testear consistencia
- [ ] 102 — Benchmark ECS
- [ ] 103 — Optimizar acceso a memoria
- [ ] 104 — Documentar arquitectura ECS

## FASE 6 — Simulation

- [ ] 105 — Diseñar Simulation API
- [ ] 106 — Integrar conceptos de ascii_physics_engine
- [ ] 107 — Integrar conceptos de fluid simulation
- [ ] 108 — Integrar cellular simulation
- [ ] 109 — Integrar raycasting
- [ ] 110 — Integrar swarm algorithms
- [ ] 111 — Definir timestep
- [ ] 112 — Definir actualización determinista
- [ ] 113 — Crear sistema de entidades simuladas
- [ ] 114 — Integrar Simulation con ECS
- [ ] 115 — Integrar Simulation con Compute
- [ ] 116 — Crear pruebas deterministas
- [ ] 117 — Crear benchmarks
- [ ] 118 — Crear ejemplos
- [ ] 119 — Documentar Simulation

## FASE 7 — Networking

- [ ] 120 — Diseñar Network API
- [ ] 121 — Definir transporte
- [ ] 122 — Definir mensajes
- [ ] 123 — Definir serialización
- [ ] 124 — Crear conexiones
- [ ] 125 — Crear endpoints
- [ ] 126 — Crear gestión de estado
- [ ] 127 — Investigar client-side prediction
- [ ] 128 — Implementar prediction
- [ ] 129 — Implementar reconciliation
- [ ] 130 — Implementar history/state buffers
- [ ] 131 — Implementar deterministic resimulation
- [ ] 132 — Gestionar pérdida de paquetes
- [ ] 133 — Gestionar latencia
- [ ] 134 — Gestionar desconexiones
- [ ] 135 — Integrar Network con Runtime
- [ ] 136 — Integrar Network con Simulation
- [ ] 137 — Crear tests de red
- [ ] 138 — Crear simulador de latencia
- [ ] 139 — Crear benchmarks
- [ ] 140 — Documentar Network

## FASE 8 — Storage

- [ ] 141 — Diseñar Storage API
- [ ] 142 — Definir almacenamiento en memoria
- [ ] 143 — Definir almacenamiento en disco
- [ ] 144 — Crear formato interno
- [ ] 145 — Crear persistencia
- [ ] 146 — Crear journal
- [ ] 147 — Crear recuperación
- [ ] 148 — Crear checksums/integridad
- [ ] 149 — Diseñar índices
- [ ] 150 — Crear estructuras eficientes
- [ ] 151 — Integrar Storage con Runtime
- [ ] 152 — Testear corrupción
- [ ] 153 — Testear recuperación
- [ ] 154 — Testear concurrencia
- [ ] 155 — Crear benchmarks
- [ ] 156 — Documentar Storage

## FASE 9 — Native / Hardware

- [ ] 157 — Definir HAL
- [ ] 158 — Definir abstracciones de plataforma
- [ ] 159 — Crear capa POSIX/Linux
- [ ] 160 — Diseñar FFI
- [ ] 161 — Crear C ABI
- [ ] 162 — Investigar integración C++
- [ ] 163 — Investigar Assembly
- [ ] 164 — Identificar hotspots reales
- [ ] 165 — Optimizar hotspots
- [ ] 166 — Añadir SIMD nativo donde corresponda
- [ ] 167 — Añadir instrucciones específicas cuando aporten beneficio
- [ ] 168 — Testear distintas arquitecturas
- [ ] 169 — Documentar Native

## FASE 10 — CLI

- [ ] 170 — Crear comando dinah
- [ ] 171 — Crear dinah init
- [ ] 172 — Crear dinah run
- [ ] 173 — Crear dinah task
- [ ] 174 — Crear dinah inspect
- [ ] 175 — Crear dinah benchmark
- [ ] 176 — Crear dinah doctor
- [ ] 177 — Crear configuración CLI
- [ ] 178 — Crear output estructurado
- [ ] 179 — Crear logs
- [ ] 180 — Crear errores amigables
- [ ] 181 — Crear documentación CLI
- [ ] 182 — Testear comandos

## FASE 11 — Observabilidad

- [ ] 183 — Diseñar sistema de tracing
- [ ] 184 — Añadir métricas Runtime
- [ ] 185 — Añadir métricas Memory
- [ ] 186 — Añadir métricas Compute
- [ ] 187 — Añadir métricas Network
- [ ] 188 — Añadir métricas Storage
- [ ] 189 — Crear profiler hooks
- [ ] 190 — Crear diagnóstico
- [ ] 191 — Crear modo debug
- [ ] 192 — Crear modo release
- [ ] 193 — Crear exportación de métricas
- [ ] 194 — Crear benchmarks reproducibles

## FASE 12 — Seguridad y robustez

- [ ] 195 — Revisar superficie de ataque
- [ ] 196 — Revisar entradas externas
- [ ] 197 — Revisar serialización
- [ ] 198 — Revisar FFI
- [ ] 199 — Revisar memoria
- [ ] 200 — Revisar concurrencia
- [ ] 201 — Revisar permisos
- [ ] 202 — Revisar filesystem
- [ ] 203 — Revisar networking
- [ ] 204 — Añadir fuzzing
- [ ] 205 — Añadir tests de corrupción
- [ ] 206 — Añadir tests de estrés
- [ ] 207 — Añadir tests de concurrencia
- [ ] 208 — Añadir tests de determinismo
- [ ] 209 — Auditoría de dependencias
- [ ] 210 — Documentar modelo de seguridad

## FASE 13 — Testing

- [ ] 211 — Unit tests Core
- [ ] 212 — Unit tests Runtime
- [ ] 213 — Unit tests Memory
- [ ] 214 — Unit tests Compute
- [ ] 215 — Unit tests ECS
- [ ] 216 — Unit tests Simulation
- [ ] 217 — Unit tests Network
- [ ] 218 — Unit tests Storage
- [ ] 219 — Unit tests Native
- [ ] 220 — Integration tests
- [ ] 221 — Concurrency tests
- [ ] 222 — Stress tests
- [ ] 223 — Determinism tests
- [ ] 224 — Regression tests
- [ ] 225 — Benchmark suite
- [ ] 226 — CI completa
- [ ] 227 — Coverage
- [ ] 228 — Test matrix Linux
- [ ] 229 — Test matrix arquitecturas soportadas

## FASE 14 — Optimización

- [ ] 230 — Crear baseline
- [ ] 231 — Perfilar Runtime
- [ ] 232 — Perfilar Memory
- [ ] 233 — Perfilar Compute
- [ ] 234 — Perfilar ECS
- [ ] 235 — Perfilar Simulation
- [ ] 236 — Perfilar Network
- [ ] 237 — Perfilar Storage
- [ ] 238 — Identificar hotspots
- [ ] 239 — Reducir allocations innecesarias
- [ ] 240 — Reducir copias
- [ ] 241 — Optimizar cache locality
- [ ] 242 — Optimizar paralelismo
- [ ] 243 — Optimizar scheduling
- [ ] 244 — Optimizar SIMD
- [ ] 245 — Benchmark antes/después
- [ ] 246 — Documentar resultados

## FASE 15 — Integración total

- [ ] 247 — Integrar Core + Runtime
- [ ] 248 — Integrar Runtime + Memory
- [ ] 249 — Integrar Runtime + Compute
- [ ] 250 — Integrar Compute + ECS
- [ ] 251 — Integrar ECS + Simulation
- [ ] 252 — Integrar Runtime + Network
- [ ] 253 — Integrar Runtime + Storage
- [ ] 254 — Integrar Native
- [ ] 255 — Integrar CLI
- [ ] 256 — Integrar observabilidad
- [ ] 257 — Ejecutar sistema completo
- [ ] 258 — Crear ejemplos completos
- [ ] 259 — Crear tests end-to-end
- [ ] 260 — Resolver dependencias circulares
- [ ] 261 — Revisar API pública
- [ ] 262 — Revisar arquitectura completa

## FASE 16 — dinah v0.1

- [ ] 263 — Congelar arquitectura v0.1
- [ ] 264 — Congelar API v0.1
- [ ] 265 — Completar documentación
- [ ] 266 — Completar ejemplos
- [ ] 267 — Completar CLI
- [ ] 268 — Completar testing
- [ ] 269 — Completar benchmarks
- [ ] 270 — Auditoría de seguridad
- [ ] 271 — Auditoría de dependencias
- [ ] 272 — Revisar licencia
- [ ] 273 — Preparar release
- [ ] 274 — Crear changelog
- [ ] 275 — Crear documentación pública
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
