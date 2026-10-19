# dinah — Checklist ciclo v0.2

Continuación de `puntos.md` (puntos 001–287, ciclo v0.1 cerrado en v0.1.0). Mismas reglas: no se renumeran ni reutilizan IDs, y para completar un punto debe existir evidencia suficiente en código, pruebas, documentación o resultados. El diseño de este ciclo está en `roadmap2.md`.

## Estado inicial (2026-10-19)

- Total de puntos numerados: 39
- Completados: 0
- En progreso: 0
- Pendientes: 39
- Obsoletos: 0

Estados: [ ] pendiente · [>] en progreso · [x] completado · [~] obsoleto/sustituido (explicar motivo).

## FASE 18 — Networking real (v0.2)

- [ ] 288 — Diseñar e implementar wire framing (delimitación de frames sobre TCP con tests de casos truncados/parciales)
- [ ] 289 — Implementar handshake y ciclo de vida de conexión completo (con estados, timeouts y rechazo de versión incompatible)
- [ ] 290 — Implementar codec de eventos versionado (wire schema v1 con compatibilidad hacia atrás documentada)
- [ ] 291 — Implementar control de congestión/pacing y compararlo con el simulador actual
- [ ] 292 — Exponer métricas de paquete/pérdida en `networking_metrics` con benchmark dedicado

## FASE 19 — ECS público (v0.2)

- [ ] 293 — Diseñar API pública de queries (iteración sobre entidades con componentes, sin exponer mutación descontrolada)
- [ ] 294 — Implementar queries con filtro por componente y pruebas de contrato
- [ ] 295 — Implementar validación declarativa de dependencias entre sistemas
- [ ] 296 — Implementar aislamiento de cambios estructurales durante la ejecución de sistemas
- [ ] 297 — Documentar el modelo de concurrencia ECS y sus garantías deterministas

## FASE 20 — Runtime, CLI y Storage (v0.2)

- [ ] 298 — Reemplazar el parser starter por TOML completo (crate auditado o parser propio con fuzzing; decisión documentada)
- [ ] 299 — Añadir sandbox opcional para la ejecución de tareas del CLI (documentando límites del sandbox elegido)
- [ ] 300 — Añadir cobertura de ramas al umbral de CI junto al 85 % de líneas existente
- [ ] 301 — Implementar transacciones atómicas en el journal de Storage con pruebas de crash simulado
- [ ] 302 — Implementar sistemas paralelos del planificador ECS sobre la API de queries (retoma APLAZADO 100 de `puntos.md`)
- [ ] 303 — Optimizar acceso a memoria tras implementar queries, con resultados multiplataforma (retoma APLAZADO 103)
- [ ] 304 — Activar observabilidad por defecto en modo debug sin coste en release (medido)
- [ ] 305 — Crear ejemplo de aplicación completa end-to-end (CLI + runtime + red + storage) como workload de referencia
- [ ] 306 — Escribir guía de migración v0.1 → v0.2 con cambios breaking

## FASE 21 — Calidad y seguridad (v0.2)

- [ ] 307 — Ampliar la revisión del conjunto de pruebas (incluir tests de propiedad donde aporte)
- [ ] 308 — Revisión completa de contratos de guías frente a la API (mantenimiento continuo de docs)
- [ ] 309 — Contratar/realizar auditoría externa de seguridad y publicar el informe con alcance y límites
- [ ] 310 — Añadir fuzzing coverage-guided del parser de manifiesto con libFuzzer (retoma APLAZADO 204)
- [ ] 311 — Ampliar el corpus adversario del CLI con reporte reproducible

## FASE 22 — Rendimiento con evidencia (v0.2)

- [ ] 312 — Definir el workload representativo (aplicación ejemplo del punto 305) como base de comparación
- [ ] 313 — Crear benchmarks del Core con carga real medible (retoma APLAZADO 033)
- [ ] 314 — Estabilizar Core v0.1 con un consumidor real que justifique `HasCoreId` (retoma APLAZADO 035)
- [ ] 315 — Añadir perfil CPU en CI (flamegraph/síntesis en runners) con artefactos archivados
- [ ] 316 — Medir cada optimización con ejecución emparejada antes/después y comparación controlada
- [ ] 317 — Reevaluar activación de SIMD con evidencia por runner y semántica numérica (retoma APLAZADO 084)
- [ ] 318 — Optimizar estructuras data-oriented midiendo construcción, conversión y escrituras (retoma APLAZADO 085)
- [ ] 319 — Optimizar hotspots identificados por perfiles reales, no cargas sintéticas (retoma APLAZADO 165)
- [ ] 320 — Profiling con workload representativo y comparación de un cambio de producción (cierra el ciclo 238–245 de `puntos.md`)
- [ ] 321 — Medir allocations/copias en hotpaths y registrar resultados

## FASE 23 — Release v0.2.0

- [ ] 322 — Congelar arquitectura y API v0.2 (actualizar `docs/public-api-review.md` e `docs/integration-matrix.md`)
- [ ] 323 — Actualizar changelog con sección v0.2.0 fechada
- [ ] 324 — Verificación local completa (fmt, clippy, tests, doc-tests, benchmarks, `cargo package`, `cargo publish --dry-run`) sobre el commit de release
- [ ] 325 — Publicar v0.2.0 en crates.io (decisión explícita del propietario) o documentar por qué no
- [ ] 326 — Crear tag `v0.2.0` y GitHub Release con notas basadas en evidencia

## Registro de cambios de estado

Añadir aquí la fecha, el ID y la evidencia cuando un punto cambie de estado. No borrar ni reutilizar IDs.

- 2026-10-19 — Checklist creada al cerrar el ciclo v0.1.0 (puntos 278–287 de `puntos.md`); los APLAZADO 033/035/084/085/100/103/165/204 se retoman como puntos 313/314/317/318/302/303/319/310 respectivamente.
