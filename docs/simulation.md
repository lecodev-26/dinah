# dinah Simulation — diseño inicial de API

Estado: diseño inicial completado (105), reloj fijo (111), actualización determinista (112), colección inicial de entidades simuladas (113), integración de identidades con ECS (114), recorrido mediante Compute (115) y prueba de replay determinista (116) implementados. `dinah::simulation` ofrece `SimulationClock`, `SimulationTime`, `SimulationTimeError`, `Simulation<S>`, `SimulationStepError`, `SimulationEntityId`, `SimulationEntities<T>`, `SimulationEntitiesError`, `SimulationEntityBindings` y `SimulationEntityBindingError`. `SimulationEntities::for_each_mut` utiliza el recorrido secuencial de Compute para actualizar estados en orden de inserción; la ejecución paralela desde Simulation sigue siendo explícita y no se activa automáticamente.

## Objetivo y responsabilidades

Simulation modelará el avance explícito de un estado de simulación a través del tiempo. Debe reutilizar Core para contratos compartidos, ECS para entidades/componentes, Compute para operaciones sobre datos y Runtime solo cuando se necesite coordinar trabajo. Simulation define el significado de un paso, el timestep, el orden de actualización y el manejo de errores; no debe convertirse en otro scheduler ni en un sistema de almacenamiento.

- **Core:** tipos y errores compartidos, sin política de simulación.
- **ECS:** ciclo de vida de entidades y estado representado mediante componentes.
- **Compute:** procesamiento secuencial/paralelo explícito de buffers y slices.
- **Runtime:** coordinación de tareas cuando una operación la requiera; no define la semántica temporal.
- **Simulation:** configuración temporal, estado del paso, orden de fases y contratos de actualización reproducible.

La primera versión será Rust-first, segura y pequeña, sin dependencias externas, reloj global, I/O, asignación oculta ni ejecución paralela automática. Se mantiene `#![forbid(unsafe_code)]`.

## Límites de la primera versión

La primera API debe permitir construir y avanzar una simulación con tiempo explícito, validar la configuración y probar resultados reproducibles. No intentará ser un motor físico completo ni imponer una representación universal de posiciones, fluidos, celdas, rayos o agentes. Esos dominios podrán añadirse como modelos o ejemplos independientes cuando sus contratos y necesidades estén claros.

Fuera de alcance inicial: reloj de pared implícito, paso variable automático, renderizado, entrada/salida, networking, rollback, DSL de escenarios, sistema de plugins, scheduler oculto, garantía de tiempo real y determinismo bit a bit entre arquitecturas para operaciones de punto flotante.

## Forma conceptual de la API

Los nombres y firmas siguientes son una propuesta de diseño, no código disponible:

```rust
pub struct SimulationConfig {
    pub fixed_delta: Duration,
    pub max_steps_per_advance: NonZeroUsize,
}

pub struct SimulationTime {
    pub tick: u64,
    pub elapsed: Duration,
    pub delta: Duration,
}

pub enum SimulationError {
    ZeroDelta,
    StepLimitReached,
    TickOverflow,
    Update(/* error de dominio preservado */),
}

pub trait SimulationStep {
    type Error;

    fn update(&mut self, time: SimulationTime) -> Result<(), Self::Error>;
}
```

El reloj ya implementado proporciona el núcleo temporal de esta propuesta: `SimulationClock::new(delta)` rechaza un delta cero y `advance()` incrementa un tick y el tiempo acumulado de forma comprobada. Si el tick o la duración se desbordan, devuelve `SimulationTimeError` y deja el reloj intacto. La política de acumulación de tiempo externo y de pasos máximos por avance queda aplazada; el reloj no consulta tiempo de pared ni ejecuta callbacks. `Simulation<S>::step` calcula el siguiente tiempo en una copia del reloj, llama una vez al callback síncrono y solo confirma el reloj tras el éxito. Si el callback devuelve error, el tick no avanza; las mutaciones de dominio previas al error no se revierten automáticamente. Si el cálculo temporal falla, el callback ni siquiera se invoca.

## Entidades simuladas (punto 113)

`SimulationEntities<T>` conserva estados de entidad propiedad de Simulation en un vector ordenado. `insert` devuelve un `SimulationEntityId` monotónico; `get`, `get_mut`, `remove`, `iter`, `len` e `is_empty` exponen operaciones explícitas. La iteración respeta el orden de inserción y `remove` conserva el orden relativo de las entidades restantes. Los IDs eliminados no se reutilizan, y el agotamiento del espacio de IDs se informa antes de modificar la colección. La búsqueda por ID y el borrado son lineales en esta primera versión; no se afirma rendimiento constante ni almacenamiento ECS.

Estos IDs son deliberadamente distintos de `ecs::Entity`: esta colección permite definir primero el contrato de estado simulado sin duplicar ni asumir el ciclo de vida del `World`. El punto 114 implementa el puente explícito `SimulationEntityBindings`: `bind` solo acepta entidades vivas en el `World` suministrado y evita asignar dos IDs al mismo handle; `resolve` vuelve a comprobar la vida del handle y rechaza entidades eliminadas o generaciones obsoletas; `unbind` elimina únicamente la asociación, nunca destruye la entidad. El llamador debe suministrar consistentemente el mismo `World` propietario, porque los handles ECS son locales a cada mundo. Los IDs de este puente pertenecen a su propia secuencia y no son IDs de `SimulationEntities<T>`. La asociación no toma posesión del ciclo de vida ECS ni sincroniza automáticamente `despawn`; el llamador debe gestionar `unbind` cuando corresponda. Para procesar estados simulados, `SimulationEntities::for_each_mut` delega el recorrido mutable al primitivo secuencial `compute::for_each`, conserva el orden de inserción y no crea workers ni altera el reloj. Un pánico se propaga y las mutaciones anteriores no se revierten.

## Contratos que deben quedar definidos

### Tiempo y avance

- El paso de simulación se expresa con duración explícita; no se lee un reloj global.
- El timestep fijo será la ruta determinista de referencia. El paso variable, si se necesita, será una capacidad explícita posterior.
- Cada paso recibe un índice monotónico y un delta validado; el desbordamiento del contador debe detectarse, no envolver silenciosamente.
- Los límites de pasos por avance deben ser configurables y no pueden descartar tiempo de forma silenciosa.
- El orden de fases y el momento en que se incrementa el tick deben documentarse y probarse.

### Actualización, errores y pánicos

- Una actualización devuelve éxito o su error de dominio; no se reemplaza un fallo por un valor por defecto.
- No se promete rollback del estado si una actualización falla después de mutaciones. Si la atomicidad resulta necesaria, deberá diseñarse explícitamente mediante estado temporal o transacciones.
- Los pánicos no se convierten silenciosamente en errores ordinarios; se mantiene la política documentada de Runtime cuando se usen sus workers.
- Una actualización fallida no avanza silenciosamente el tick. El punto 112 debe fijar la regla exacta y cubrirla con tests.

### Determinismo

- Con la misma configuración, estado inicial y secuencia de entradas explícitas, las actualizaciones deben seguir el mismo orden lógico.
- No se promete determinismo temporal ni igualdad bit a bit de operaciones flotantes entre arquitecturas.
- Las iteraciones de mapas/sets no ordenados no pueden definir el orden observable del paso.
- Las semillas aleatorias, si se incorporan, deberán ser explícitas y serializables; no habrá aleatoriedad global implícita en el núcleo.

### Integración con ECS y Compute

- La integración con ECS se hará mediante asociaciones explícitas y deberá respetar préstamos, ciclo de vida de entidades y límites de queries realmente implementados.
- La integración con Compute es explícita: `for_each_mut` es secuencial; una operación no se vuelve paralela solo por procesar muchos elementos.
- No se ejecutarán sistemas ECS en paralelo hasta que el punto 100 tenga validación de accesos y aislamiento estructural seguros.
- La API no exigirá `Send + Sync` a todos los modelos si la ejecución secuencial no lo necesita.

## Secuencia propuesta de implementación

1. **105 — Diseño de API:** completado en este documento; firmas y límites iniciales definidos.
2. **106–110 — Dominios:** evaluar por separado física ASCII, fluidos, autómatas celulares, raycasting y enjambres. Cada integración debe tener un caso de uso, límites de datos y tests propios; no se importarán abstracciones sin validar.
3. **111 — Timestep:** implementado como `SimulationClock`, con delta fijo no nulo, tick, tiempo acumulado, avance explícito y errores de overflow comprobados. Quedan para después los límites de pasos por llamada y el acumulador de tiempo externo.
4. **112 — Actualización determinista:** completado en la primera versión mediante un callback síncrono por paso, tiempo explícito y commit del reloj solo tras éxito; el estado de dominio no tiene rollback automático.
5. **113–115 — Entidades e integración:** 113 crea la colección de estado simulado; 114 añade el puente explícito entre IDs simulados y handles ECS; 115 integra el recorrido mutable secuencial con Compute, conservando orden y sin activar paralelismo implícito.
6. **116–118 — Calidad de uso:** tests reproducibles (116), benchmark inicial de workloads concretos (117; mediciones de referencia aún pendientes) y ejemplos pequeños (118).
7. **119 — Documentación:** describir solo la API implementada, sus límites y resultados medidos.

## Pruebas deterministas (punto 116)

El test `deterministic_replay_reproduces_tick_trace_and_entity_state` ejecuta dos veces el mismo escenario con delta fijo y una secuencia explícita de entradas. Cada tick actualiza entidades en orden de inserción mediante `for_each_mut`; la traza registra el tick y cada par ID/estado. Las dos trazas y los estados finales deben coincidir exactamente. El test no lee reloj de pared, no usa fuentes de aleatoriedad ni depende del orden de finalización de workers. Esto verifica reproducibilidad lógica para esta carga secuencial concreta; no constituye una garantía de determinismo bit a bit entre arquitecturas ni de ejecución paralela.

## Benchmarks (punto 117)

`benches/simulation.rs` proporciona un benchmark ejecutable con `cargo bench --bench simulation`. Informa la arquitectura, usa nueve muestras y la mediana, y separa estos casos: 256 avances de `SimulationClock` (incluye construcción), 256 pasos de `Simulation` con una actualización pequeña (incluye construcción), recorrido mutable de `SimulationEntities<T>` y la combinación de iteración más `get` para tamaños de 64, 1.024 y 16.384 entidades. Cada muestra repite la operación 32 veces y normaliza el tiempo; la columna `ns_per_operation` divide además por ticks/entidades cuando corresponde.

El benchmark no contiene resultados archivados ni afirma una mejora de rendimiento: primero debe ejecutarse en runners de referencia y guardar el entorno y salida antes de extraer conclusiones. Los casos de entidades incluyen el coste de actualizar estados acumulativamente, y `iter + get` mide explícitamente la búsqueda lineal actual, no una query ECS. Los resultados son específicos de máquina y compilación release.

## Criterios de aceptación

- Los tipos propuestos se convierten en API solo tras fijar los puntos 111 y 112 y comprobar que no duplican responsabilidades de Core, ECS o Runtime.
- Los parámetros inválidos se rechazan antes de mutaciones observables cuando sea viable. `SimulationClock` rechaza delta cero.
- Tests del reloj cubren estado inicial, progresión del tick, delta cero y overflow de tick/duración sin mutación parcial. El fallo de actualización se prueba en el punto 112: el reloj permanece igual si el callback falla; el estado de dominio puede conservar mutaciones previas al error. El límite de avance y el acumulador externo siguen pendientes.
- Tests de determinismo comparan estados/resultados lógicos y no dependen del tiempo real ni del orden de finalización de workers.
- Los benchmarks separan coste de avance vacío, actualización por entidad y workloads de dominio; informan tamaño, configuración, entorno y mediana, sin generalizar resultados de microbenchmarks.
- Los ejemplos compilan contra la API real; las firmas conceptuales de este documento no se presentan como utilizables todavía.

## Decisiones aplazadas

Quedan abiertas: si se permite acumular tiempo externo, la política de pausa, la serialización del estado, los eventos, rollback/transacciones del estado de dominio, la integración concreta con cada dominio y la estrategia paralela. Se resolverán con pruebas y necesidades reales, no añadiendo abstracciones preventivas.
