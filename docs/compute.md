# dinah Compute — diseño inicial de la API

Estado: diseño para el punto 077. Este documento fija límites y decisiones para guiar la implementación; no declara que exista todavía un módulo `dinah::compute` ni una API pública de Compute.

## Objetivo y límites

Compute ofrecerá operaciones de procesamiento de datos que puedan ejecutarse primero de forma secuencial y, cuando haya una necesidad medida, de forma paralela. Debe reutilizar los contratos de Core y las primitivas de Runtime y Memory sin fusionar sus responsabilidades.

- **Core** valida tipos y contratos compartidos; no ejecuta kernels.
- **Memory** posee almacenamiento contiguo y fija sus límites de capacidad; no decide cómo procesar los datos.
- **Runtime** ejecuta y coordina trabajo; Compute define la semántica de las operaciones de datos.
- **Compute** ofrece operaciones puras o explícitamente mutables sobre datos prestados, con límites claros de errores, orden y concurrencia.

La primera versión será pequeña, genérica y sin dependencias externas. No habrá DSL, grafo de tareas, compilador JIT, allocator propio ni garantías de tiempo real.

## Diseño de API por etapas

### Etapa A — primitivas secuenciales

Empezar con funciones genéricas sobre slices prestados, sin asignaciones implícitas ni estado global. La forma conceptual inicial es:

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

Estas firmas son propuestas de diseño, no código compilable ni API publicada. Antes de implementarlas se revisarán nombres, errores, documentación de asignaciones y ergonomía. En particular, `map` asigna un `Vec` y debe documentarlo; una futura variante con salida prestada o buffer del llamador se evaluará solo si hace falta.

### Etapa B — lotes explícitos

El procesamiento por lotes debe describir claramente el tamaño de lote y qué sucede con el último lote incompleto. La primera implementación no debe copiar todos los datos a un buffer temporal ni crear una tarea por elemento. Las vistas de lote serán préstamos a segmentos del slice original cuando sea posible.

Un tamaño de lote cero se rechazará antes de iniciar trabajo. Los errores de validación deben ocurrir antes de mutar datos siempre que la operación pueda validar el lote completo de antemano. No se prometerá rollback ante pánicos del código del usuario.

### Etapa C — iteración y reducción paralelas

La paralelización será opt-in y estará conectada a las primitivas reales de Runtime, no a un pool oculto independiente. Los límites `Send`/`Sync`, propiedad de closures y vida útil de referencias se decidirán por operación y se documentarán. El primer contrato paralelo usará particiones disjuntas y no compartirá referencias mutables entre workers.

- La iteración paralela no promete orden de finalización; sí debe garantizar que cada elemento elegible se procesa exactamente una vez en una ejecución normal.
- La reducción paralela requiere un operador de combinación explícito y asociativo para obtener resultados independientes de la partición. No se prometerá equivalencia bit a bit para operaciones de punto flotante reordenadas.
- Los errores de las tareas se propagarán sin convertirlos silenciosamente en valores por defecto. La política de agregación de varios errores se definirá antes de exponer la API.
- Los pánicos seguirán la política documentada de Runtime; Compute no debe tragárselos.
- La cancelación será cooperativa y no puede interrumpir de forma segura una closure arbitraria.

### Etapa D — pipelines

Un pipeline solo se incorporará cuando las operaciones básicas demuestren un caso de uso real. La API deberá indicar si las etapas se ejecutan de forma inmediata o diferida, quién posee los datos intermedios, cómo se propagan errores y cómo se cancelan. La primera versión evitará ejecución diferida implícita, almacenamiento oculto y reordenación automática de etapas.

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
- **078 — Definir primitivas de procesamiento:** seleccionar e implementar la primera API secuencial, con tests y documentación pública.
- **079 — Crear procesamiento por lotes:** validar tamaño, último lote, errores y no asignación oculta.
- **080 — Crear parallel iteration:** particiones disjuntas, integración explícita con Runtime, pruebas de cobertura y errores.
- **081 — Crear reducción paralela:** contrato del operador, orden de combinación, errores y límites de determinismo.
- **082 — Crear pipelines:** solo después de las primitivas anteriores; semántica explícita de ejecución, ownership y errores.
- **083–089:** investigar SIMD, optimizar layouts, instrumentar y comparar secuencial/paralelo, y probar determinismo con resultados archivados.
- **090 — Documentar Compute:** manual de usuario basado en APIs ya implementadas, no en propuestas.

La CI remota valida formato, tests, Clippy y documentación pública. Los benchmarks pesados y comparaciones de rendimiento deben archivarse bajo `artifacts/benchmarks/` con comando, entorno y fecha; nunca se inventan cifras para cerrar un punto.

## Decisiones aplazadas

Quedan abiertas hasta que un caso de uso y pruebas las justifiquen: traits de kernel, tipos de configuración, tamaño óptimo de lote, política de cancelación por operación, representación de errores paralelos, estrategia de reducción flotante, pipelines diferidos, SIMD y dependencias externas. No se deben anticipar estas abstracciones solo para ampliar la API.
