# DINAH — investigación SIMD (punto 083)

Estado: investigación completada; no se añade código SIMD ni se afirma una aceleración. La decisión se limita a fijar criterios para elegir una operación real antes de implementar el punto 084.

## Resumen de la decisión

**Mantener por ahora la ruta escalar segura y dejar que LLVM/Rust intente vectorizar bucles simples.** No introducir todavía `std::simd`, intrinsics de arquitectura, `unsafe`, alineación especial ni una dependencia externa. La Compute API actual es genérica y no existe aún un benchmark CPU dedicado que demuestre una operación concreta limitada por cálculo donde una ruta SIMD manual mejore el resultado de extremo a extremo.

## Qué significa SIMD para DINAH

SIMD (single instruction, multiple data) aplica una operación a varios lanes de datos en una instrucción vectorial. Puede ayudar en bucles regulares con operaciones aritméticas simples y acceso contiguo; no es una mejora automática. El coste de preparar vectores, tratar el resto de elementos, realizar conversiones o mover datos puede superar el ahorro, especialmente en entradas pequeñas.

La contigüidad de un slice facilita el acceso secuencial, pero **no garantiza por sí sola una alineación especial** ni asegura que el compilador genere instrucciones vectoriales. Tampoco todo kernel es vectorizable: dependencias entre iteraciones, ramas, aliasing, operaciones con efectos laterales y orden estricto pueden impedirlo o cambiar el resultado observable.

## Opciones consideradas

| Opción | Ventajas | Costes y límites | Decisión actual |
|---|---|---|---|
| Bucle escalar seguro y monomorfizado | Portátil; simple; referencia de corrección; LLVM puede vectorizar patrones reconocibles | La vectorización depende del código, optimizaciones, target y flags; no debe darse por garantizada | **Base recomendada** |
| `std::simd` / Portable SIMD | Modelo vectorial portable y tipos expresivos | La disponibilidad estable depende del toolchain; no comprometer la API pública a una superficie experimental sin verificar estabilidad | No adoptar aún |
| `std::arch` intrinsics por arquitectura | Control directo sobre instrucciones de CPU específicas | API específica de target; algunas operaciones requieren `unsafe` y detección de características; aumenta la matriz de pruebas | No adoptar aún |
| Crate externo de SIMD | Puede aportar abstracciones maduras y dispatch por arquitectura | Dependencia, política de compatibilidad, tamaño binario y contrato adicional que revisar | No añadir sin un caso medido |
| Assembly escrito a mano | Control máximo en un target concreto | Portabilidad y mantenimiento pobres; interacción compleja con ABI, compilador y seguridad | Fuera del alcance actual |

Esta evaluación no afirma que una alternativa sea siempre más rápida. La decisión debe cambiar si mediciones reproducibles demuestran un beneficio relevante para un caso de uso soportado.

## Compatibilidad con las reglas actuales del crate

- `src/lib.rs` contiene `#![forbid(unsafe_code)]`; añadir intrinsics que exijan `unsafe` no es compatible con la política actual sin una revisión explícita y separada de seguridad.
- `Cargo.toml` no declara dependencias externas. Mantener la implementación base sin dependencias conserva la superficie de mantenimiento actual.
- La Compute API opera sobre slices prestados y ofrece funciones genéricas. No hay todavía un kernel aritmético especializado cuya semántica y perfil de rendimiento justifiquen una API vectorial pública.
- Los contratos de Memory no prometen alineación SIMD especial. No añadir padding ni sobreasignación especulativa para intentar conseguirla.
- Debe conservarse una ruta escalar correcta, incluso si una futura implementación añade dispatch por características de CPU.

## Candidato para el punto 084

Evaluar primero una operación numérica concreta sobre slices contiguos, por ejemplo una transformación elemento a elemento con semántica claramente definida y sin dependencias entre elementos. Antes de escribir una ruta SIMD:

1. Especificar la semántica exacta (overflow, NaN, infinitos, signed zero y tolerancia, si aplica).
2. Implementar o conservar una referencia escalar sencilla y probarla.
3. Confirmar con inspección de código generado o herramientas adecuadas si la versión escalar ya se vectoriza en los targets elegidos.
4. Comparar la referencia con la alternativa vectorial en builds `--release`, incluyendo entradas pequeñas y grandes, longitudes no múltiplos del ancho vectorial y el tail.
5. Registrar CPU, sistema operativo, `rustc -Vv`, target, flags, método de medida, número de muestras y estadístico usado. Repetir las mediciones y comparar resultados, no solo una ejecución.
6. Rechazar la ruta especializada si la mejora no es repetible, no compensa la complejidad o altera el contrato numérico esperado.

## Matriz mínima de corrección

- Slice vacío, un elemento y entradas menores que el ancho vectorial.
- Longitudes exactas y no exactas respecto al ancho; el tail debe procesarse exactamente una vez.
- Valores límite del tipo, incluyendo overflow según la semántica elegida.
- Resultados comparados con la referencia escalar; igualdad exacta cuando el contrato lo permite y tolerancia explícita si se trata de aritmética flotante.
- Builds y tests en los targets soportados; una ruta especializada nunca debe ejecutarse si la CPU no soporta las características necesarias.
- No afirmar alineación ni usar lecturas/escrituras fuera de los límites del slice.

## Criterio de salida del punto 083

La investigación recomienda **no implementar SIMD manual todavía**. El punto 083 no incluye código vectorial ni números de rendimiento. El punto 084 deberá aportar un kernel concreto, tests de equivalencia, una comparación reproducible con la ruta escalar y evidencia de que el beneficio compensa el coste. Si no se encuentra ese beneficio, documentar y mantener la ruta escalar es un resultado válido.


## Punto 084 — medición inicial del kernel y decisión provisional

Se añadió `benches/compute.rs` y el harness de benchmarks lo ejecuta en CI como `cargo bench --bench compute`. Compara dos bucles seguros que aplican exactamente la misma transformación in-place (`x = x * 1.0001 + 0.25`): un bucle directo sobre el slice y `compute::for_each`. Comprueba que los resultados finales coinciden y mide cuatro tamaños (64, 1.024, 16.384 y 262.144 elementos), con 11 muestras por caso y mediana. Este benchmark mide el coste relativo del bucle/API; **no compara todavía una implementación SIMD manual contra una escalar**.

### Entorno exploratorio registrado

- Dispositivo: Samsung SM-A165F; Android 16 / Termux.
- Arquitectura/target: `aarch64-linux-android`; CPU ARMv8, `CPU part 0xd05`, con `asimd` anunciado por el sistema.
- Compilador: `rustc 1.99.0 (b940084d7 2026-09-28)`, LLVM 21.1.8.
- Comando: `cargo bench --bench compute`; perfil release; tres ejecuciones del harness, cada una con 11 muestras por caso.
- Los tiempos fluctúan de forma visible entre ejecuciones, por lo que son observaciones de este dispositivo, no resultados portables ni una afirmación de aceleración.

### Resultados observados (mediana por ejecución)

| Elementos | Bucle directo (ns) | `compute::for_each` (ns) | Lectura prudente |
|---:|---:|---:|---|
| 64 | 77, 77, 77 | 77, 77, 77 | Sin diferencia medible en la resolución observada |
| 1.024 | 308, 385, 538 | 308, 385, 538 | Coinciden en las tres ejecuciones |
| 16.384 | 3.693, 5.461, 5.308 | 3.693, 5.538, 5.154 | Diferencias pequeñas y con dirección variable |
| 262.144 | 65.154, 92.923, 80.000 | 68.615, 93.462, 81.846 | La diferencia cambia entre aproximadamente +5,3 %, +0,6 % y +2,3 % de tiempo para `for_each`; no prueba una mejora SIMD |

La inspección del ensamblado release confirma instrucciones de carga/almacenamiento vectorial en partes del binario, pero también operaciones aritméticas escalares en el kernel inspeccionado. Esa inspección preliminar no basta para atribuir vectorización a una función específica ni para afirmar que el compilador ya vectoriza el kernel de extremo a extremo.

### Decisión provisional

No se añade SIMD manual, `unsafe`, intrinsics ni una dependencia nueva: las mediciones actuales no comparan una alternativa SIMD y no demuestran una mejora reproducible. El punto 084 permanece **en progreso** hasta que se evalúe una implementación vectorial segura y compatible, o se establezca con evidencia suficiente que no existe un beneficio que compense el coste. Próximos pasos: aislar el kernel en ensamblado, probar una alternativa vectorial estable/segura si encaja con la política del crate, medirla con el mismo harness y validar longitudes pequeñas, tails y equivalencia numérica.
