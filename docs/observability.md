# DINAH Observability — puntos 183–190

`dinah::observability` ofrece una primera capa opt-in y sin dependencias para registrar señales explícitas. No instala un collector global ni instrumenta automáticamente Runtime, Memory, Compute, Network o Storage.

## Tracing

`Observability` posee un `TraceBuffer` de capacidad fija y usa los eventos de ciclo de vida ya definidos por Runtime. `trace` devuelve un error si el buffer está lleno o bloqueado; no sobrescribe eventos antiguos. El llamador decide cuándo registrar cada evento. `trace_snapshot` devuelve los eventos en orden de registro.

## Métricas por subsistema

`MetricsRegistry` registra nombres de métricas por subsistema (`Runtime`, `Memory`, `Compute`, `Network`, `Storage`). `increment` mantiene contadores saturados para evitar overflow; `set_gauge` reemplaza un valor instantáneo. `snapshot` devuelve una vista ordenada y copiada. Las métricas reflejan únicamente llamadas explícitas de la aplicación: la biblioteca no infiere uso de memoria, bytes de red, duración de tareas ni operaciones de compute/storage por sí sola.

Los nombres vacíos y el poisoning del mutex producen errores. Los nombres son etiquetas locales, no un esquema de telemetría versionado. No se definen cardinalidad máxima de etiquetas, agregaciones, ventanas temporales ni exportadores externos todavía.

## Profiler hooks

`profile(name, operation)` mide el tiempo transcurrido de una operación síncrona y guarda una muestra mientras quede capacidad. Las muestras posteriores no desplazan las antiguas al llenarse el buffer. `profile_with_hook` entrega la muestra al `ProfilerHook` suministrado por el llamador. Si la operación entra en pánico, el pánico se propaga y no se genera una muestra final. Esto no es un profiler de CPU, asignaciones ni stack sampling.

## Diagnóstico

`diagnose` informa del número de métricas, número/capacidad de muestras de profiling, capacidad del buffer de tracing y cantidad de nombres de métricas por subsistema. No realiza comprobaciones del host, de permisos, de seguridad, de salud de conexiones ni de integridad de datos.

## Ejemplo

```rust
use dinah::observability::{Observability, Subsystem};
use dinah::runtime::tracing::TraceEvent;
use std::num::NonZeroUsize;

let telemetry = Observability::new(
    NonZeroUsize::new(128).expect("nonzero capacity"),
    NonZeroUsize::new(64).expect("nonzero capacity"),
);
telemetry.trace(TraceEvent::TaskSubmitted)?;
telemetry.metrics().increment(Subsystem::Runtime, "tasks.submitted", 1)?;
let report = telemetry.diagnose()?;
assert_eq!(report.metric_count, 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

La implementación es local, explícita y síncrona. No incluye exportación de métricas, modo debug/release específico ni benchmarks de observabilidad; esos puntos siguen pendientes.
