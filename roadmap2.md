# Roadmap v0.2 — Segundo ciclo

Fecha: 2026-10-19 · Estado: diseño aprobado para el ciclo v0.2 · Fuente: cierre del ciclo v0.1.0 (puntos 261–287 de `puntos.md`)

Este documento registra la evaluación pedida por los puntos 278–281, el diseño v0.2 (punto 282) y los criterios de salida del ciclo. La checklist operativa vive en `puntos2.md` (puntos 288–326). El contrato v0.1 congelado está en `docs/public-api-review.md` y `docs/integration-matrix.md`.

## 1. Evaluación de la arquitectura real

Límites estructurales confirmados en el freeze v0.1 que v0.2 debe atacar en orden de prioridad:

- **Red sin contrato de transporte.** TCP fiable existe, pero no hay framing, handshake ni codec de eventos; `simulator.rs` es un simulador, no un transporte real. Es la mayor brecha para cualquier uso real.
- **ECS sin superficie pública de ejecución.** El planificador determinista y los metadatos existen, pero no hay queries de componentes ni sistemas paralelos; `app_*_example` solo prueba acciones simples.
- **Ciclo de vida caller-owned en todas las fronteras.** Coherente y deliberado, pero genera código repetitivo; v0.2 puede ofrecer helpers opcionales sin romper el contrato.
- **Journal de Storage sin transacciones atómicas** ni control de concurrencia; el flujo bidireccional está probado, la durabilidad fuerte no está prometida.
- **CLI con parser starter, no TOML completo**; sin sandbox para la ejecución opt-in de tareas.

## 2. Evaluación del rendimiento

Evidencia disponible al cerrar v0.1:

- Cinco plataformas con benchmarks archivados en CI (Runtime, Memory, Compute, ECS, Simulation, Networking, Storage) y ejecución local completa en el commit de release.
- Scheduler determinista en paridad con el pool de workers (0,922×–1,073× lifecycle no-op; 0,898×–1,112× CPU-heavy steady-state) con cargas sintéticas.
- Observaciones documentadas en `docs/performance-baseline.md`: el arranque/despacho por llamada es más caro que el pool persistente para tareas no-op; los cambios de reutilización del vector de lote y de canales por worker se midieron con resultados mixtos por runner (sin mejora consistente demostrable).
- Cinco plataformas con medianas archivadas para reloj/paso de simulación, recorrido mutable y búsquedas lineales (tabla del README).
- Lo que NO existe: profiling con workload representativo, medición de allocations/copias en hotpaths, cache misses, ni comparación emparejada de un cambio de producción. Los puntos 033/084/085/100/103/165/204 quedaron APLAZADO precisamente por esto y se retoman aquí como puntos 312–321.

## 3. Limitaciones v0.1 heredadas

- Parser de manifiesto: subconjunto starter (sin TOML completo).
- Ejecución de tareas CLI explícita y sin sandbox.
- Networking sin framing/auth/codec — solo TCP fiable y simulador.
- Observabilidad opt-in (métricas/tracing en el runtime, no global).
- Cobertura de líneas ≥85 % en CI; sin cobertura de ramas ni fuzzing coverage-guided.
- MSRV Rust 1.95 (exigida por `AtomicU64::try_update` y `wide` en benchmarks).

## 4. Necesidades reales

No hay consumidores externos del crate todavía (registrado en el punto 281). Las necesidades de v0.2 se derivan de las brechas entre el contrato v0.1 y los casos de uso objetivo del dominio (runtime de simulación/games deterministas):

1. Enviar y recibir eventos estructurados por TCP de forma delimitada y versionada.
2. Consultar entidades por componentes desde la API pública.
3. Ejecutar más de un sistema ECS por tick con aislamiento razonable.
4. Configurar el CLI con TOML estándar.
5. Poder medir mejoras de rendimiento reales (workload de referencia incluido).

## 5. Diseño v0.2

### Metas (mapeadas a `puntos2.md`)

| Meta | Fase | Puntos |
|---|---|---|
| Wire framing + handshake + codec de eventos versionado | 18 | 288–292 |
| Queries ECS públicas y sistemas declarativos | 19 | 293–297 |
| TOML completo, sandbox opcional, journal transaccional | 20 | 298–306 |
| Branch coverage, fuzzing, auditoría externa | 21 | 307–311 |
| Workload representativo y optimizaciones con evidencia | 22 | 312–321 |
| Release v0.2.0 con el mismo estándar de evidencia | 23 | 322–326 |

### No-metas de v0.2

- No cambiar el modelo caller-owned de ciclo de vida (solo helpers opcionales).
- No prometer durabilidad fuerte del storage más allá de transacciones atómicas.
- No implementar auth/criptografía propia (si se cifra, será con un crate auditado).
- No perseguir 1.0.0 ni congelar APIs más allá de semver 0.x.
- No añadir dependencias de runtime salvo justificación medida y documentada.

## 6. Criterios de salida del ciclo v0.2

1. Suite completa verde (tests + doc-tests) y CI verde en el commit de release.
2. Cada optimización medida con ejecución emparejada antes/después (punto 316).
3. Benchmarks ejecutados en las cinco plataformas y archivados.
4. Auditoría de seguridad, dependencias y documentación repetida sobre el commit de release (punto 286 de `puntos.md`).
5. Changelog v0.2.0 fechado, README actualizado y `cargo publish --dry-run` verde.
6. Tag `v0.2.0` + GitHub Release con notas basadas en evidencia.

La publicación efectiva de cualquier versión en crates.io sigue siendo decisión explícita del propietario del repositorio.
