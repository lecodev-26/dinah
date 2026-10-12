# dinah Memory — guía práctica

El módulo `dinah::memory` ofrece buffers contiguos con capacidad lógica fija. El subsistema sigue siendo experimental: su API no se considera estable. Esta guía describe las primitivas existentes, no capacidades planeadas.

## Elegir una estructura

| Necesidad | Opción inicial |
|---|---|
| Secuencia homogénea con límite fijo de elementos | `Buffer<T>` |
| Dos campos contiguos que se recorren por separado | `SoaBuffer<A, B>` |
| Compartir estado mutable entre hilos | Envolver explícitamente en `dinah::runtime::Shared<T>` |
| Crecimiento automático o límite estricto en bytes | No está cubierto por estos buffers |

Si casi siempre se consumen todos los campos de una entidad a la vez, un `Vec<MyStruct>` o una estructura AoS puede resultar más simple. SoA es un candidato para recorrer un campo de muchas filas, no una optimización universal. Mide con una carga representativa.

## `Buffer<T>`

`Buffer<T>` posee sus elementos y reserva capacidad mediante `try_with_capacity`. La capacidad es un máximo lógico de elementos, no un presupuesto exacto de bytes ni una garantía de memoria residente. El buffer no crece automáticamente.

```rust
use dinah::memory::{Buffer, BufferCreateError, BufferPushError};

fn main() -> Result<(), BufferCreateError> {
    let mut values = Buffer::try_with_capacity(3)?;
    values.try_push(10).expect("hay espacio");
    values.try_extend_from_slice(&[20, 30]).expect("el lote cabe");
    assert_eq!(values.as_slice(), &[10, 20, 30]);

    let rejected = values.try_push(40).expect_err("el buffer está lleno");
    assert_eq!(rejected, BufferPushError::Full(40));

    values.clear(); // destruye los elementos, conserva la reserva
    values.try_push(50).expect("la reserva se reutiliza");
    Ok(())
}
```

- `try_with_capacity(n)` devuelve `BufferCreateError::AllocationFailed` si no puede reservar la capacidad solicitada.
- `try_push(value)` inserta un elemento o devuelve `BufferPushError::Full(value)`, conservando el valor rechazado.
- `try_extend_from_slice(slice)` requiere `T: Clone`. Si el lote no cabe, devuelve `BufferExtendError { requested, remaining }` sin modificar el contenido por ese error de capacidad.
- `as_slice()` y `as_mut_slice()` prestan los elementos inicializados. La vista mutable permite cambiar valores, no longitud ni capacidad.
- `len()`, `is_empty()`, `capacity()` e `is_full()` describen el estado lógico.
- `pop()` transfiere al llamador el último elemento; `clear()` destruye todos los elementos actuales y conserva la reserva.

La capacidad cero es válida: el buffer está vacío y lleno a la vez, y no acepta inserciones. El tipo no implementa `Clone`; moverlo transfiere su propiedad. Los préstamos de Rust impiden modificar el buffer mientras siga viva una vista incompatible.

## `SoaBuffer<A, B>`

`SoaBuffer` guarda dos columnas contiguas. La fila lógica `i` corresponde al par `(first[i], second[i])`; ambas columnas mantienen la misma longitud y capacidad.

```rust
use dinah::memory::{SoaBuffer, SoaCreateError};

fn main() -> Result<(), SoaCreateError> {
    let mut entities = SoaBuffer::try_with_capacity(3)?;
    entities.try_push(100_u32, 1.5_f32).expect("hay espacio");
    entities
        .try_extend_from_slices(&[200, 300], &[2.5, 3.5])
        .expect("las columnas coinciden y el lote cabe");

    let (ids, positions) = entities.as_slices();
    assert_eq!(ids, &[100, 200, 300]);
    assert_eq!(positions, &[1.5, 2.5, 3.5]);
    Ok(())
}
```

- `try_with_capacity(n)` reserva ambas columnas; si falla una reserva, no se devuelve un objeto parcialmente construido.
- `try_push(a, b)` inserta una fila completa o devuelve ambos valores en `SoaPushError { first, second }`.
- `try_extend_from_slices(a, b)` requiere `A: Clone` y `B: Clone`. Rechaza longitudes distintas (`SoaExtendError::LengthMismatch`) o un lote que no cabe (`CapacityExceeded`) antes de modificar las columnas.
- `as_slices()` y `as_mut_slices()` prestan ambas columnas sin permitir cambiar sus longitudes.
- `len()`, `is_empty()`, `capacity()` e `is_full()` describen el número lógico de filas; `pop()` extrae el último par y `clear()` elimina las filas conservando la reserva.

Las garantías de no modificación aplican a los errores normales de longitud/capacidad. `Clone` puede ejecutar código del usuario y entrar en pánico; estas operaciones no prometen rollback frente a un pánico durante la clonación.

## Cobertura de determinismo observable (punto 089)

`tests/memory_determinism.rs` repite secuencias sobre la API pública de `Buffer` y `SoaBuffer`: inserción hasta el límite, rechazo sin pérdida de contenido, extracción del último elemento, `clear`, reutilización de la capacidad y alineación de columnas al insertar por lotes y extraer filas. También repite errores de extensión por capacidad/longitud comprobando que el estado no cambia, y los contratos de buffer de capacidad cero (vacío y lleno simultáneamente). Las comprobaciones validan estados observables deterministas para una misma secuencia de operaciones; no implican sincronización interna ni una garantía de orden entre hilos.

## Ownership, reserva y concurrencia

Los buffers son propietarios y no incorporan sincronización interna. Las reservas usan contenedores estándar y operaciones fallibles de reserva; no hay allocator personalizado, crecimiento automático ni acceso a punteros crudos. La capacidad solicitada no representa un límite exacto de bytes.

En un solo hilo, usa el buffer directamente. Si varios hilos deben modificar el mismo estado, usa explícitamente `dinah::runtime::Shared<T>` y su `lock()`, tratando el error de sincronización como parte del flujo de control. Mantén el guard del mutex solo durante el acceso necesario. Las operaciones compuestas, como insertar una fila SoA, deben ejecutarse bajo el mismo guard para conservar las invariantes. Un mutex no garantiza fairness, ausencia de starvation ni mejor rendimiento.

`clear()` destruye valores; no es un reset genérico ni garantiza borrar físicamente bytes sensibles. Los slices solo cubren valores inicializados y no pueden sobrevivir al propietario. La implementación evita `unsafe` y sigue las reglas de ownership de Rust.

## Benchmarks y alcance

El benchmark reproducible está en `benches/memory.rs`. Los resultados históricos y sus notas se conservan en [`artifacts/benchmarks/`](../artifacts/benchmarks/): [línea base de Memory](../artifacts/benchmarks/memory-baseline-2026-10-10.txt) y [comparación de inserción SoA por lotes](../artifacts/benchmarks/memory-soa-batch-2026-10-11.txt). Las mediciones describen runners y condiciones concretos; no son garantías universales ni deben sustituir mediciones de la carga real.

Pools, arenas, crecimiento dinámico, límites estrictos en bytes, allocators personalizados, SIMD y ECS no forman parte de esta API inicial. Consulta el [modelo de diseño de Memory](memory-model.md) para las invariantes y decisiones aplazadas, y [`puntos.md`](../puntos.md) para el estado del roadmap.
