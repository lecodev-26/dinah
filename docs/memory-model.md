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

## Layout, alineación y concurrencia

- Usar el layout y la alineación garantizados por los tipos y contenedores estándar. No inferir layout estable de tipos `repr(Rust)` ni serializar su representación en memoria.
- Estructuras contiguas y layouts orientados a datos se elegirán por acceso y carga de trabajo medidos, no por preferencia estética.
- Layouts SIMD-friendly deben respetar alineación y requisitos del hardware sin asumir que todo vector está alineado a una frontera especial.
- Los recursos compartidos deben cumplir los contratos `Send`/`Sync` aplicables y documentar quién sincroniza cada mutación. Un pool no es seguro para compartir por el mero hecho de estar envuelto en un puntero compartido.
- Evitar bloqueos globales como valor por defecto. Comparar alternativas con benchmarks y pruebas de concurrencia antes de optimizar.

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

## Contratos que deben resolver los siguientes puntos

| Punto | Decisión pendiente |
|---|---|
| 062 | Completado: primer buffer contiguo y acotado; quedan otros tipos de buffer sujetos a necesidades futuras |
| 063 | Completado: propiedad única de `Buffer<T>`, préstamos con lifetimes del compilador, transferencia por movimiento, sin clonación implícita ni acceso mutable al contenedor interno |
| 064 | Completado: usar contenedores estándar y reservas fallibles; aplazar un trait propio hasta disponer de un segundo backend real |
| 065 | Completado: contrato de pool acotado, agotamiento explícito, leases exclusivos y sincronización local por defecto |
| 066 | Reglas de reinicio, limpieza y reutilización |
| 067 | Requisitos concretos de alineación |
| 068 | Casos SIMD y layouts respaldados por mediciones |
| 069–070 | Representaciones contiguas y data-oriented para cargas identificadas |
| 071–073 | Sincronización, invariantes y pruebas de seguridad/concurrencia |
| 074–075 | Baselines de asignaciones y optimizaciones demostrables |
| 076 | Documentación integrada y criterios de salida del subsistema |

## Criterios para aceptar una implementación

- API documentada con propiedad, lifetimes, límites, errores y garantías de concurrencia claros.
- Pruebas de límites, buffers vacíos, fallos, reutilización, destrucción y accesos compartidos.
- Sin `unsafe` en el alcance inicial salvo una propuesta separada, motivada, revisada y probada.
- CI remota verde para formato, tests, Clippy y documentación.
- Cualquier afirmación de mejora de rendimiento respaldada por un benchmark repetible y una comparación antes/después.

## Fuera de alcance por ahora

No se implementan aquí allocators personalizados, arenas, pools, SIMD, FFI ni garantías de tiempo real. El `Buffer<T>` inicial es una primitiva mínima experimental, no un subsistema Memory completo. Los puntos siguientes deben justificar y probar cada abstracción antes de incorporarla.
