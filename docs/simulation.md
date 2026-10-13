# dinah Simulation — diseño inicial de API

Estado: diseño del contrato inicial (punto 105). No existe todavía un módulo `dinah::simulation`; este documento fija límites y criterios para implementarlo sin presentar APIs propuestas como si estuvieran disponibles.

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

Esta forma solo sirve para concretar las decisiones que deben resolverse antes de implementar. La firma definitiva debe evitar borrar errores de dominio, validar valores inválidos antes de mutar cuando sea posible y dejar explícito quién posee el estado. La política de `advance` —un paso fijo por llamada frente a consumir un acumulador— se decidirá junto con el punto 111, sin introducir tiempo de pared en el contrato básico.

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

- La integración con ECS se hará en un punto separado y deberá respetar préstamos, ciclo de vida de entidades y límites de queries realmente implementados.
- La integración con Compute será explícita: una operación no se vuelve paralela solo por procesar muchos elementos.
- No se ejecutarán sistemas ECS en paralelo hasta que el punto 100 tenga validación de accesos y aislamiento estructural seguros.
- La API no exigirá `Send + Sync` a todos los modelos si la ejecución secuencial no lo necesita.

## Secuencia propuesta de implementación

1. **105 — Diseño de API:** fijar responsabilidades, configuración, tiempo, errores, orden y no-objetivos. Este documento cubre el contrato inicial; la implementación se mantiene pendiente.
2. **106–110 — Dominios:** evaluar por separado física ASCII, fluidos, autómatas celulares, raycasting y enjambres. Cada integración debe tener un caso de uso, límites de datos y tests propios; no se importarán abstracciones sin validar.
3. **111 — Timestep:** definir duración fija, contador, avance, límites y desbordamientos.
4. **112 — Actualización determinista:** definir orden de fases, semántica de error y qué estado/tick se conserva ante un fallo.
5. **113–115 — Entidades e integración:** diseñar el estado simulado y después conectar ECS y Compute, sin anticipar queries o ejecución paralela que aún no existan.
6. **116–118 — Calidad de uso:** tests reproducibles, benchmarks de workloads concretos y ejemplos pequeños.
7. **119 — Documentación:** describir solo la API implementada, sus límites y resultados medidos.

## Criterios de aceptación

- Los tipos propuestos se convierten en API solo tras fijar los puntos 111 y 112 y comprobar que no duplican responsabilidades de Core, ECS o Runtime.
- Los parámetros inválidos se rechazan antes de mutaciones observables cuando sea viable.
- Tests cubren paso inicial, progresión del tick, timestep inválido, límite de avance, overflow y fallo de actualización.
- Tests de determinismo comparan estados/resultados lógicos y no dependen del tiempo real ni del orden de finalización de workers.
- Los benchmarks separan coste de avance vacío, actualización por entidad y workloads de dominio; informan tamaño, configuración, entorno y mediana, sin generalizar resultados de microbenchmarks.
- Los ejemplos compilan contra la API real; las firmas conceptuales de este documento no se presentan como utilizables todavía.

## Decisiones aplazadas

Quedan abiertas: la firma final de `advance`, si se permite acumular tiempo externo, la política de pausa, la serialización del estado, los eventos, la semántica transaccional de errores, la integración concreta con cada dominio y la estrategia paralela. Se resolverán con pruebas y necesidades reales, no añadiendo abstracciones preventivas.
