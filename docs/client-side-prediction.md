# Client-side prediction — investigación (punto 127)

Estado: investigación de arquitectura completada. Este documento fija un modelo recomendado para los puntos 128–131; **no implementa prediction, reconciliation, buffers de historial ni resimulación**.

## Problema y objetivo

En un modelo cliente-servidor autoritativo, esperar a que un input viaje al servidor y vuelva su resultado añade al control al menos el retardo de ida y vuelta más el tiempo de procesamiento. La predicción del cliente ejecuta inmediatamente una simulación local de los inputs propios para que el control se sienta receptivo. Esa simulación es provisional: el servidor conserva la autoridad y puede rechazar inputs o producir un estado distinto.

La predicción no es reconciliación. Prediction estima el resultado antes de recibir confirmación; reconciliation incorpora un estado autoritativo posterior y vuelve a aplicar los inputs aún no confirmados para recuperar el presente local.

## Modelo recomendado para DINAH

Adoptar para la primera iteración un modelo **autoritativo de servidor + inputs numerados + predicción local acotada + reconciliación por secuencia confirmada**. Debe ser una capa explícita sobre Simulation, no una responsabilidad implícita de `network::Connection`.

1. El cliente crea un input de dominio con secuencia monotónica y tick lógico. El input representa una intención validable (por ejemplo, dirección de movimiento), no una posición final que el cliente pueda imponer al servidor.
2. El cliente guarda el input en un historial acotado y lo aplica inmediatamente a su estado predictivo local.
3. El transporte envía inputs al servidor. La entrega, el orden, la duplicación y la pérdida deben definirse en el protocolo posterior; TCP existente solo ofrece un byte stream y no equivale a un protocolo de inputs.
4. El servidor valida y procesa los inputs conforme a su política autoritativa y publica un snapshot del estado con el último número de secuencia procesado para ese cliente.
5. Al recibir un snapshot, el cliente instala como base el estado autoritativo, descarta inputs reconocidos y vuelve a simular en orden los inputs pendientes posteriores al reconocimiento.
6. La simulación corregida es el estado de juego. Cualquier suavizado visual debe ser una capa de presentación separada: no debe contaminar el estado usado para física, colisiones, reglas o próximos inputs.

## Datos y contratos que deben fijarse antes de implementar

- **Input:** secuencia, tick o referencia temporal lógica y payload de dominio validado. Los tipos exactos pertenecen al protocolo de aplicación, no al wrapper TCP.
- **Snapshot autoritativo:** tick del servidor, estado mínimo necesario para reanudar la simulación y última secuencia de input procesada. Debe ser posible distinguir snapshot viejo, duplicado o incompatible.
- **Historial:** buffer circular o estructura acotada que guarda inputs pendientes; cada entrada tiene identidad estable. La política de capacidad, overflow y pérdida de entradas debe ser explícita, sin crecer sin límite por datos remotos.
- **Reloj:** usar ticks explícitos de `Simulation`; no depender de `wall-clock`, del orden de llegada de paquetes ni de timers ocultos para producir un replay reproducible.
- **Autoridad y seguridad:** el servidor valida cada input y el estado definitivo. La predicción del cliente no concede permisos ni constituye evidencia fiable para economía, combate, inventario o resultados competitivos.
- **Errores y límites:** inputs inválidos, secuencias imposibles, snapshots obsoletos, saltos de tick, historial agotado y versiones incompatibles necesitan políticas explícitas. No ocultar un fallo borrando el historial y fingiendo una reconciliación correcta.
- **Integración:** Networking transporta bytes; el futuro codec valida el wire format; una capa de protocolo mapea mensajes a inputs/snapshots; Simulation ejecuta la transición de estado. Evitar dependencias circulares y mutación oculta del mundo.

## Determinismo: requisito condicionado, no supuesto

El replay solo reconstruirá el mismo resultado si el modelo simulado es suficientemente reproducible para el estado inicial y la misma secuencia de inputs. Debe investigarse por cada modelo: orden de iteración, aritmética de punto flotante, estado externo, aleatoriedad, concurrencia y efectos secundarios. No se debe declarar determinismo bit a bit entre arquitecturas sin pruebas específicas. Si la simulación no es determinista, la reconciliación todavía puede corregir el estado, pero la resimulación puede divergir y necesitar snapshots más completos o un diseño distinto.

## Corrección y experiencia visual

La opción segura por defecto es corregir el estado de simulación a la autoridad y reejecutar inputs pendientes. Un pequeño offset visual amortiguado puede disimular correcciones pequeñas, pero no debe alterar la verdad de simulación ni esconder indefinidamente una divergencia grande. Para correcciones grandes, estado inválido o historial perdido, priorizar convergencia explícita sobre suavidad estética.

La predicción del jugador local no resuelve la visualización de otras entidades. Interpolación de snapshots remotos y lag compensation son problemas relacionados pero separados; no deben colarse dentro del punto 128.

## Riesgos y límites

- Colisiones con otros jugadores, objetos movidos por el servidor, eventos simultáneos y reglas no predecibles producen correcciones legítimas.
- Un cliente puede ejecutar inputs localmente que el servidor luego rechace; el juego debe separar feedback provisional de efectos autoritativos irreversibles.
- TCP puede sufrir bloqueo de cabecera de línea cuando se pierde un segmento; el transporte actual no ofrece garantías de tiempo real ni es una política de entrega de inputs.
- Un historial demasiado corto no permite replay; uno ilimitado expone al cliente a consumo de memoria creciente. La capacidad debe derivarse de una ventana temporal y un máximo configurados.
- No es correcto asumir que la predicción eliminará el lag, evitará trampas o garantizará sincronización exacta.

## Secuencia de trabajo para puntos 128–131

- **128 — Implementar prediction:** comenzar con un modelo de estado pequeño y puro, inputs explícitos y tick fijo; comparar resultado predictivo con la ejecución de referencia. No integrarlo aún con sockets.
- **129 — Implementar reconciliation:** definir snapshot autoritativo y secuencia confirmada; aplicar snapshot y reejecutar solo los inputs posteriores a esa secuencia. Cubrir snapshots atrasados, duplicados y discrepancias.
- **130 — Implementar history/state buffers:** limitar capacidad, validar secuencias, especificar overflow y medir memoria; los buffers no deben retener datos sin límite.
- **131 — Implementar deterministic resimulation:** replay repetible con la misma base/inputs, tests de repetición y trazas por tick. Documentar exactamente el alcance del determinismo que las pruebas demuestren.

Los tests iniciales deben ser deterministas y locales, con retrasos, duplicados, pérdida, reordenación y snapshots antiguos simulados mediante entradas explícitas; no deben depender de sleeps ni de servicios externos. Los escenarios de pérdida/latencia de producción corresponden también a los puntos 132–134.

## Criterios de aceptación de la investigación

- Separación explícita entre predicción, reconciliación, historial, resimulación e interpolación visual.
- Autoridad del servidor y confianza limitada en el cliente documentadas.
- Protocolo conceptual de secuencias, snapshots y inputs pendientes descrito sin afirmar APIs existentes.
- Determinismo, límites de memoria, errores, eventos no predecibles y políticas de corrección identificados.
- Plan incremental para 128–131 y pruebas deterministas definido.

## Referencias consultadas

- Gabriel Gambetta, [Client-Side Prediction and Server Reconciliation](https://www.gabrielgambetta.com/client-side-prediction-server-reconciliation.html): inputs numerados, estado autoritativo y replay de inputs no confirmados.
- Glenn Fiedler, [What Every Programmer Needs To Know About Game Networking](https://www.gafferongames.com/post/what_every_programmer_needs_to_know_about_game_networking/): historial circular de estado/inputs y rewind/replay tras una corrección.
- Gabriel Gambetta, [Entity Interpolation](https://www.gabrielgambetta.com/entity-interpolation.html): predicción de entidad local frente a interpolación de entidades remotas.
- Glenn Fiedler, [Snapshot Interpolation](https://gafferongames.com/post/snapshot_interpolation/): límites de asumir determinismo perfecto y alternativa de interpolar snapshots.

Las referencias fundamentan el modelo conceptual; no implican copiar sus implementaciones ni garantizan que sus parámetros sean adecuados para DINAH.
