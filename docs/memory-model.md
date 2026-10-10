# dinah Memory — modelo de diseño inicial

Estado: diseño inicial; el punto 062 implementa el primer buffer acotado. El subsistema sigue siendo experimental y su API no es una garantía de estabilidad.

## Objetivos

- Hacer explícitos el propietario de cada recurso, su duración y el coste de reservar capacidad.
- Mantener la seguridad de memoria de Rust sin `unsafe` en la implementación inicial.
- Permitir almacenamiento contiguo y reutilización cuando una carga real demuestre que son útiles.
- Hacer que límites, fallos de reserva y sincronización sean visibles para quien llama.
- Evitar promesas de rendimiento sin benchmarks reproducibles.

## Modelo de ownership y ciclo de vida

1. **Propiedad única por defecto.** Los valores pertenecen a una estructura propietaria (`Vec<T>`, `Box<T>` u otro contenedor estándar adecuado). La destrucción sigue el ciclo de vida de Rust y libera los recursos propiedad del contenedor.
2. **Préstamos para acceso temporal.** Las vistas usan referencias y slices con lifetimes explícitos; no retienen referencias más allá de lo permitido por el compilador. Una vista no adquiere ni transfiere la propiedad del almacenamiento.
3. **Transferencia explícita.** Mover un valor transfiere la propiedad sin duplicar por sí mismo el buffer. Las copias de elementos o clones de almacenamiento deben ser operaciones explícitas y documentadas.
4. **Compartición excepcional y visible.** Cuando varios consumidores deban compartir datos, preferir primero préstamos; usar `Arc<T>` para propiedad compartida y `Mutex<T>`/`RwLock<T>` solo si hace falta mutabilidad sincronizada. La sincronización no se añade implícitamente.
5. **Sin referencias colgantes ni liberación manual.** No exponer punteros crudos ni permitir que un pool recicle almacenamiento mientras existan vistas válidas. La implementación inicial no usa `unsafe` ni libera memoria manualmente.

## Reservas, capacidad y errores

- Los contenedores estándar de Rust son la estrategia inicial; no crear un allocator propio por anticipado.
- Las APIs que acepten límites deben distinguir capacidad máxima de longitud actual y documentar qué pasa al alcanzar el límite.
- Las estructuras acotadas deben preferir un error recuperable o devolver el elemento no insertado frente a truncar, sobrescribir o crecer silenciosamente.
- No prometer que una capacidad solicitada puede reservarse físicamente. Si una API necesita informar fallos de reserva, debe usar una operación fallible cuando esté disponible y preservar el error concreto.
- Una reserva inicial puede reducir realocaciones, pero no equivale a una garantía de ausencia de futuras reservas salvo que el contrato y las operaciones lo aseguren.
- Los tamaños derivados de entradas externas deben validarse antes de reservar y evitar cálculos de capacidad que desborden.

## Abstracción de asignación (punto 064)

**Decisión: no introducir todavía un trait `Allocator` propio ni un allocator global.** El único almacenamiento implementado utiliza `Vec<T>` de la biblioteca estándar y `try_reserve_exact`, que ya permite informar fallos de reserva sin recurrir a una ruta de asignación deliberadamente infalible. Una interfaz propia ahora duplicaría capacidades sin tener un segundo backend que demuestre qué contrato común necesita DINAH.

Contrato inicial:

- Los tipos de Memory solicitan y poseen almacenamiento mediante contenedores estándar seguros; no administran direcciones ni liberación manual.
- Las operaciones de construcción que necesitan reservar capacidad usan rutas fallibles cuando la biblioteca las ofrece. El error público describe el fallo de la operación, sin exponer detalles internos del allocator del sistema.
- La capacidad lógica solicitada pertenece al contrato del contenedor; no se interpreta como un presupuesto exacto de bytes ni como garantía de memoria residente.
- Las abstracciones futuras deberán separar, si la evidencia lo requiere, el mecanismo de reserva de la política del contenedor (límite, crecimiento y recuperación). No se debe añadir un trait que pretenda soportar allocators incompatibles sin un caso de uso y pruebas concretos.
- Reconsiderar un trait cuando exista al menos un segundo backend real (por ejemplo, un pool explícito) que comparta operaciones y semántica de error con el almacenamiento estándar. Evaluar entonces fallos, alineación, ownership, concurrencia, compatibilidad `no_std` si aplica y coste de abstracción.

`BufferCreateError::AllocationFailed` es la frontera pública actual: evita filtrar el error concreto del allocator y mantiene estable el contrato del buffer. Esta decisión no impide introducir una abstracción más rica en un punto posterior; evita comprometer prematuramente la API.

## Buffers, pools y reutilización

- Los buffers propios serán dueños de sus elementos; las vistas serán prestadas y no poseerán la memoria.
- Pools y arenas son optimizaciones opt-in, acotadas por una política explícita de capacidad y recuperación. No se introducen como asignador global ni como comportamiento oculto.
- Una API de pool debe definir quién adquiere y devuelve cada recurso, cuándo puede reutilizarse, qué sucede al agotarse y si el pool es local a un hilo o compartido.
- La reutilización no puede cambiar la semántica de inicialización ni exponer valores obsoletos. Los elementos deben estar correctamente inicializados antes de leerse.
- El descarte de un pool libera sus recursos propios; los handles prestados no pueden sobrevivir al propietario del almacenamiento.

## Contrato de pools (punto 065)

**Decisión: diseñar primero un pool explícito, acotado y local a su propietario; no implementarlo todavía ni convertirlo en allocator global.** El punto 065 fija el contrato. El punto 066 resolverá por separado cómo reiniciar y limpiar cada clase de recurso antes de reutilizarla.

Contrato de diseño para una primera implementación futura:

1. **Capacidad fija y explícita.** La creación recibe un límite de recursos y falla de forma recuperable si no puede reservar/preparar lo requerido. El pool no crece automáticamente al agotarse y no promete un presupuesto exacto de bytes salvo que una API posterior lo mida explícitamente.
2. **Agotamiento visible.** Una adquisición sin recursos disponibles devuelve un error/resultado de agotamiento; no bloquea indefinidamente, no asigna un recurso ocultamente y no sobrescribe un recurso prestado.
3. **Préstamo exclusivo con retorno verificable.** La primera API preferirá un lease/handle RAII que represente la adquisición exclusiva y devuelva el recurso al pool al finalizar su vida. El tipo de handle debe impedir que el pool se destruya o que el mismo slot se adquiera dos veces mientras el lease siga vivo. No se devolverán índices o referencias que permitan reutilización sin controlar el ciclo de vida.
4. **Propiedad y fallos claros.** El pool es dueño de sus slots y recursos; el usuario posee temporalmente el lease, no el slot subyacente. Si la construcción del pool falla, no se publica una instancia parcial. Si no puede satisfacerse una adquisición, el estado del pool permanece válido y el error es observable.
5. **Sin crecimiento ni recuperación implícitos.** La capacidad y la política de agotamiento forman parte del contrato. La liberación de un lease permite volver a adquirir el recurso, pero la política de reset/limpieza queda definida en el punto 066 y no se presume que todos los tipos puedan reutilizarse de la misma forma.
6. **Sincronización explícita.** La primera implementación será local y no sincronizada: no promete `Send`/`Sync` adicionales ni acceso concurrente automático. El uso compartido y su estrategia de sincronización se deciden en los puntos 071 y 073; no habrá mutex global implícito.
7. **Seguridad de memoria.** Mantener la implementación inicial en Rust seguro, sin punteros crudos ni `unsafe`. No se podrá leer un slot sin inicializar ni reciclarlo mientras exista un lease válido.
8. **Sin compromiso prematuro de backend.** El pool no obliga a introducir un trait `Allocator`. Solo se extraerá una abstracción común cuando otro backend real demuestre operaciones y semánticas compartidas.

**Criterio para implementar más adelante:** pruebas de pool vacío y lleno, agotamiento, adquisición/retorno, destrucción con recursos disponibles, exclusividad de leases, fallo de construcción y ausencia de acceso a recursos no inicializados. Los detalles de limpieza/reset, las pruebas de concurrencia y los benchmarks corresponden a sus puntos específicos; no se consideran completados por esta decisión de diseño.

## Contrato de reutilización de memoria (punto 066)

**Decisión: reutilizar almacenamiento no significa conservar vivos los valores anteriores.** La estrategia por defecto será destruir los valores que ya no pertenecen al usuario y conservar solo la reserva de almacenamiento cuando el contenedor lo permita. No se añadirá un trait genérico `Reset` ni se supondrá que cualquier tipo puede volver a un estado inicial de forma segura.

Reglas para las APIs presentes y futuras:

1. **Separar almacenamiento y valor.** La capacidad reservada puede reutilizarse aunque los valores anteriores se hayan destruido. La vida útil de cada valor sigue las reglas de Rust; reservar memoria no equivale a inicializar objetos en ella.
2. **`Buffer::clear` destruye, no resetea.** Destruye exactamente los elementos actualmente contenidos y conserva la reserva del buffer. Un `Drop` de elemento puede ejecutar efectos observables. Una inserción posterior debe aportar un valor nuevo y correctamente inicializado.
3. **`pop` transfiere propiedad.** El elemento extraído pasa al llamador y deja de pertenecer al buffer; el buffer puede reutilizar la capacidad liberada para futuras inserciones. El llamador es responsable de la vida útil del valor extraído.
4. **No exponer contenido obsoleto.** Las vistas solo cubren los elementos inicializados actualmente. Después de limpiar o extraer elementos, ninguna API podrá leerlos mediante el buffer ni asumir que la memoria reservada contiene valores válidos. No se expondrá memoria sin inicializar ni se usarán bytes antiguos como valores nuevos.
5. **Reset explícito y específico.** Si una estructura futura necesita conservar un objeto vivo y reiniciarlo, deberá definir una operación explícita cuyo contrato describa qué campos se restablecen, qué recursos se liberan, cómo se manejan errores y qué invariantes quedan garantizadas. No se aplicará automáticamente a tipos arbitrarios.
6. **Reutilización de slots de pool.** Un lease devuelto marca el slot como disponible, pero no autoriza por sí solo a reutilizar el estado anterior. Antes de entregarlo al siguiente usuario, el pool deberá destruir y reconstruir el valor o ejecutar un reset específico, documentado y probado. La política concreta se elige por tipo y no debe permitir dos leases simultáneos para el mismo slot.
7. **Fallos sin estado corrupto.** Si reinicializar un recurso puede fallar, la API debe representar ese fallo y mantener el slot fuera de circulación hasta que vuelva a un estado válido o se retire de forma segura. Nunca debe marcarse como disponible un recurso parcialmente reinicializado.
8. **Sin garantías de puesta a cero.** Destruir un valor no garantiza borrar físicamente sus bytes. Los datos sensibles requieren una política de borrado seguro específica; no se afirmará que `clear` o `Drop` constituyen borrado criptográfico.
9. **Medir antes de sofisticar.** La reutilización debe preservar la semántica observable y se optimizará solo cuando las mediciones indiquen que evitar construcciones/asignaciones compensa la complejidad añadida.

**Pruebas requeridas al implementar:** contar destrucciones con tipos instrumentados; comprobar que `clear` mantiene la capacidad y permite insertar valores nuevos; verificar que `pop` transfiere exactamente un valor; probar errores de reset si se introduce; y demostrar que no se puede observar contenido de elementos destruidos o no inicializados. El pool concreto, la sincronización y los benchmarks siguen siendo tareas separadas.

## Contrato de alineación (punto 067)

**Decisión: la alineación debe venir garantizada por el tipo y por la API de almacenamiento utilizada; DINAH no prometerá alineación especial sin representarla explícitamente en el contrato.** La implementación inicial permanece en Rust seguro y no calculará direcciones ni desplazamientos manualmente.

Reglas:

1. **Alineación ordinaria.** Para `Buffer<T>` y los contenedores estándar, confiar en las garantías de alineación de `T` y del contenedor de Rust. El consumidor no necesita ni puede asumir una alineación adicional distinta de la documentada por el tipo.
2. **Sin promesas basadas en el layout accidental.** No inferir offsets, padding ni layout estable a partir de `repr(Rust)`. Si un contrato binario requiere una representación concreta, deberá elegir y documentar una representación admitida, como `repr(C)`, sin confundir layout de campos con alineación especial de una reserva.
3. **Alineación mayor que la ordinaria.** Si un caso real exige alineación explícita (por ejemplo, una interfaz de hardware), modelarla en un tipo o API de asignación cuya garantía sea verificable. La construcción debe fallar de forma recuperable cuando no pueda cumplirse el contrato; nunca devolver almacenamiento con alineación insuficiente.
4. **No falsear alineación SIMD.** Que un buffer sea contiguo no significa que empiece en una frontera SIMD específica. El punto 068 evaluará por separado requisitos concretos de instrucciones, tipos y plataformas; no añadir padding ni sobreasignación especulativa sin mediciones.
5. **Sin aritmética manual de punteros.** No exponer punteros crudos ni usar `unsafe` para ajustar direcciones en esta fase. Si las APIs seguras disponibles no pueden satisfacer un requisito demostrado, documentar la limitación y proponer un diseño separado antes de ampliar el alcance.
6. **Casos límite.** Cualquier implementación futura debe definir el comportamiento para capacidad cero y tipos de tamaño cero (ZST), además de tamaños/alineaciones elevados y fallos de reserva. No deducir alineación útil de una dirección cuando no exista almacenamiento de elementos.
7. **Alineación no es rendimiento garantizado.** Una alineación más estricta puede aumentar el coste o el desperdicio de memoria; solo se adoptará si benchmarks representativos demuestran una mejora sin romper el contrato de seguridad.

**Pruebas requeridas al implementar alineación especial:** comprobar con `align_of`/`align_of_val` las garantías del tipo; probar tipos ordinarios y sobrealineados en las APIs que los admitan; cubrir capacidad cero y ZST; comprobar que un fallo de construcción no publica una instancia parcial; y verificar que la API no afirma garantías más fuertes que las que realmente ofrece. Las pruebas SIMD y los benchmarks comparativos permanecen en los puntos 068 y 074–075.

## Investigación de layouts compatibles con SIMD (punto 068)

**Resultado de la investigación: no existe un layout universalmente óptimo.** La elección debe seguir el patrón de acceso y los datos que realmente procesa una operación. Este punto fija criterios de diseño; no añade instrucciones SIMD, dependencias, tipos vectoriales ni garantías de alineación nuevas.

### Alternativas de layout

- **Array of Structures (AoS).** Cada elemento contiene todos sus campos (por ejemplo, `struct Particle { x, y, z, mass }`). Es adecuado cuando se consume la mayoría de los campos de cada elemento. Si una operación recorre solo un campo entre muchos, los datos relevantes quedan intercalados y puede reducirse la eficiencia de caché/vectorización.
- **Structure of Arrays (SoA).** Cada campo ocupa su propio array (`x[]`, `y[]`, `z[]`, `mass[]`). Favorece recorridos que aplican la misma operación a muchos valores de un campo y puede facilitar el acceso contiguo y la vectorización automática. Puede ser menos cómodo cuando casi siempre se necesita reunir todos los campos de un único elemento.
- **Array of Structures of Arrays (AoSoA).** Agrupa bloques de varios elementos y almacena cada campo en arrays dentro de cada bloque. Puede equilibrar procesamiento por lotes y agrupación por entidad, pero añade complejidad, padding potencial y una elección de tamaño de bloque dependiente de la carga y del hardware. No será la opción inicial sin evidencia.
- **Slices contiguos de un solo tipo.** Para operaciones simples sobre un campo (`&[f32]`, por ejemplo), empezar por slices contiguos y bucles ordinarios seguros. Permitir que el compilador optimice antes de introducir intrinsics o una API vectorial específica.

### Decisiones para DINAH

1. **Primero, semántica y patrón de acceso.** Documentar si una operación consume entidades completas, campos individuales o bloques, y si inserta/elimina elementos con frecuencia. Elegir AoS, SoA o AoSoA en función de ese patrón.
2. **SoA es un candidato, no una regla global.** Para cargas numéricas que recorren el mismo campo de muchas entidades, evaluar SoA primero; conservar AoS cuando las operaciones usan casi todos los campos de cada entidad o el diseño simplifica ownership y coherencia.
3. **Contigüidad no implica vectorización.** Un slice contiguo ayuda al acceso secuencial, pero la vectorización depende también de aliasing, dependencias entre iteraciones, tipos, opciones de compilación y arquitectura. No prometer que un bucle será vectorizado por su forma o por su alineación.
4. **No exigir alineación SIMD especial por defecto.** El diseño base respeta la alineación normal del tipo. Solo considerar alineación/padding adicional si una API o instrucción elegida lo exige y una medición lo justifica. No asumir un ancho fijo de vector ni un tamaño de bloque universal.
5. **Mantener una implementación escalar de referencia.** La corrección no debe depender de que SIMD esté disponible. Si más adelante se añade una ruta vectorial, comparar sus resultados con la ruta escalar, incluidos tamaños no múltiplos del ancho de lote y el procesamiento del resto (tail).
6. **No introducir todavía una API SIMD pública.** Primero identificar una operación real y sus datos. Evaluar las capacidades estables del toolchain objetivo cuando se diseñe la implementación; cualquier intrinsics, feature detection o ruta específica de CPU queda fuera de este punto.
7. **Medir de forma reproducible.** Comparar layouts con los mismos datos, algoritmo, configuración de compilación y condiciones de ejecución. Separar tiempo de conversión/reorganización de datos del tiempo del kernel; incluir tamaños pequeños y grandes. Registrar CPU, versión de Rust, flags, número de iteraciones y variabilidad. Los benchmarks de Memory y la optimización permanecen en los puntos 074–075.

### Matriz de decisión inicial

| Patrón de trabajo | Candidato inicial | Riesgo que comprobar |
|---|---|---|
| Operaciones que consumen la mayoría de campos de cada entidad | AoS | Campos no usados y stride entre valores equivalentes |
| Mismo cálculo sobre un campo de muchas entidades | SoA | Coste de reconstruir una entidad completa y coherencia entre arrays |
| Procesamiento por lotes que necesita agrupación de entidades y campos | AoSoA experimental | Tamaño de bloque, padding y complejidad de mantenimiento |
| Transformación numérica sobre una secuencia homogénea | Slice contiguo y bucle escalar idiomático | Si el compilador vectoriza y cómo procesa el resto |

**Criterio para avanzar a implementación:** elegir una operación real, establecer una versión escalar correcta, crear datos representativos y medir al menos dos layouts comparables. Solo añadir complejidad SIMD/alineación cuando los resultados repetibles demuestren una mejora útil y se mantenga una ruta correcta para los casos límite.

## Layout, alineación y concurrencia

- Usar el layout y la alineación garantizados por los tipos y contenedores estándar. No inferir layout estable de tipos `repr(Rust)` ni serializar su representación en memoria.
- Estructuras contiguas y layouts orientados a datos se elegirán por acceso y carga de trabajo medidos, no por preferencia estética.
- Layouts SIMD-friendly deben respetar alineación y requisitos del hardware sin asumir que todo vector está alineado a una frontera especial.
- Los recursos compartidos deben cumplir los contratos `Send`/`Sync` aplicables y documentar quién sincroniza cada mutación. Un pool no es seguro para compartir por el mero hecho de estar envuelto en un puntero compartido.
- Evitar bloqueos globales como valor por defecto. Comparar alternativas con benchmarks y pruebas de concurrencia antes de optimizar.

## Pruebas de seguridad de memoria (punto 072)

Las pruebas de Memory deben verificar el contrato observable sin usar `unsafe` ni depender de detalles internos del allocator:

- **Propiedad y destrucción:** los valores extraídos por `pop` pasan al llamador; `clear` destruye los elementos restantes una sola vez y permite reutilizar la reserva; la destrucción del contenedor destruye todos los valores que aún posee. `SoaBuffer` aplica estas reglas a ambas columnas y conserva la alineación lógica de filas.
- **Límites:** capacidad cero, capacidad llena, inserciones rechazadas y extensiones que no caben no deben producir longitudes superiores a la capacidad ni perder valores aceptados/rechazados.
- **Tipos especiales:** los tipos de tamaño cero siguen sujetos a la capacidad lógica; los tipos sobrealineados conservan la alineación garantizada por Rust. No se infiere una alineación mayor que la del tipo.
- **Accesos:** las vistas contienen únicamente elementos inicializados y los préstamos de Rust impiden mutaciones incompatibles mientras una vista siga viva. El ejemplo `compile_fail` de `Buffer` valida esta restricción en la documentación.
- **Implementación:** `#![forbid(unsafe_code)]` en la raíz del crate impide introducir bloques `unsafe` en el código del proyecto. Estas pruebas refuerzan el contrato, pero no sustituyen Miri, fuzzing o una auditoría cuando se incorporen abstracciones más complejas.

## Sincronización explícita de Memory (punto 071)

Los tipos de `dinah::memory` siguen siendo propietarios y no incorporan bloqueos implícitos. Cuando varios hilos necesiten mutar la misma estructura, se puede envolver en `dinah::runtime::Shared<T>`, la primitiva existente basada en `Arc<Mutex<T>>`. Clonar `Shared<T>` comparte el mismo valor; no copia el buffer ni sus elementos.

- `Shared::lock()` devuelve un guard de mutex y comunica el envenenamiento mediante `SynchronizationError::Poisoned`; el llamador debe gestionar ese error de forma explícita. No se recupera silenciosamente un estado potencialmente inconsistente.
- Mantener los guards lo más breves posible. No ejecutar callbacks externos ni esperar trabajo asíncrono mientras se mantiene un bloqueo; definir un orden consistente si se necesitan varios locks para evitar interbloqueos.
- Las operaciones de una estructura compuesta —por ejemplo, insertar una fila en ambas columnas de `SoaBuffer`— deben ejecutarse bajo el mismo guard para conservar invariantes. No repartir las columnas entre locks independientes.
- El wrapper no hace que cualquier `T` sea seguro para cualquier uso: las restricciones `Send`/`Sync` siguen dependiendo de los tipos de Rust y de la API utilizada. El wrapper no sustituye la validación de invariantes ni las pruebas de concurrencia.
- La sincronización es opt-in. Para datos inmutables, preferir compartir referencias/valores inmutables; para acceso local a un solo hilo, usar directamente el buffer y evitar el coste del mutex. No se afirma que `Mutex` sea la estrategia más rápida para todas las cargas.

Las pruebas de concurrencia comprueban también que 16 hilos que empiezan mediante una barrera no pierden ninguna de sus 32.000 actualizaciones, y que un segundo hilo no puede adquirir el mutex mientras otro conserva el guard: tras liberar el guard, la adquisición termina. El test usa un timeout solo para detectar que el bloqueo sigue retenido y no presupone un orden de planificación entre hilos.

La prueba `shared_soa_serializes_concurrent_row_insertions` comprueba que varios hilos pueden insertar filas completas mediante un `Shared<SoaBuffer<_, _>>` sin desalinear las columnas. Estas pruebas validan la exclusión mutua y las invariantes observables, pero no demuestran ausencia de interbloqueos en cualquier programa, fairness, ausencia de starvation ni rendimiento escalable. Los benchmarks completos siguen perteneciendo a los puntos 074–075.

## Buffer inicial (punto 062)

`dinah::memory::Buffer<T>` es un contenedor propietario y contiguo de capacidad lógica fija. `try_with_capacity` reserva de forma fallible la capacidad máxima antes de devolver el buffer; por tanto, una reserva fallida no deja una instancia parcialmente inicializada y los `try_push` posteriores no necesitan aumentar la reserva mientras se respete el límite. La reserva real del allocator puede ser mayor que la solicitada.

- La capacidad es el máximo de elementos, no un límite de bytes; `T` puede tener tamaños distintos.
- Capacidad cero es válida. `is_full()` es verdadero cuando longitud y capacidad son cero.
- `try_push` devuelve el elemento original en `BufferPushError::Full(T)` si está lleno; nunca sobrescribe ni descarta silenciosamente datos.
- `as_slice` presta únicamente los elementos inicializados; la propiedad permanece en el buffer y el compilador impide mutaciones incompatibles mientras la vista siga viva.
- `Buffer<T>` no implementa `Clone`: evita una duplicación implícita de almacenamiento cuya reserva podría no cubrir el límite lógico. La propiedad se transfiere mediante movimientos; duplicar elementos requiere una decisión explícita del consumidor.
- `pop` transfiere al llamador la propiedad del último elemento y `clear` destruye los elementos actuales conservando la reserva. Al destruir el buffer se destruyen todos los elementos que aún posee.
- No se expone el `Vec<T>` interno: el consumidor no puede eludir el límite lógico mediante una mutación externa del almacenamiento.
- No hay sincronización interna: el acceso mutable requiere `&mut self`.
- La API no promete un límite de memoria en bytes ni que la reserva física permanezca residente.

La implementación y sus pruebas viven en `src/memory/buffer.rs`. No se expone acceso mutable al almacenamiento interno ni una API de crecimiento dinámico en este punto.

## Estructuras contiguas iniciales (punto 069)

La primera estructura contigua implementada es `Buffer<T>`: posee una secuencia contigua de elementos inicializados con una capacidad lógica máxima fija. El punto 069 amplía su utilidad para cargas por lotes sin duplicar el almacenamiento ni exponer el `Vec<T>` interno:

- `as_slice()` presta acceso de solo lectura a los elementos inicializados; `as_mut_slice()` permite modificar esos elementos in situ, pero no cambiar la longitud, la capacidad ni la asignación.
- `try_extend_from_slice()` permite añadir una secuencia de elementos `Clone` en una sola operación lógica. Comprueba la capacidad restante antes de modificar el buffer; si no caben todos, devuelve `BufferExtendError { requested, remaining }` y conserva el contenido original.
- La capacidad sigue siendo un máximo de elementos, no un presupuesto de bytes. La reserva del constructor y el límite lógico evitan crecimiento por encima del límite en operaciones exitosas, pero no prometen tamaño físico exacto ni residencia de memoria.
- La API permanece segura, sin punteros crudos ni `unsafe`, y no añade sincronización implícita. Los préstamos siguen gobernando cuándo puede mutarse el buffer.
- `try_extend_from_slice` necesita `T: Clone`; la clonación de elementos puede ejecutar código del usuario y entrar en pánico. La garantía de no modificación aplica al error normal de capacidad, no a una garantía transaccional frente a pánicos de `Clone`.

No se crea una segunda envoltura sobre `Vec<T>` ni una abstracción genérica adicional: primero se aprovecha y refuerza la estructura existente. Una matriz contigua, SoA u otra representación se añadirá cuando el punto 070 o una carga concreta defina dimensiones, ownership y patrón de acceso que justifiquen su API.

## Estructura data-oriented inicial (punto 070)

`SoaBuffer<A, B>` implementa una estructura de dos columnas (Structure of Arrays, SoA) encima de los `Buffer<T>` acotados existentes. Cada columna mantiene sus elementos contiguos; una fila lógica es el par de valores en el mismo índice. Ambas columnas comparten capacidad y longitud por construcción, y el tipo no expone acceso mutable a los buffers internos.

- `try_with_capacity` reserva ambas columnas mediante operaciones fallibles. Si falla cualquiera, no devuelve una estructura parcial y Rust libera la reserva ya creada.
- `try_push` añade ambos valores como una fila lógica o devuelve ambos valores sin insertarlos cuando se alcanza la capacidad. Las capacidades iguales y el control privado de las columnas mantienen alineadas sus longitudes.
- `as_slices` y `as_mut_slices` permiten procesar un campo a la vez sin reconstruir filas ni modificar la longitud. `pop` y `clear` operan sobre ambas columnas para preservar la correspondencia por índice.
- `try_extend_from_slices` añade dos slices como un lote: valida que tengan la misma longitud y que todo el lote quepa antes de mutar cualquiera de las columnas. Reutiliza las reservas y evita ejecutar la comprobación de capacidad por cada fila. Requiere `Clone` para ambos tipos y no promete rollback si una clonación entra en pánico.
- Capacidad cero es válida; la estructura no crece automáticamente, no sincroniza internamente y no usa `unsafe`. La API no promete alineación SIMD especial ni una mejora de rendimiento sin benchmarks.
- SoA conviene cuando se procesa repetidamente un campo de muchas filas. Para cargas que consumen casi todos los campos de una fila, AoS puede ser más simple o eficiente; AoSoA queda aplazado hasta tener una carga y mediciones que lo justifiquen.

La implementación y las pruebas viven en `src/memory/soa.rs`. Es una primitiva genérica limitada a dos columnas, no una API ECS ni una afirmación de layout universalmente óptimo.

## Contratos que deben resolver los siguientes puntos

| Punto | Decisión pendiente |
|---|---|
| 062 | Completado: primer buffer contiguo y acotado; quedan otros tipos de buffer sujetos a necesidades futuras |
| 063 | Completado: propiedad única de `Buffer<T>`, préstamos con lifetimes del compilador, transferencia por movimiento, sin clonación implícita ni acceso mutable al contenedor interno |
| 064 | Completado: usar contenedores estándar y reservas fallibles; aplazar un trait propio hasta disponer de un segundo backend real |
| 065 | Completado: contrato de pool acotado, agotamiento explícito, leases exclusivos y sincronización local por defecto |
| 066 | Completado: separar reserva de almacenamiento y vida de valores; `clear` destruye, `pop` transfiere propiedad y reset debe ser explícito |
| 067 | Completado: usar garantías de alineación del tipo/contenedor; alineación especial solo con contrato explícito, errores recuperables y pruebas |
| 068 | Completado: documentados AoS, SoA, AoSoA y slices contiguos; implementación SIMD y benchmarks quedan separados |
| 069 | Completado: `Buffer<T>` contiguo con acceso por slices y extensión por lotes acotada; sin crecimiento implícito |
| 070 | Completado: `SoaBuffer<A, B>` con dos columnas contiguas, filas coordinadas y límites explícitos |
| 071–073 | Sincronización, invariantes y pruebas de seguridad/concurrencia |
| 074 | Completado: benchmark reproducible `benches/memory.rs` para push preasignado, extensión por lotes, inserción/escaneo SoA y coste de `Shared` con mediana de 7 muestras; resultados de cada ejecución se guardan como artifact de CI |
| 075 | Añadido `SoaBuffer::try_extend_from_slices` para validar una vez e insertar lotes en las columnas reservadas; benchmark `soa_bulk_extend` añadido. La mejora debe confirmarse en CI y archivarse antes de cerrar el punto |
| 076 | Documentación integrada y criterios de salida del subsistema |

## Criterios para aceptar una implementación

- API documentada con propiedad, lifetimes, límites, errores y garantías de concurrencia claros.
- Pruebas de límites, buffers vacíos, fallos, reutilización, destrucción y accesos compartidos.
- Sin `unsafe` en el alcance inicial salvo una propuesta separada, motivada, revisada y probada.
- CI remota verde para formato, tests, Clippy y documentación.
- Cualquier afirmación de mejora de rendimiento respaldada por un benchmark repetible y una comparación antes/después.

## Fuera de alcance por ahora

No se implementan aquí allocators personalizados, arenas, pools, SIMD, FFI ni garantías de tiempo real. El `Buffer<T>` inicial es una primitiva mínima experimental, no un subsistema Memory completo. Los puntos siguientes deben justificar y probar cada abstracción antes de incorporarla.
