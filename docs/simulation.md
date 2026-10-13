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

## API implementada

La API disponible en `dinah::simulation` incluye `SimulationClock`, `SimulationTime`, `SimulationTimeError`, `Simulation<S>`, `SimulationStepError`, `SimulationEntityId`, `SimulationEntities<T>`, `SimulationEntitiesError`, `SimulationEntityBindings` y `SimulationEntityBindingError`. El siguiente fragmento conserva solo el esquema conceptual inicial; no es una API adicional ni una firma que deba copiarse literalmente:

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
2. **106–110 — Dominios (pendientes tras auditoría):** no hay implementaciones ni dependencias para estos dominios en la primera versión. Cada propuesta debe permanecer opcional y separada del reloj y del núcleo de Simulation, con un caso de uso acotado, límites de datos, tests de dominio y revisión de licencia/procedencia si se toma código externo. Criterios mínimos por dominio:
   - **106 — Física ASCII:** rejilla discreta, reglas explícitas de movimiento/obstáculos y demo pequeña reproducible; no se promete un motor físico general.
   - **107 — Fluidos:** escoger modelo y variables antes de implementar; definir estabilidad numérica y pruebas de valores finitos/conservación apropiadas al modelo; no prometer CFD general.
   - **108 — Autómatas celulares:** definir rejilla, vecindad, bordes y actualización síncrona (snapshot/buffer doble); probar patrones conocidos y límites.
   - **109 — Raycasting:** decidir 2D o 3D, tipos numéricos y contrato de impacto; cubrir miss, bordes, origen dentro del objeto y tolerancias antes de optimizar.
   - **110 — Enjambres:** elegir un modelo acotado (por ejemplo, flocking 2D), definir vecindad y actualización sin dependencia del orden de recorrido; probar reproducibilidad y límites numéricos.
   Ninguno de estos puntos se considera completado por la API genérica de Simulation ni por los benchmarks de su reloj/colección de entidades.
3. **111 — Timestep:** implementado como `SimulationClock`, con delta fijo no nulo, tick, tiempo acumulado, avance explícito y errores de overflow comprobados. Quedan para después los límites de pasos por llamada y el acumulador de tiempo externo.
4. **112 — Actualización determinista:** completado en la primera versión mediante un callback síncrono por paso, tiempo explícito y commit del reloj solo tras éxito; el estado de dominio no tiene rollback automático.
5. **113–115 — Entidades e integración:** 113 crea la colección de estado simulado; 114 añade el puente explícito entre IDs simulados y handles ECS; 115 integra el recorrido mutable secuencial con Compute, conservando orden y sin activar paralelismo implícito.
6. **116–118 — Calidad de uso:** tests reproducibles (116), benchmark inicial medido y archivado en cinco plataformas (117) y ejemplo ejecutable `examples/simulation_basics.rs` (118).
7. **119 — Documentación:** esta guía recoge la API implementada, sus límites, mediciones de referencia y un ejemplo ejecutable. La documentación seguirá evolucionando al ampliar el módulo.

## Pruebas deterministas (punto 116)

El test `deterministic_replay_reproduces_tick_trace_and_entity_state` ejecuta dos veces el mismo escenario con delta fijo y una secuencia explícita de entradas. Cada tick actualiza entidades en orden de inserción mediante `for_each_mut`; la traza registra el tick y cada par ID/estado. Las dos trazas y los estados finales deben coincidir exactamente. El test no lee reloj de pared, no usa fuentes de aleatoriedad ni depende del orden de finalización de workers. Esto verifica reproducibilidad lógica para esta carga secuencial concreta; no constituye una garantía de determinismo bit a bit entre arquitecturas ni de ejecución paralela.

## Benchmarks (punto 117)

**Punto 117 completado con mediciones reales de CI** en el commit `83cbcc3f3a36bd38ee661ae12075a0e27b654414`, workflow [dinah benchmarks, ejecución 38484605568](https://github.com/lecodev-26/dinah/actions/runs/38484605568). Las salidas completas y los entornos están archivados en [`artifacts/benchmarks/83cbcc3f3a36bd38ee661ae12075a0e27b654414/run-38484605568/`](../artifacts/benchmarks/83cbcc3f3a36bd38ee661ae12075a0e27b654414/run-38484605568/), con resultados para Linux x86-64, Linux ARM64, Windows x86-64, macOS x86-64 y macOS ARM64.

`benches/simulation.rs` se ejecuta en release, usa nueve muestras y reporta medianas; cada muestra repite la operación 32 veces. Incluye 256 avances de `SimulationClock` (con construcción), 256 pasos de `Simulation` con actualización pequeña (con construcción), recorrido mutable de colecciones de 64, 1.024 y 16.384 entidades y 16 búsquedas `get` distribuidas por colección. Tabla de referencia para los casos temporales y el tamaño de 16.384 entidades:

| Plataforma | Reloj (ns/avance) | Paso (ns/paso) | Recorrido mutable (ns/entidad) | 16 búsquedas lineales (ns/búsqueda) |
| --- | ---: | ---: | ---: | ---: |
| Linux x86-64 | 7,801 | 12,547 | 0,368 | 2.437,000 |
| Linux ARM64 | 5,855 | 8,000 | 0,438 | 1.792,938 |
| Windows x86-64 | 7,836 | 12,242 | 0,346 | 2.446,062 |
| macOS x86-64 | 10,488 | 10,305 | 0,823 | 4.990,062 |
| macOS ARM64 | 2,395 | 6,012 | 0,281 | 1.533,875 |

Son observaciones específicas de máquinas y compilaciones concretas, no garantías ni comparaciones controladas entre sistemas operativos. La construcción está incluida en los casos de reloj y paso, por lo que no deben interpretarse como coste aislado por tick en régimen estable. El caso de búsquedas mide la búsqueda lineal actual sobre 16 IDs repartidos por la colección; no es una query ECS. Estos datos establecen una línea base, no prueban una mejora frente a una versión anterior ni justifican por sí solos cambiar la representación de almacenamiento. El workflow de benchmarks ejecuta la suite y archiva la salida y el entorno de los cinco runners.

## Ejemplo ejecutable (punto 118)

`cargo run --example simulation_basics` muestra un `Simulation<Vec<Particle>>` que avanza tres pasos con delta fijo de 16 ms y actualiza posición usando el delta lógico entregado al callback. Después crea una `SimulationEntities<Particle>`, actualiza sus estados con `for_each_mut` y recorre los resultados en orden de inserción. El ejemplo no usa reloj de pared, workers implícitos ni integra entidades simuladas con ECS automáticamente.

## Criterios y límites actuales

- La API publicada corresponde al código de `src/simulation.rs`; el diseño evita duplicar responsabilidades de Core, ECS o Runtime.
- `SimulationClock` rechaza delta cero y comprueba overflow de tick y duración antes de mutar su estado.
- `Simulation::step` invoca un callback síncrono con tiempo explícito y confirma el reloj solo tras éxito. El estado de dominio no tiene rollback automático si el callback muta y luego falla.
- Las pruebas de replay comparan trazas y estado para una secuencia explícita, sin reloj real ni aleatoriedad implícita. No prometen determinismo bit a bit entre arquitecturas ni seguridad de ejecución paralela.
- Los benchmarks informan tamaño, configuración, entorno y medianas. Son líneas base de microbenchmarks, no pruebas de mejora ni predicciones universales.
- `cargo run --example simulation_basics` utiliza la API real. Los pasos máximos por avance, el acumulador de tiempo externo, la política de pausa, rollback/transacciones, serialización y modelos de dominio siguen fuera del alcance implementado.

## Decisiones aplazadas

Quedan abiertas: si se permite acumular tiempo externo, la política de pausa, la serialización del estado, los eventos, rollback/transacciones del estado de dominio, la integración concreta con cada dominio y la estrategia paralela. Se resolverán con pruebas y necesidades reales, no añadiendo abstracciones preventivas.
