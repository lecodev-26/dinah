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

## Buffers, pools y reutilización

- Los buffers propios serán dueños de sus elementos; las vistas serán prestadas y no poseerán la memoria.
- Pools y arenas son optimizaciones posteriores, opt-in y acotadas por una política explícita de capacidad y recuperación. No se introducen como asignador global ni como comportamiento oculto.
- Una API de pool debe definir claramente quién devuelve cada bloque, cuándo se puede reutilizar, qué sucede al agotarse y si el pool es local a un hilo o compartido.
- La reutilización no puede cambiar la semántica de inicialización ni exponer valores obsoletos. Los elementos deben estar correctamente inicializados antes de leerse.
- El descarte de un pool libera sus recursos propios; los handles prestados no pueden sobrevivir al propietario del almacenamiento.

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
- `as_slice` presta únicamente los elementos inicializados; la propiedad permanece en el buffer.
- `pop` elimina el último elemento y `clear` destruye los elementos actuales conservando la reserva.
- No hay sincronización interna: el acceso mutable requiere `&mut self`.
- La API no promete un límite de memoria en bytes ni que la reserva física permanezca residente.

La implementación y sus pruebas viven en `src/memory/buffer.rs`. No se expone acceso mutable al almacenamiento interno ni una API de crecimiento dinámico en este punto.

## Contratos que deben resolver los siguientes puntos

| Punto | Decisión pendiente |
|---|---|
| 062 | Completado: primer buffer contiguo y acotado; quedan otros tipos de buffer sujetos a necesidades futuras |
| 063 | Tipos propietarios y límites de ownership interno |
| 064 | Si se necesita una abstracción de asignación y cómo mantenerla fallible |
| 065 | Diseño de pools, política de agotamiento y ámbito de sincronización |
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
