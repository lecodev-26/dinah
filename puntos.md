# dinah — Checklist maestro

Este archivo es el registro operativo vivo. No se renumeran ni reutilizan IDs. Para completar un punto, debe existir evidencia suficiente en código, pruebas, documentación o resultados según corresponda.

## Estado inicial

- Total de puntos numerados: 287
- Completados: 78
- En progreso: 2
- Pendientes: 207

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
- [>] 035 — Estabilizar Core v0.1

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
- [ ] 081 — Crear reducción paralela
- [ ] 082 — Crear pipelines
- [ ] 083 — Investigar SIMD
- [ ] 084 — Implementar SIMD donde aporte beneficio real
- [ ] 085 — Optimizar estructuras data-oriented
- [ ] 086 — Añadir métricas de rendimiento
- [ ] 087 — Crear benchmarks CPU
- [ ] 088 — Comparar secuencial/paralelo
- [ ] 089 — Testear determinismo
- [ ] 090 — Documentar Compute

## FASE 5 — Data / ECS

- [ ] 091 — Diseñar modelo ECS
- [ ] 092 — Definir Entity
- [ ] 093 — Definir Component
- [ ] 094 — Definir World
- [ ] 095 — Definir queries
- [ ] 096 — Crear almacenamiento contiguo
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
