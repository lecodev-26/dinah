# dinah ECS — diseño inicial

Estado: diseño inicial aprobado; `dinah::ecs::Entity` y el contrato `dinah::ecs::Component` implementados. Storage, `World` y queries siguen pendientes. Las decisiones de este documento son una propuesta inicial revisable, no una garantía de estabilidad pública.

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


### Contrato implementado: `Component` (punto 093)

`dinah::ecs::Component` es un trait marcador público con `Component for T` para todo `T: 'static`. No aporta métodos ni exige derive/manual boilerplate; hace explícito el bound conceptual en las futuras APIs de storage y `World`. La implementación blanket es intencional: el contrato inicial es la duración de vida requerida por `Any`/`TypeId`, no la capacidad de enviar o compartir el valor entre hilos. Tests unitarios y de integración comprueban tipos comunes y `Rc<Cell<u32>>`, que no es `Send` ni `Sync`.

### Primer contrato implementado: `Entity` (punto 092)

`dinah::ecs::Entity` es un handle `Copy` con dos campos privados `u32`: índice y generación. Expone `index()` y `generation()` como accesores de solo lectura y una representación diagnóstica `indexvgen` mediante `Display`. Su constructor es privado para usuarios externos; el `World` será responsable de emitir handles válidos. La implementación no asigna IDs, no prueba por sí sola que un handle esté vivo y no promete unicidad fuera del mundo propietario.

Los tests verifican que igualdad y hashing distinguen generaciones distintas del mismo índice, que handles equivalentes se comparan igual, que los valores máximos se conservan y que el handle ocupa ocho bytes en la plataforma objetivo de Rust (dos `u32`). La validación de vida útil y reutilización queda pendiente de `World`, que aún no está implementado.

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

## Queries e iteración

La primera query se limitará a un conjunto pequeño de componentes tipados y existentes. El diseño preferido es iterar sobre el storage más pequeño disponible y comprobar la presencia de los demás, pero esta optimización solo se implementará cuando los tipos de préstamo y el contrato de mutabilidad estén claros.

- Iteración de lectura puede devolver referencias compartidas mientras el préstamo del `World` lo permita.
- Iteración mutable debe impedir aliasing de un mismo componente y no permitir operaciones estructurales sobre el `World` durante el préstamo.
- Las queries no prometen orden estable tras inserciones/eliminaciones; el orden será el del storage conductor y podrá cambiar por `swap_remove`.
- Si una query no puede expresar con seguridad combinaciones de préstamos disjuntos en Rust estable, se empieza por queries de lectura o una API más limitada; no se usará `unsafe` como atajo.

## Límites de seguridad y errores

El crate mantiene `#![forbid(unsafe_code)]`. La implementación preferirá `Vec`, `HashMap<TypeId, Box<dyn Any>>` y downcasts seguros, con las restricciones de `borrow checker` visibles en la API. Las operaciones que puedan fallar por una entrada inválida deben devolver errores tipados. Un fallo de reserva de memoria no debe dejar un `World` con entidades o columnas desincronizadas; las operaciones que necesitan varias mutaciones deben validar y reservar antes de comprometer cambios cuando sea posible.

La destrucción de una entidad debe dejar todos los storages coherentes. Las pruebas cubrirán componentes en varios storages, filas movidas por `swap_remove`, reutilización de índice y rechazo de handles obsoletos. Los pánicos de destructores siguen la semántica de Rust y no se promete recuperación transaccional frente a un pánico.

## Integración futura

- **Memory (096):** reutilizar las lecciones de `Buffer<T>` y `SoaBuffer<A, B>`, pero no forzar el buffer fijo actual a ser un storage de tamaño dinámico sin evidencia de que sus contratos encajen.
- **Compute (097, 099):** primero iteración segura y secuencial; luego ofrecer slices o particiones disjuntas cuando el layout y los préstamos lo permitan.
- **Runtime (098, 100):** los sistemas se ejecutarán explícitamente; no se introducirá scheduling oculto en operaciones de `World`. Dependencias de lectura/escritura entre sistemas se deben modelar antes de permitir ejecución concurrente.
- **Consistencia (101):** invariantes para generaciones, lista libre, alineación de columnas e índices dispersos, y ausencia de componentes en entidades muertas.
- **Benchmarks (102–103):** comparar sparse sets con un baseline simple, registrar tamaño de entidad/componentes, composición, lecturas/escrituras, inserciones/borrados y distribución de queries. No afirmar superioridad universal ni adoptar archetypes solo por intuición.

## Secuencia de aceptación

1. **091 — Diseño ECS:** decisiones, invariantes, límites y secuencia de integración documentados aquí.
2. **092–094 — Entity, Component, World:** Entity y el bound público Component ya están definidos; fijar las firmas de World y sus errores antes de declararlos API pública.
3. **095–097 — Queries, almacenamiento e iteración:** tests de consistencia antes de optimizar.
4. **098–100 — Integración Runtime/Compute y sistemas paralelos:** solo con ownership y conflictos de acceso definidos.
5. **101–104 — Consistencia, benchmarks, acceso a memoria y documentación:** medir antes de cambiar representación o prometer rendimiento.
