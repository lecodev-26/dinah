# dinah — Roadmap maestro

## 1. Identidad y propósito

dinah es un proyecto independiente, con Rust como lenguaje principal, orientado a construir una base modular de ejecución y procesamiento de alto rendimiento. El objetivo es combinar un runtime, gestión de memoria, compute, estructuras data-oriented, simulación y, cuando exista una necesidad demostrada, capacidades de red y persistencia.

dinah no se define por acumular funciones: cada módulo debe tener límites claros, pruebas, documentación y resultados medibles.

## 2. Principios no negociables

- **Independencia:** dinah es un proyecto propio. AXIOMSYSTEMS, AXIOMSYSTEMS2 y AXIOM-NEXUS-Q quedan fuera del alcance y no se incorporan como dependencias ni como subproyectos.
- **Rust primero:** C/C++ y Assembly se evalúan caso por caso; nunca se añaden por espectáculo.
- **Medir antes de optimizar:** cada optimización relevante debe incluir una comparación reproducible.
- **Corrección antes que velocidad:** seguridad de memoria, concurrencia, determinismo y APIs comprensibles.
- **Dependencias justificadas:** limitar superficie de ataque, mantenimiento y complejidad.
- **CI remota:** compilaciones, pruebas, análisis estático y benchmarks se ejecutan en GitHub Actions. Termux se reserva para edición, Git y control del proyecto.
- **Checklist auditable:** los IDs de puntos.md son permanentes. No se renumeran ni se reutilizan.
- **Licencia dual:** el código propio se distribuye bajo MIT o Apache-2.0, a elección del usuario, salvo avisos específicos. Toda dependencia o contribución externa debe conservar sus avisos y cumplir sus propias licencias.
- **Identidad visual:** usar siempre `dinah` en minúsculas y una identidad naranja coherente en el README y los recursos gráficos. Paleta base: naranja `#e87524`, naranja claro `#ffb15c` y naranja oscuro `#c65312`; conservar variantes legibles para fondos claros y oscuros.
- **Evolución basada en evidencia:** las versiones posteriores se deciden con datos y necesidades reales, no por ampliar una lista indefinidamente.

## 3. Qué es y qué no es dinah

dinah aspira a ser una biblioteca/runtime modular reutilizable para tareas concurrentes, procesamiento de datos y simulación. No es, por definición, un motor de videojuegos, un sistema operativo, un sustituto universal de todas las bibliotecas ni una promesa de rendimiento sin benchmarks.

Las ideas externas se estudian por separado y solo se integran cuando la licencia lo permite, se entiende su procedencia y existe una justificación técnica.

## 4. Arquitectura objetivo

La arquitectura prevista se divide en capas que pueden evolucionar de forma independiente:

1. **Core:** tipos, errores, configuración, IDs, estados, traits e invariantes.
2. **Runtime:** tareas, workers, scheduler, executor, cancelación, sincronización y tracing.
3. **Memory:** buffers, ownership, allocators, pools y layouts contiguos.
4. **Compute:** procesamiento por lotes, iteración paralela, reducciones y pipelines.
5. **Data / ECS:** entidades, componentes, mundos, queries y almacenamiento orientado a datos.
6. **Simulation:** timestep, actualización determinista, entidades y algoritmos de simulación.
7. **Networking:** transporte, mensajes, serialización y herramientas para latencia y sincronización.
8. **Storage:** persistencia, journal, recuperación, integridad e índices.
9. **Native / Hardware:** abstracciones de plataforma, POSIX/Linux, FFI y optimizaciones justificadas.
10. **CLI:** interfaz para crear, ejecutar, inspeccionar y diagnosticar proyectos dinah.
11. **Observabilidad:** métricas, tracing, profiler hooks y diagnósticos.
12. **Seguridad y calidad transversal:** testing, fuzzing, auditorías, CI y benchmarks reproducibles.

Las capas son objetivos de diseño, no módulos ya implementados. Se evitarán dependencias circulares y APIs públicas prematuras.

## 5. Estrategia de desarrollo

Cada cambio sigue el ciclo:

**Diseñar → implementar → probar en CI → medir cuando corresponda → documentar → revisar → actualizar puntos.md.**

No se marca un punto como completado solo porque exista un archivo o una implementación parcial. Un punto se completa cuando satisface su definición y tiene evidencia suficiente. Los benchmarks no bloquean cada cambio ordinario, pero sí deben acompañar optimizaciones y decisiones de rendimiento.

## 6. Entorno y CI

- GitHub es el repositorio de referencia.
- El repositorio debe ser privado mientras el proyecto esté en fase de desarrollo cerrado.
- GitHub Actions ejecuta formato, compilación, tests y linting en runners remotos.
- Las tareas de benchmark y estrés se separan del CI rápido cuando su coste lo justifique.
- No ejecutar compilaciones, suites de tests ni benchmarks pesados en Termux.
- Los workflows deben tener permisos mínimos, límites de tiempo y versiones de acciones controladas.
- Los resultados de CI son la evidencia de validación; no se afirma que una prueba haya pasado hasta comprobar su ejecución.

## 7. Fases

### Fase 0 — Fundación y reglas
Definir identidad, propósito, alcance, exclusiones, políticas de lenguaje, dependencias, pruebas, benchmarks, documentación, licencia dual MIT/Apache-2.0 e identidad visual básica. Crear README, roadmap y checklist.

### Fase 1 — Core
Construir los tipos, errores, configuración, IDs, estados, abstracciones, traits e invariantes esenciales. Estabilizar la primera API interna y acompañarla con tests y documentación. El Core no se declara estable solo porque CI esté verde: deben cumplirse los criterios de salida de `docs/core.md`. El punto de benchmarks permanece abierto hasta que exista una operación representativa; no se publican cifras sintéticas de rendimiento.

### Fase 2 — Runtime
Diseñar tareas y workers; crear executor, scheduler y colas; añadir ejecución secuencial y paralela, sincronización, cancelación, errores, métricas y tracing. Probar carreras y comportamiento del scheduler.

### Fase 3 — Memory
Definir ownership, buffers, abstracciones de asignación, pools, alineación y estructuras contiguas. Medir asignaciones y seguridad bajo concurrencia.

### Fase 4 — Compute
Crear primitivas, lotes, iteración y reducción paralelas, pipelines y optimizaciones SIMD justificadas por mediciones.

### Fase 5 — Data / ECS
Definir Entity, Component, World, queries y almacenamiento eficiente; integrar ECS con Runtime y Compute y medir la localidad de memoria.

### Fase 6 — Simulation
Diseñar timestep y actualización determinista; investigar e integrar, si procede, simulación física, fluidos, autómatas celulares, raycasting y algoritmos de enjambre. Revisar licencias y procedencia de código externo.

### Fase 7 — Networking
Definir transporte, mensajes, serialización, conexiones y endpoints. Investigar predicción, reconciliación y resimulación determinista; probar pérdida, latencia y desconexiones.

### Fase 8 — Storage
Definir APIs de almacenamiento en memoria y disco, formato, persistencia, journal, recuperación, integridad e índices; probar corrupción y concurrencia.

### Fase 9 — Native / Hardware
**Estado parcial (puntos 157–169):** HAL y abstracciones de plataforma documentadas; proveedor host y adaptador Unix seguro; ABI C escalar versionada y header público; auditoría de hotspots y matriz de benchmarks documentadas. El punto 165 (optimizar hotspots) sigue pendiente por falta de una mejora repetible demostrada. La ABI aún necesita una prueba real de compilación/enlace en CI. No existe acceso directo al hardware ni assembly optimizado.

### Fase 10 — CLI
Estado actual (puntos 170–182): existe el binario dinah con init, validación run, listado de declaraciones task, inspect, catálogo benchmark y doctor básico. Hay opciones de salida text/json, logging verbose, mensajes de error y pruebas unitarias. Las tareas y benchmarks solo se enumeran: no se ejecutan desde el CLI. El parser de DINAH.toml es deliberadamente parcial, no un parser TOML completo.

### Fase 11 — Observabilidad
**Observabilidad (puntos 183–194):** módulo público `dinah::observability` con tracing opt-in acotado, registro explícito de contadores y gauges para Runtime/Memory/Compute/Network/Storage, muestras de profiling bajo demanda, diagnóstico y exportación JSON ordenada de snapshots. Los perfiles debug/release son los perfiles Cargo documentados y el runner de benchmarks registra el entorno para comparar ejecuciones. No instrumenta subsistemas automáticamente, no envía telemetría a servicios externos y no constituye profiling de CPU ni garantía de rendimiento.

### Fase 12 — Seguridad y robustez

**Revisión de seguridad actualizada (puntos 195–203, 205–208, 210, 217–219, 221–224):** el documento `docs/security-review.md` cubre memoria, concurrencia, permisos, filesystem, networking, entradas, serialización, FFI, integridad del journal y el modelo de confianza. Hay pruebas adicionales para longitudes/campos corruptos, transiciones de conexión, replay determinista, ABI escalar y concurrencia/estrés acotados sobre journals independientes. Sigue pendiente el punto 204 (infraestructura de fuzzing), 207–209 (concurrencia/determinismo y auditoría formal de dependencias), y las pruebas de estrés prolongadas y el smoke test C/link. La revisión es estática y no certifica ausencia de vulnerabilidades.

### Fase 13 — Testing
Asegurar tests unitarios por módulo, integración, concurrencia, estrés, determinismo, regresión, cobertura y matriz de plataformas soportadas.

### Fase 14 — Optimización
Establecer baselines, perfilar cada subsistema, identificar hotspots, reducir asignaciones y copias, mejorar localidad y scheduling, y documentar comparaciones antes/después.

### Fase 15 — Integración total
Integrar capas, resolver dependencias circulares, revisar APIs y ejecutar pruebas end-to-end con ejemplos completos.

### Fase 16 — dinah v0.1
Congelar alcance y API de la primera versión coherente; completar docs, ejemplos, CLI, pruebas, benchmarks, auditorías, changelog y publicación.

### Fase 17 — dinah v0.2+
Evaluar la arquitectura y las limitaciones reales, recoger necesidades, priorizar cambios y repetir el ciclo de testing, benchmarking y auditoría. Las funciones futuras se deciden por evidencia.

## 8. Criterio de salida de v0.1

La versión 0.1 debe ser coherente, documentada, reproducible y utilizable en el alcance realmente implementado. No se declarará terminada por haber llegado a un número de fase: deben pasar CI, pruebas, revisión de seguridad y validación de la API.

## 9. Registro operativo

puntos.md es la fuente de verdad para el estado de los trabajos. Los números no se renumeran. Estados permitidos:

- [ ] pendiente
- [>] en progreso
- [x] completado con evidencia
- [~] obsoleto o sustituido, explicando el motivo

Los nuevos puntos empiezan después del ID más alto existente. Las fases posteriores pueden ajustarse, pero se conserva el historial.
