# DINAH — evaluación inicial de layouts AoS/SoA (punto 085)

Estado: exploración inicial documentada; **no se adopta un layout universal ni se declara completado el punto 085**.

## Pregunta y alcance

Se comparan dos representaciones de los mismos datos (`first = i`, `second = i + 1`):

- **AoS (Array of Structures):** `Vec<DataRow>`, con ambos campos en cada fila.
- **SoA (Structure of Arrays):** `SoaBuffer`, con cada campo en una columna contigua.

El benchmark `benches/memory.rs` mide recorridos que leen solo `first` y recorridos que leen y combinan ambos campos. Las estructuras se construyen antes de medir, por lo que estos resultados **no incluyen** coste de construcción, conversión, asignación ni mantenimiento de ambas representaciones. Cada caso informa la mediana de siete muestras en tamaños de 64, 1.024 y 16.384 filas.

## Evidencia reproducible disponible

- Commit del código medido: `92069269bbda514702808e769a416dbd519e76d6`.
- Ejecución de benchmarks: [GitHub Actions run 38206675890](https://github.com/lecodev-26/dinah/actions/runs/38206675890).
- Los resultados completos, el entorno y el manifiesto se archivaron automáticamente en `artifacts/benchmarks/92069269bbda514702808e769a416dbd519e76d6/run-38206675890/`, separados por runner.
- Tres ejecuciones independientes por runner; cada caso resume siete muestras por ejecución. La primera midió el commit `92069269bbda514702808e769a416dbd519e76d6`; la segunda y la tercera midieron el mismo código de benchmark (`114e0f73e05d17bdfd6bb7002a52ee234b72f834` y `8854d3794dfda846f7465920930b6c403a1d3ab8`, respectivamente). La tercera ejecución amplía la evidencia, pero tres ejecuciones no bastan para afirmar estabilidad estadística general.

## Resultados a 16.384 filas

Los tiempos son nanosegundos por operación tal como los informa el harness. El porcentaje expresa el cambio de SoA respecto a AoS: negativo favorece SoA; positivo favorece AoS. Los valores se comparan solo dentro de cada runner.

| Runner | Leer un campo AoS | Leer un campo SoA | Cambio SoA | Leer ambos AoS | Leer ambos SoA | Cambio SoA |
|---|---:|---:|---:|---:|---:|---:|
| Linux x86-64 | 0,473 | 0,465 | −1,7 % | 0,886 | 0,895 | +1,0 % |
| Linux ARM64 | 0,469 | 0,407 | −13,2 % | 0,694 | 0,773 | +11,4 % |
| macOS x86-64 | 0,511 | 0,421 | −17,6 % | 0,851 | 0,817 | −4,0 % |
| macOS ARM64 | 0,315 | 0,394 | +25,1 % | 0,684 | 0,732 | +7,0 % |
| Windows x86-64 | 0,378 | 0,513 | +35,7 % | 0,781 | 0,745 | −4,6 % |

## Repeticiones de la medición

Se lanzó una segunda ejecución completa sobre los cinco runners: [GitHub Actions run 38207649616](https://github.com/lecodev-26/dinah/actions/runs/38207649616). Los resultados completos y el entorno quedaron archivados en `artifacts/benchmarks/114e0f73e05d17bdfd6bb7002a52ee234b72f834/run-38207649616/`. A 16.384 filas, estos son los cambios de SoA respecto a AoS en cada ejecución (negativo favorece SoA; positivo favorece AoS):

| Runner | Un campo: primera | Un campo: repetición | Ambos: primera | Ambos: repetición |
|---|---:|---:|---:|---:|
| Linux x86-64 | −1,7 % | −1,2 % | +1,0 % | +0,0 % |
| Linux ARM64 | −13,2 % | −2,6 % | +11,4 % | +10,8 % |
| macOS x86-64 | −17,6 % | −1,6 % | −4,0 % | −4,9 % |
| macOS ARM64 | +25,1 % | −9,7 % | +7,0 % | −2,0 % |
| Windows x86-64 | +35,7 % | +33,8 % | −4,6 % | −4,9 % |

La repetición confirma que **no hay una ventaja universal**, pero tampoco que todas las diferencias grandes sean estables: la dirección de macOS ARM64 cambia en ambos patrones; Linux ARM64 mantiene la dirección pero reduce mucho la ventaja al leer un campo; Windows conserva una penalización de SoA al leer un campo, mientras que en lectura de ambos campos favorece SoA por un margen pequeño. En Linux x86-64 los resultados quedan prácticamente empatados. Estos resultados no justifican cambiar la representación ni elegir un layout por defecto.

El tamaño intermedio (1.024 filas) tampoco da un ganador universal: AoS gana en lectura de un campo en los cinco runners en esa ejecución, mientras que la lectura de ambos campos varía según runner. Para 64 filas, las mediciones son demasiado pequeñas y cuantizadas para fundamentar una decisión de diseño.

Se completó una tercera ejecución completa en cinco runners: [GitHub Actions run 38208067774](https://github.com/lecodev-26/dinah/actions/runs/38208067774), archivada en `artifacts/benchmarks/8854d3794dfda846f7465920930b6c403a1d3ab8/run-38208067774/`. A 16.384 filas, el cambio de SoA respecto a AoS fue:

| Runner | Un campo: tercera | Ambos: tercera |
|---|---:|---:|
| Linux x86-64 | +8,7 % | +0,3 % |
| Linux ARM64 | −2,9 % | +11,1 % |
| macOS x86-64 | +1,0 % | −3,6 % |
| macOS ARM64 | +23,1 % | +0,5 % |
| Windows x86-64 | +35,6 % | +0,7 % |

La tercera ejecución refuerza la señal de Windows: SoA fue aproximadamente un 34–36 % más lento al leer un campo en las tres ejecuciones. En cambio, las diferencias al leer ambos campos suelen ser pequeñas, salvo Linux ARM64, donde SoA fue alrededor de un 11 % más lento en las tres ejecuciones. La lectura de un campo cambia de dirección o magnitud en Linux x86-64 y macOS ARM64; macOS x86-64 queda casi empatado en la tercera ejecución. Esto sigue siendo evidencia de un microbenchmark sintético, no una recomendación de layout para cargas reales.

## Interpretación y decisión

1. SoA puede ayudar cuando el recorrido consume una columna y evita cargar campos que no se necesitan, pero el efecto medido depende de la arquitectura y del tamaño.
2. Cuando se consumen ambos campos, AoS es competitivo y a menudo más rápido en esta muestra; SoA no debe presentarse como una optimización general.
3. Los porcentajes grandes de una sola ejecución no prueban una ventaja estable. Hay diferencias de resolución, ruido del runner y efectos del compilador/caché.
4. La prueba excluye el coste de crear y mantener el layout. Un cambio de arquitectura solo se justificaría para una carga de trabajo real donde la distribución de lecturas y escrituras esté definida.

**Decisión:** mantener ambos tipos disponibles y no cambiar la API ni la representación interna por estos números. Antes de considerar optimización adicional, repetir el benchmark en varios commits/ejecuciones por runner, conservar la configuración de entorno y añadir un caso que mida el coste de construcción/conversión si un consumidor real lo necesita. Comparar las medianas dentro de cada runner y exigir una ventaja repetible que compense la complejidad.

## Criterio de salida del punto 085

- [x] Añadir benchmark comparativo AoS/SoA con valores equivalentes para lectura de un campo y de ambos campos.
- [x] Archivar resultados y entorno de los cinco runners.
- [x] Documentar que la evidencia inicial es mixta y no autoriza una optimización general.
- [x] Completar una tercera ejecución en los cinco runners y documentar las señales repetidas y las que siguen siendo variables.
- [ ] Si se identifica una carga de trabajo representativa, medir también creación/conversión y el patrón de escrituras correspondiente.

Las tres ejecuciones no justifican una representación universal ni un cambio de API. Sigue pendiente una carga de trabajo representativa que permita medir creación/conversión y escrituras; por ello, el punto 085 permanece **en progreso**.
