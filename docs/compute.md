# dinah Compute — API secuencial y diseño evolutivo

Estado: las primitivas secuenciales, los lotes, la iteración y reducción paralelas, y la primera API de pipelines están implementados y documentados. Las optimizaciones y etapas posteriores siguen siendo trabajo futuro; no se deben interpretar como APIs disponibles.

## Objetivo y límites

Compute ofrecerá operaciones de procesamiento de datos que puedan ejecutarse primero de forma secuencial y, cuando haya una necesidad medida, de forma paralela. Debe reutilizar los contratos de Core y las primitivas de Runtime y Memory sin fusionar sus responsabilidades.

- **Core** valida tipos y contratos compartidos; no ejecuta kernels.
- **Memory** posee almacenamiento contiguo y fija sus límites de capacidad; no decide cómo procesar los datos.
- **Runtime** ejecuta y coordina trabajo; Compute define la semántica de las operaciones de datos.
- **Compute** ofrece operaciones puras o explícitamente mutables sobre datos prestados, con límites claros de errores, orden y concurrencia.

La primera versión será pequeña, genérica y sin dependencias externas. No habrá DSL, grafo de tareas, compilador JIT, allocator propio ni garantías de tiempo real.

## Diseño de API por etapas

### Etapa A — primitivas secuenciales

La API inicial usa funciones genéricas sobre slices prestados, sin estado global. `for_each` y `try_reduce` no asignan memoria propia; `map` asigna explícitamente el `Vec` de salida.

```rust
pub fn for_each<T, F>(items: &mut [T], operation: F)
where
    F: FnMut(&mut T);

pub fn map<T, U, F>(items: &[T], operation: F) -> Vec<U>
where
    F: FnMut(&T) -> U;

pub fn try_reduce<T, A, E, F>(
    items: &[T],
    initial: A,
    operation: F,
) -> Result<A, E>
where
    F: FnMut(A, &T) -> Result<A, E>;
```

Estas funciones forman parte de la API pública `dinah::compute`. Las tres operaciones son secuenciales y conservan el orden de entrada. `try_reduce` propaga el primer error del usuario y se detiene; `for_each` no ofrece rollback si la closure entra en pánico. Una futura variante de `map` con buffer del llamador se evaluará solo si hace falta.

### Etapa B — lotes explícitos

La API secuencial de lotes está implementada como `for_each_batch`. Recibe un tamaño positivo y entrega a la closure slices mutables prestados de la entrada original. No crea buffers temporales ni tareas por elemento. El último lote puede ser más pequeño que el tamaño solicitado.

```rust
pub enum BatchError {
    ZeroSize,
}

pub fn for_each_batch<T, F>(
    items: &mut [T],
    batch_size: usize,
    operation: F,
) -> Result<(), BatchError>
where
    F: FnMut(&mut [T]);
```

Un tamaño cero devuelve `BatchError::ZeroSize` antes de invocar la closure o mutar elementos. Con una entrada vacía y un tamaño válido no se invoca la closure. Los lotes y elementos conservan el orden original. No se promete rollback si la closure entra en pánico; el pánico se propaga.

### Etapa C — iteración y reducción paralelas

La iteración paralela está implementada mediante `for_each_parallel` y utiliza explícitamente `Runtime::Scheduler::run_parallel`; no mantiene un pool oculto ni crea un sistema de ejecución independiente. Las particiones son slices mutables disjuntos y se entregan como tareas a la cola de Runtime. Los workers están acotados por `worker_count` y por el número de elementos; son scoped y se unen antes de que la llamada termine.

```rust
pub enum ParallelError {
    ZeroWorkers,
}

pub fn for_each_parallel<T, F>(
    items: &mut [T],
    worker_count: usize,
    operation: F,
) -> Result<(), ParallelError>
where
    T: Send,
    F: Fn(&mut T) + Sync;
```

Un `worker_count` de cero devuelve `ParallelError::ZeroWorkers` antes de mutar la entrada. Una entrada vacía con un valor positivo no crea workers. Cada elemento se procesa exactamente una vez en una ejecución normal, pero el orden de finalización no está definido. `T: Send` permite mover particiones a workers y `F: Sync` permite compartir la operación de forma segura. Los pánicos se propagan conforme al contrato de Runtime tras unirse las tareas del lote actual; las mutaciones previas no se revierten. No se garantiza aceleración para cargas pequeñas.

La reducción paralela está implementada como `try_reduce_parallel`. Cada partición empieza con una copia de `identity`; sus acumuladores parciales se combinan secuencialmente en el orden de entrada de las particiones. Para resultados independientes de la partición, `identity`, `operation` y `combine` deben respetar el contrato algebraico indicado (incluida asociatividad); las operaciones de punto flotante pueden variar por el reagrupamiento.

```rust
pub enum ParallelReduceError<E> {
    ZeroWorkers,
    Operation(E),
}

pub fn try_reduce_parallel<T, A, E, F, C>(
    items: &[T],
    worker_count: usize,
    identity: A,
    operation: F,
    combine: C,
) -> Result<A, ParallelReduceError<E>>
where
    T: Sync,
    A: Clone + Send,
    E: Send,
    F: Fn(A, &T) -> Result<A, E> + Sync,
    C: Fn(A, A) -> A + Sync;
```

Cero workers se rechaza antes de ejecutar las closures; la entrada vacía devuelve `identity`. Todas las particiones se completan antes de devolver el resultado. Si varias particiones fallan, se devuelve el error de la primera partición según el orden de entrada, no el primer error temporalmente observado. La combinación solo ocurre si todas las particiones tuvieron éxito. Los pánicos siguen el contrato de Runtime.

- La iteración paralela no promete orden de finalización; sí garantiza que cada elemento se procesa exactamente una vez en una ejecución normal.
- La reducción paralela requiere un operador de combinación explícito y asociativo para obtener resultados independientes de la partición. No se prometerá equivalencia bit a bit para operaciones de punto flotante reordenadas.
- Los errores de las tareas se propagarán sin convertirlos silenciosamente en valores por defecto. La política de agregación de varios errores se definirá antes de exponer la API.
- Los pánicos seguirán la política documentada de Runtime; Compute no debe tragárselos.
- La cancelación será cooperativa y no puede interrumpir de forma segura una closure arbitraria.

### Etapa D — pipelines

La primera versión está implementada como `ComputePipeline<'a, T>`, una cadena de etapas mutables sobre un único slice prestado. `ComputePipeline::new(&mut items)` toma un préstamo mutable exclusivo; `for_each`, `for_each_batch` y `for_each_parallel` ejecutan cada etapa inmediatamente y devuelven `Result<&mut Self, PipelineError>` para poder encadenarlas. `finish(self)` consume el pipeline y devuelve el préstamo del slice original.

```rust
use dinah::compute::ComputePipeline;

let mut values = [1, 2, 3, 4];
ComputePipeline::new(&mut values)
    .for_each(|value| *value += 1)?
    .for_each_batch(2, |batch| batch.reverse())?
    .for_each_parallel(2, |value| *value *= 2)?;
# Ok::<(), dinah::compute::PipelineError>(())
```

- **Ejecución:** inmediata, en el orden de las llamadas; no se almacenan closures ni se difiere el trabajo.
- **Ownership:** el pipeline mantiene un préstamo mutable exclusivo de la entrada. No posee ni crea una colección intermedia, y no asigna memoria propia.
- **Errores:** `PipelineError::Batch` y `PipelineError::Parallel` preservan el error de configuración de la etapa correspondiente. Una etapa fallida corta la cadena: las siguientes no se ejecutan. Las mutaciones previas no se revierten.
- **Paralelismo:** solo ocurre en la etapa que invoca explícitamente `for_each_parallel`; usa Runtime y conserva el contrato de concurrencia de esa función.
- **Límites:** esta primera versión transforma in-place el mismo slice. `map` y las reducciones siguen siendo funciones independientes porque producen valores de salida/acumuladores con ownership distinto. No hay planificación, cancelación diferida ni reordenación automática.
- **Pánicos:** se propagan según el contrato de la primitiva ejecutada; no se promete rollback.

## Contratos transversales

1. **Ownership:** preferir slices prestados; la salida solo se asigna cuando la firma lo dice explícitamente.
2. **Mutabilidad:** operaciones de lectura reciben referencias compartidas; operaciones que escriben reciben acceso mutable exclusivo o particiones disjuntas.
3. **Errores:** validar parámetros inválidos antes de empezar cuando sea viable; conservar los errores de usuario.
4. **Determinismo:** documentar orden de visitas, orden de resultados y comportamiento de reducciones por separado.
5. **Pánicos:** no ocultar pánicos ni prometer rollback de mutaciones que ya ocurrieron.
6. **Concurrencia:** ninguna operación se paraleliza automáticamente en la primera etapa; el paralelismo explícito evita sorpresas de coste.
7. **Rendimiento:** medir overhead, asignaciones, tamaños pequeños y grandes; no introducir SIMD ni complejidad de scheduler sin evidencia.
8. **Seguridad:** mantener `#![forbid(unsafe_code)]` hasta que una propuesta concreta, auditada y medida justifique revisar esa política.

## Secuencia de implementación y aceptación

- **077 — Diseñar Compute API:** responsabilidades, familias de operaciones y contratos documentados; sin declarar funciones inexistentes.
- **078 — Definir primitivas de procesamiento:** implementado con `for_each`, `map` y `try_reduce`, pruebas unitarias y ejemplos de documentación pública.
- **079 — Crear procesamiento por lotes:** implementado con `for_each_batch` y `BatchError`; tamaño cero validado antes de mutar, último lote parcial y slices prestados sin buffer temporal.
- **080 — Crear parallel iteration:** implementado con `for_each_parallel`, particiones disjuntas, `Scheduler::run_parallel`, pruebas de cobertura, entrada vacía y rechazo de cero workers.
- **081 — Crear reducción paralela:** implementado con `try_reduce_parallel` y `ParallelReduceError`; combinación en orden de partición, primer error según orden de entrada y documentación de identidad/asociatividad/determinismo.
- **082 — Crear pipelines:** implementado como `ComputePipeline` inmediato sobre slice prestado, con etapas secuenciales, por lotes y paralelas; errores tipados, detención tras fallo, sin buffers ocultos ni ejecución diferida.
- **083–089:** investigar SIMD, optimizar layouts, instrumentar y comparar secuencial/paralelo, y probar determinismo con resultados archivados.
- **090 — Documentar Compute:** manual de usuario basado en APIs ya implementadas, no en propuestas.

La CI remota valida formato, tests, Clippy y documentación pública. Los benchmarks pesados y comparaciones de rendimiento deben archivarse bajo `artifacts/benchmarks/` con comando, entorno y fecha; nunca se inventan cifras para cerrar un punto.

## SIMD y evidencia de rendimiento (puntos 083–084)

La investigación de alternativas, restricciones y la primera medición exploratoria están registradas en [`docs/simd-research.md`](simd-research.md). `benches/compute.rs` compara el bucle directo con `compute::for_each` en un kernel numérico seguro; las diferencias observadas son pequeñas/variables y no comparan aún una implementación SIMD. Por ello no se añade SIMD manual ni se afirma aceleración: el punto 084 sigue en progreso hasta medir una alternativa vectorial frente a la referencia escalar con evidencia reproducible. No se prometen vectorización automática ni alineación especial.

## Decisiones aplazadas

Quedan abiertas hasta que un caso de uso y pruebas las justifiquen: traits de kernel, tipos de configuración, tamaño óptimo de lote, política de cancelación por operación, representación de errores paralelos, estrategia de reducción flotante, pipelines diferidos, SIMD y dependencias externas. No se deben anticipar estas abstracciones solo para ampliar la API.
