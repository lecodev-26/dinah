# dinah ECS — diseño inicial

Estado: diseño inicial aprobado; `Entity`, `Component`, el ciclo de vida inicial de `World` y el adaptador explícito de sistemas a Runtime están implementados. El punto 099 añade un adaptador interno que procesa secuencialmente los componentes contiguos mediante `compute::for_each`, manteniendo alineados los handles de entidad. El almacenamiento todavía no está conectado a `World` ni expuesto como API pública. Las queries siguen pendientes. Las decisiones de este documento son una propuesta inicial revisable, no una garantía de estabilidad pública.

## Objetivos

- Representar entidades como identificadores ligeros, sin datos de dominio embebidos.
- Guardar componentes tipados en almacenamiento contiguo, separado por tipo, con acceso seguro y sin `unsafe`.
- Hacer explícitos ownership, errores, eliminación, reutilización de IDs y semántica de iteración.
- Permitir queries legibles y ofrecer después integración explícita con Runtime y Compute.
- Medir cargas reales antes de introducir archetypes, generación de código, SIMD o paralelismo automático.

Fuera de alcance inicial: reflexión/serialización automática, relaciones jerárquicas integradas, eventos globales, scheduler de sistemas, macros de derive, plugins dinámicos y garantías de tiempo real.

## Decisión inicial: sparse sets tipados

La primera versión usará un almacenamiento independiente por tipo de componente, implementado con vectores contiguos y un índice disperso para localizar una entidad. No se adoptará inicialmente un diseño archetype/table: mover filas entre tablas cuando cambia la composición añade complejidad de movimiento y préstamo antes de tener benchmarks representativos. La decisión podrá revisarse si las cargas medidas favorecen consultas por composición sobre accesos por tipo.

Un `ComponentStorage<T>` conceptual contiene:

- `entities: Vec<Entity>`: entidades en el mismo orden que los componentes.
- `components: Vec<T>`: valores contiguos, alineados por índice con `entities`.
- `sparse: Vec<Option<usize>>`: índice de fila para cada índice de entidad conocido; la entrada se valida también contra la generación.

Al borrar una fila se usa `swap_remove`; la última fila se mueve a la posición vacante y su índice disperso se actualiza. Por tanto, el orden de iteración no está garantizado tras mutaciones estructurales. Los slices de un almacenamiento son contiguos, pero no se promete que los componentes de distintos tipos estén juntos.

## Identidad y ciclo de vida de Entity

`Entity` será un valor pequeño con `index` y `generation` (ambos enteros sin signo). La generación evita que un identificador guardado por el usuario vuelva a referirse accidentalmente a otra entidad después de destruir y reutilizar su índice.

- La creación la administra `World`; no se exige al usuario inventar índices ni se reutiliza `CoreId` como allocator.
- `World` mantiene generaciones y una lista libre de índices reutilizables.
- La validez requiere que el índice exista, la generación coincida y la entidad esté viva.
- Destruir una entidad invalida el handle, elimina sus componentes de todos los storages y solo después permite reutilizar el índice con una generación nueva.
- El overflow de generación debe tener una política explícita y probada; nunca debe permitir que un handle antiguo se valide silenciosamente. La implementación podrá retirar ese índice si no puede incrementar la generación.
- Los handles son locales a un `World`; no implican unicidad global ni estabilidad entre procesos, serializaciones o sesiones.


### Contrato implementado: `World` (punto 094)

`dinah::ecs::World` es el propietario single-owner de la tabla de entidades. `spawn()` devuelve `Result<Entity, WorldError>`; `despawn(entity)` invalida un handle vivo o devuelve `EntityNotAlive` para handles inexistentes, obsoletos o ya destruidos. `contains`, `len` e `is_empty` permiten inspeccionar el estado. Los índices liberados se reutilizan con generación incrementada; si una generación llega a `u32::MAX`, el slot se retira para impedir el wraparound. La saturación del espacio de índices devuelve `EntityIndexExhausted`. Este punto aún no implementa almacenamiento ni eliminación de componentes, que pertenecen a los puntos posteriores. Tests unitarios y de integración cubren liveness, conteos, reutilización, rechazo de handles obsoletos y retirada por overflow.

### Adaptador explícito de sistemas a Runtime (punto 098)

`World::run_system(&mut self, &Scheduler, closure)` permite ejecutar una operación de sistema que recibe acceso exclusivo a `World` mediante el Runtime público. En Runtime v0.1, `Scheduler::schedule` ejecuta la closure inmediatamente en el hilo llamador; por tanto, el adaptador no crea hilos, no retiene el mundo ni promete ejecución en background. El resultado anidado distingue un rechazo del scheduler (`ScheduleError`) del error propio del sistema. El préstamo exclusivo impide ejecutar simultáneamente dos closures mutables sobre el mismo `World` a través de esta API. La integración es explícita y síncrona; los sistemas paralelos quedan fuera de este punto.

### Metadatos tipados de acceso (punto 100, en progreso)

`SystemAccess` permite declarar `read::<T>()` y `write::<T>()` por tipo de componente y consultar el modo declarado. Las declaraciones repetidas se deduplican; si un tipo aparece como lectura y escritura, prevalece `Write`, que es la interpretación conservadora. `conflicts_with` devuelve conflicto si ambos sistemas declaran el mismo tipo y al menos uno lo escribe; lectura/lectura y tipos distintos son compatibles. El descriptor es metadata pública: no verifica que una closure respete su declaración, no presta componentes y no ejecuta sistemas. Por tanto, no debe usarse como prueba suficiente para lanzar workers todavía. Tests unitarios y de contrato público cubren lectura/lectura, lectura/escritura, escritura/escritura, tipos distintos, declaraciones repetidas y componentes `!Send`/`!Sync`.

### Planificador determinista de metadatos (punto 100, en progreso)

`SystemPlan::from_accesses` crea fases de índices de sistemas de forma determinista y preserva el orden de entrada. Usa una estrategia conservadora de lotes contiguos: añade un sistema a la fase actual si no entra en conflicto con ninguno de sus miembros; si hay conflicto, abre una nueva fase. Así, los sistemas conflictivos mantienen su orden relativo y los accesos compatibles pueden compartir fase. La API expone `batches()`, `len()` e `is_empty()`, y acepta entrada vacía sin crear fases. Los tests verifican agrupación, orden y repetibilidad. Este planificador opera solo sobre declaraciones: no ejecuta closures, no prueba que coincidan con sus declaraciones, no detecta operaciones estructurales ni hace que `World` sea `Send`/`Sync`; por tanto, aún no habilita ejecución paralela segura.

### Contrato de seguridad para sistemas paralelos (punto 100, en progreso)

La ejecución paralela de sistemas ECS no se habilita todavía. El scheduler de Runtime puede ejecutar closures independientes en paralelo, pero eso no convierte un `World` en un objeto seguro para compartir: `World` es single-owner, `Component` admite tipos que no son `Send`/`Sync`, y el almacenamiento de componentes aún no está conectado a `World`. Pasar el mismo `&mut World` a varios workers o capturar componentes compartidos sin análisis de préstamos sería incorrecto.

Antes de implementar la ejecución, cada sistema debe declarar de forma explícita los tipos de componentes que lee y escribe. Dos sistemas solo podrán pertenecer al mismo lote paralelo si sus conjuntos de acceso son compatibles: lectura/lectura del mismo tipo es compatible; escritura/lectura, lectura/escritura y escritura/escritura del mismo tipo son conflictos. Una declaración de acceso ausente, dinámica o ambigua debe bloquear la ejecución paralela, no asumir que es segura. Los componentes que crucen límites de worker tendrán que satisfacer los bounds `Send`/`Sync` necesarios en la API concurrente, sin restringir por ello el contrato general de `Component` para el uso secuencial.

La futura implementación debe además garantizar que las operaciones estructurales (`spawn`, `despawn`, inserción/eliminación de componentes) no invaliden referencias activas. Inicialmente podrán ejecutarse en una fase exclusiva separada de las fases de lectura/escritura de componentes. Los lotes paralelos deben ser deterministas en la selección y en la presentación de resultados, aunque el orden temporal de finalización no sea estable. El número de workers será explícito y no se prometerá aceleración para cargas pequeñas.

**Criterio de aceptación:** descriptor tipado, detector básico de conflictos y planificador determinista de lotes contiguos implementados; falta una API de ejecución que valide declaraciones y no permita compartir `World` mutablemente entre workers; tests de límites (tipos repetidos, acceso de lectura/escritura, lote vacío y worker count inválido); y CI multiplataforma antes de marcar el punto 100 como completado. Hasta entonces, `World::run_system` sigue siendo síncrono y no existe una API pública de sistemas paralelos ECS.

### Consistencia de slots, generaciones y storage (punto 101)

Las pruebas de consistencia comprueban invariantes internas tras secuencias de mutaciones, no solo el resultado de una operación aislada. Para `World`, el contador `alive_count` debe coincidir con el número de slots vivos; cada índice de la lista libre aparece una sola vez y referencia un slot no vivo ni retirado; todo slot no vivo y no retirado debe estar en la lista libre; y los slots vivos o retirados no pueden estar libres. La generación nunca se valida si el slot no está vivo.

Para `ComponentStorage<T>`, las columnas `entities` y `components` deben conservar la misma longitud. Cada entidad almacenada debe apuntar desde su índice disperso a su fila actual, y cada entrada sparse presente debe referenciar una fila válida cuyo índice de entidad coincide con la posición sparse. Las pruebas verifican estas propiedades después de inserciones, reemplazos, eliminaciones con `swap_remove` y reutilización del mismo índice con una generación distinta. El storage sigue siendo interno y no está integrado con `World`, por lo que estas pruebas no afirman todavía que `despawn` elimine componentes del mundo.

### Contrato implementado: `Component` (punto 093)

`dinah::ecs::Component` es un trait marcador público con `Component for T` para todo `T: 'static`. No aporta métodos ni exige derive/manual boilerplate; hace explícito el bound conceptual en las futuras APIs de storage y `World`. La implementación blanket es intencional: el contrato inicial es la duración de vida requerida por `Any`/`TypeId`, no la capacidad de enviar o compartir el valor entre hilos. Tests unitarios y de integración comprueban tipos comunes y `Rc<Cell<u32>>`, que no es `Send` ni `Sync`.

### Primer contrato implementado: `Entity` (punto 092)

`dinah::ecs::Entity` es un handle `Copy` con dos campos privados `u32`: índice y generación. Expone `index()` y `generation()` como accesores de solo lectura y una representación diagnóstica `indexvgen` mediante `Display`. Su constructor es privado para usuarios externos; el `World` será responsable de emitir handles válidos. La implementación no asigna IDs, no prueba por sí sola que un handle esté vivo y no promete unicidad fuera del mundo propietario.

Los tests verifican que igualdad y hashing distinguen generaciones distintas del mismo índice, que handles equivalentes se comparan igual, que los valores máximos se conservan y que el handle ocupa ocho bytes en la plataforma objetivo de Rust (dos `u32`). La validación de vida útil y reutilización la realiza `World`; el handle por sí solo no demuestra que la entidad siga viva.

### Almacenamiento contiguo e iteración (puntos 096–097)

`ComponentStorage<T>` es un detalle interno del ECS con tres vectores: `entities` y `components` forman columnas contiguas alineadas por fila; `sparse` mapea el índice de entidad a una fila candidata. Antes de devolver o quitar un valor se comprueba la identidad completa de `Entity`, incluida la generación. Reemplazar el componente de la misma entidad conserva la fila; reutilizar un índice con otra generación elimina la fila antigua y añade la nueva. Al borrar una fila se usa `swap_remove` y se actualiza el índice disperso de la fila movida.

Las inserciones reservan capacidad en todas las columnas que puedan crecer antes de modificar sus contenidos. Un error de reserva devuelve `StorageError::AllocationFailed` y no deja las columnas con longitudes diferentes. Esta estructura aún no valida por sí sola que el handle esté vivo en `World`; esa comprobación pertenece al propietario y la integración posterior. El punto 096 no afirma que `World::insert/get/remove` ni las queries estén implementadas.

El punto 097 añade `iter()` e `iter_mut()`, que recorren linealmente las dos columnas alineadas y producen pares `(Entity, &T)` o `(Entity, &mut T)` sin buscar cada fila en el índice disperso. La iteración mutable solo concede préstamos mutables de filas distintas y no ofrece operaciones estructurales sobre el storage. Ambos recorridos siguen el orden físico actual de las filas, que puede cambiar después de inserciones o eliminaciones; no es un orden estable de entidades. Las pruebas comprueban pares alineados, storage vacío y actualización mutable sin alterar la forma de las columnas.

## Componentes

Un componente es cualquier tipo Rust `'static`. El punto 093 expone `dinah::ecs::Component` como trait marcador con implementación blanket para todos los tipos `'static`: los usuarios no necesitan implementar nada manualmente y la trait sirve como bound legible en APIs ECS. No se exige `Send + Sync`: `World` es inicialmente single-owner y tipos como `Rc<Cell<T>>` siguen siendo componentes válidos. Si una API concurrente futura necesita límites más estrictos, los impondrá en esa API en vez de restringir prematuramente todos los componentes.

Cada `TypeId` identifica el storage del tipo en un `World`. El almacenamiento se mantendrá detrás de un registro privado type-erased (`Any`) y cada acceso tipado comprobará el downcast; un desacuerdo de tipos se tratará como un error/invariante interna, nunca mediante punteros crudos. No se expondrán referencias que sobrevivan a una mutación estructural incompatible del `World`.

## World y operaciones iniciales

`World` es el propietario de entidades, generaciones, lista libre y storages. La API se implementará incrementalmente con operaciones equivalentes a:

- `spawn` / `despawn` para crear y destruir entidades.
- `insert<T>` para añadir o reemplazar un componente.
- `get<T>` / `get_mut<T>` para acceder a un componente existente.
- `remove<T>` para retirar un componente.
- `contains<T>` y conteos para inspección explícita.

Los nombres y tipos exactos de error se fijarán en los puntos 092–094, junto con pruebas de handle inválido, entidad muerta, duplicación/reemplazo, falta de componente y capacidad. `World` será inicialmente single-owner y no sincronizado internamente; la concurrencia se añadirá por APIs explícitas que puedan demostrar préstamos disjuntos.

## Contrato de queries (punto 095)

Esta sección fija la semántica de la primera API de queries; no afirma que las queries ni el almacenamiento estén implementados. La API concreta se incorporará junto con los storages en los puntos 096–097, de modo que el compilador pueda validar los préstamos reales.

### Selección y resultados

- Una query selecciona entidades vivas que tienen todos los tipos de componente requeridos. Una entidad que carece de cualquiera de ellos no aparece en el resultado.
- La primera versión se limita a una combinación pequeña y explícita de componentes tipados; no incluye filtros arbitrarios, expresiones dinámicas, joins generales ni generación de código.
- El conjunto consultado pertenece a un único `World`. Los `Entity` devueltos son handles de ese mundo y deben validarse por índice, generación y liveness.
- Las queries no crean entidades ni componentes y no realizan cambios estructurales implícitos.
- Una query sin coincidencias es un resultado vacío válido, no un error.

### Préstamos y mutabilidad

- Las queries de solo lectura pueden exponer `&T` mientras se mantenga el préstamo compartido de `World`.
- Una query con escritura debe expresar los componentes mutables en el tipo de su API y excluir aliasing mutable. No se permitirá combinar accesos incompatibles al mismo tipo mediante parámetros de query duplicados.
- Mientras existan referencias de una query, el sistema de tipos debe impedir mutaciones estructurales del `World` que invaliden esas referencias.
- Si la API genérica para combinar préstamos no puede implementarse de forma segura en Rust estable, se empezará con una superficie menor (incluyendo solo lectura) en lugar de introducir `unsafe` o prometer una capacidad no demostrada.
- No se promete `Send`/`Sync` para resultados de query: los componentes y `World` iniciales son single-owner, y el paralelismo se definirá por separado.

### Orden y complejidad

- El orden de resultados no forma parte del contrato estable y puede cambiar tras inserciones, eliminaciones o reutilización de filas.
- La implementación inicial puede recorrer un storage conductor y comprobar los demás componentes; elegir el storage más pequeño es una optimización posterior, no una garantía observable.
- Las queries no prometen complejidad constante por entidad ni rendimiento específico hasta que los puntos de almacenamiento, iteración y benchmarks aporten evidencia.

### Casos que deben cubrir las pruebas

1. Coincidencia completa, componente ausente y query vacía.
2. Entidades destruidas y handles obsoletos excluidos después de reutilizar un índice.
3. Varias entidades coincidentes y componentes alineados con su entidad, incluso tras `swap_remove`.
4. Préstamos de lectura coexistentes y rechazo en compilación de accesos incompatibles cuando la forma de la API lo permita.
5. Ausencia de cambios estructurales como efecto secundario de iterar.

Estos casos son criterios de aceptación para la implementación futura; el punto 095 solo queda cerrado como contrato de diseño documentado.

## Límites de seguridad y errores

El crate mantiene `#![forbid(unsafe_code)]`. La implementación preferirá `Vec`, `HashMap<TypeId, Box<dyn Any>>` y downcasts seguros, con las restricciones de `borrow checker` visibles en la API. Las operaciones que puedan fallar por una entrada inválida deben devolver errores tipados. Un fallo de reserva de memoria no debe dejar un `World` con entidades o columnas desincronizadas; las operaciones que necesitan varias mutaciones deben validar y reservar antes de comprometer cambios cuando sea posible.

La destrucción de una entidad debe dejar todos los storages coherentes. Las pruebas cubrirán componentes en varios storages, filas movidas por `swap_remove`, reutilización de índice y rechazo de handles obsoletos. Los pánicos de destructores siguen la semántica de Rust y no se promete recuperación transaccional frente a un pánico.

## Integración futura

- **Memory (096):** reutilizar las lecciones de `Buffer<T>` y `SoaBuffer<A, B>`, pero no forzar el buffer fijo actual a ser un storage de tamaño dinámico sin evidencia de que sus contratos encajen.
- **Compute (099):** `ComponentStorage::for_each_mut` reutiliza `compute::for_each` sobre la columna contigua de componentes y empareja cada elemento con su `Entity` en el mismo índice. La operación es secuencial, no permite mutaciones estructurales desde el callback y conserva el tamaño/alineación de las columnas. No es todavía una API pública de `World`: conectar los storages a `World`, añadir queries y diseñar particiones paralelas son tareas separadas; no se promete paralelismo ni speedup.
- **Runtime (098, 100):** `World::run_system` conecta explícitamente un sistema síncrono con `Scheduler`. El punto 100 permanece en progreso: el contrato de seguridad exige declaraciones explícitas de componentes leídos/escritos, rechazo de lotes con conflictos y aislamiento de cambios estructurales; todavía no hay ejecución paralela ECS implementada. No se introduce scheduling oculto en operaciones de `World`.
- **Consistencia (101):** invariantes para generaciones, lista libre, alineación de columnas e índices dispersos, y ausencia de componentes en entidades muertas.
- **Benchmarks (102–103):** comparar sparse sets con un baseline simple, registrar tamaño de entidad/componentes, composición, lecturas/escrituras, inserciones/borrados y distribución de queries. No afirmar superioridad universal ni adoptar archetypes solo por intuición.

## Secuencia de aceptación

1. **091 — Diseño ECS:** decisiones, invariantes, límites y secuencia de integración documentados aquí.
2. **092–094 — Entity, Component, World:** implementados los handles, el bound público y el ciclo de vida inicial de World; el storage de componentes permanece fuera de estos puntos.
3. **095–097 — Queries, almacenamiento e iteración:** contrato de queries, storage contiguo e iteradores seguros secuenciales implementados/documentados incrementalmente; la integración con World y las queries públicas aún requieren sus propios puntos y tests de consistencia.
4. **098–101 — Integración Runtime/Compute, sistemas paralelos y consistencia:** adaptador síncrono explícito de `World` a Runtime e integración secuencial interna del storage con `compute::for_each` implementados; el punto 100 tiene contrato de seguridad, `SystemAccess`, detección básica de conflictos y `SystemPlan` determinista implementados; sigue en progreso hasta disponer de ejecución segura que valide declaraciones, aísle cambios estructurales y pase CI multiplataforma. El punto 101 añade pruebas de invariantes de slots/lista libre y del índice sparse/columnas bajo mutaciones repetidas.
5. **102–104 — Benchmarks, acceso a memoria y documentación:** medir antes de cambiar representación o prometer rendimiento.

### Benchmark baseline (punto 102, completado)

`benches/ecs.rs` mide la API pública disponible hoy: creación de entidades, destrucción, reutilización de slots y recorrido de `contains`, con tamaños de 64, 1.024 y 16.384 entidades y siete muestras por caso. La metodología y los límites están documentados en `docs/ecs-benchmarks.md`. La primera ejecución de CI terminó correctamente en las cinco plataformas y sus resultados están resumidos en `docs/ecs-benchmarks.md`. No mide componentes ni queries porque el almacenamiento todavía no está integrado en `World`; el punto se limita expresamente a la línea base del ciclo de vida público.
