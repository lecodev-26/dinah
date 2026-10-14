# dinah Networking — diseño de API y límites de v0.1

Estado: diseño incremental (puntos 120–122). Este documento fija las fronteras de Networking y los contratos de transporte y mensaje. No declara que haya sockets, serialización ni conexiones implementados.

## Objetivo

Networking ofrecerá primitivas explícitas para intercambiar mensajes entre endpoints, con transporte y formato de wire separados de la lógica de aplicación. Más adelante podrá soportar simulación de latencia, pérdida, predicción y reconciliación para Simulation, pero estas capacidades no se presuponen en el diseño inicial.

## Responsabilidades y fronteras

- **Core:** tipos y errores verdaderamente compartidos. No depende de Networking ni contiene política de transporte.
- **Runtime:** ejecuta trabajo explícito. Networking no debe crear un scheduler oculto, pool persistente ni tareas de fondo implícitas.
- **Simulation:** aporta estado y pasos de simulación; Networking no modifica el estado de Simulation de forma implícita.
- **Networking:** define endpoints, conexiones, mensajes, contratos de transporte y serialización, además de errores de red.
- **Aplicación:** decide autenticación/autorización, protocolo de juego o negocio, límites de confianza y cómo tratar mensajes válidos pero no autorizados.

La primera implementación debe usar Rust seguro, sin `unsafe` ni dependencias externas hasta que un requisito concreto y una revisión de licencia/seguridad justifiquen una excepción.

## API propuesta por capas

La API pública se desarrollará incrementalmente; los nombres siguientes son propuestas de diseño, no tipos existentes todavía.

```rust
pub trait Transport {
    type Address;
    type Connection;
    type Error;

    fn connect(&mut self, address: Self::Address)
        -> Result<Self::Connection, Self::Error>;
    fn send(&mut self, connection: &mut Self::Connection,
            frame: &[u8]) -> Result<(), Self::Error>;
    fn receive(&mut self, connection: &mut Self::Connection,
               buffer: &mut [u8]) -> Result<usize, Self::Error>;
}
```

Esta firma es un punto de discusión, no una promesa de API final. El punto 121 ya fija el primer transporte como byte stream bloqueante, con lectura parcial y EOF explícito; las firmas definitivas deberán reflejar ese contrato y no tratar una llamada de escritura como si delimitara un mensaje completo. Los tipos de error y el proveedor concreto se decidirán al implementar conexiones.

Capas previstas:

1. **Mensaje lógico:** tipo/versión de mensaje y payload, sin acoplarlo a TCP, UDP ni a una estructura Rust en memoria.
2. **Serialización:** codifica/decodifica bytes con límites explícitos, validación de longitud y errores tipados. Nunca confiar en tamaños proporcionados por el peer.
3. **Transporte:** entrega bytes y declara semántica de stream/datagrama, bloqueo, orden, errores y cierre.
4. **Conexión y endpoint:** identidad local/remota y estado de conexión, sin confundir dirección, sesión autenticada y autorización.
5. **Integración:** adaptadores explícitos hacia Runtime y Simulation; ninguna mutación oculta ni callback de fondo implícito.

## Decisión de transporte — punto 121

Para la primera versión se define un **transporte de byte stream bloqueante y explícito**, con operaciones de lectura y escritura que pueden completar una cantidad parcial de bytes. El contrato es independiente del proveedor concreto: la primera implementación con sockets queda para el punto de conexiones y no forma parte de este punto de diseño. No se añade UDP ni una abstracción común stream/datagrama todavía; esos modelos tienen semánticas distintas y no deben esconderse tras la misma API.

Contrato elegido:

- **Escritura completa:** la operación de alto nivel `write_all(bytes)` solo devuelve éxito cuando todos los bytes solicitados se han aceptado. Una implementación interna puede hacer varias escrituras parciales. Si ocurre un error después de escribir un prefijo, devuelve el error y no reintenta de forma oculta desde cero: el llamador debe tratar el estado del stream como potencialmente desincronizado y normalmente cerrar la conexión.
- **Lectura:** `read(buffer)` devuelve el número de bytes realmente leídos, que puede ser menor que el tamaño del buffer. `0` significa EOF/cierre ordenado del peer, no “todavía no hay datos”. La lectura parcial no delimita mensajes; el framing pertenece a una capa posterior.
- **Bloqueo:** las operaciones esperan a que haya progreso o un error del sistema. La primera API no expone `WouldBlock` como flujo normal ni promete operaciones no bloqueantes; una evolución asíncrona requerirá otro contrato explícito.
- **Timeouts:** no se oculta ningún reloj ni deadline global. Si el proveedor permite configurar timeout, se establece explícitamente en su configuración y el timeout se devuelve como error tipado. No se promete una latencia máxima portátil.
- **Cierre:** EOF indica cierre remoto de lectura. Los errores de escritura/lectura se conservan como errores; no se convierten en buffer vacío. El cierre local debe ser una operación explícita del proveedor de conexión, especificada al implementar conexiones.
- **Propiedad y concurrencia:** los bytes se prestan solo durante la llamada y no se retienen. El objeto de conexión es mutable y no se comparte concurrentemente sin sincronización externa. El orden de bytes se conserva por la semántica del stream, pero una llamada de lectura no corresponde necesariamente a una escritura.
- **Límites:** el transporte mueve bytes y no asigna memoria según longitudes remotas. La capa de framing/serialización define y valida los máximos antes de reservar payloads.
- **Fallos parciales:** tras un fallo de escritura parcial, el estado del protocolo se considera incierto; no se reintenta automáticamente la trama completa. La recuperación o reconexión queda fuera del contrato base.

Este punto cierra una decisión de diseño, no una implementación de red. Las pruebas del contrato pueden usar un proveedor falso; las pruebas de sockets locales pertenecen al punto 137 y no deben depender de tiempos exactos ni de servicios externos.

## Contrato de mensaje lógico — punto 122

Un mensaje lógico es una unidad de protocolo independiente del transporte concreto y del layout de bytes. La lectura del stream no crea mensajes: el framing que permitirá reconstruirlos se especifica en el punto 123. Este punto define la semántica y la identidad del mensaje, no su codificación binaria.

### Envelope conceptual

Cada mensaje debe tener estos elementos semánticos:

- **Versión de protocolo:** identifica la versión del contrato común de Networking que interpreta el envelope. No equivale a la versión de un tipo de mensaje.
- **Identificador de tipo:** identificador numérico estable asignado a una clase de mensaje dentro del protocolo. El valor cero queda reservado como inválido; un identificador publicado no se reutiliza para otro significado.
- **Versión de mensaje:** versión del esquema semántico del tipo identificado, separada de la versión global del protocolo.
- **Payload opaco:** contenido específico del tipo. La capa de mensajes no interpreta structs Rust en memoria ni presupone que el payload tenga una representación portable sin serialización explícita.

Como notación de diseño, no como API ya implementada:

```rust
struct MessageEnvelope<'a> {
    protocol_version: ProtocolVersion,
    message_type: MessageType,
    message_version: MessageVersion,
    payload: &'a [u8],
}
```

Estos nombres ilustran el contrato conceptual; no existen aún como tipos públicos de Rust. La representación y el orden exactos de los campos, el endianness, la codificación de enteros y el framing pertenecen al punto 123.

### Reglas de validación y evolución

- El receptor valida la versión de protocolo, el identificador de tipo, la versión del mensaje y el tamaño del payload antes de entregar el mensaje a la lógica de aplicación.
- Un tipo desconocido o una versión de mensaje no admitida produce un error tipado por defecto; ignorarlo solo se permitirá mediante una política de compatibilidad explícita del protocolo, nunca por descarte silencioso implícito.
- Una versión incompatible del protocolo se rechaza antes de despachar el payload. Las reglas de compatibilidad hacia delante/atrás deben declararse por versión; no se infieren del hecho de que el payload pueda decodificarse.
- Los identificadores publicados son estables y no se reasignan. Una modificación incompatible del significado requiere un nuevo identificador o una nueva versión de mensaje, conforme a las reglas de compatibilidad documentadas.
- El tamaño máximo de payload y del mensaje completo debe ser finito y comprobarse antes de reservar memoria basada en datos remotos. El valor concreto y la representación de longitud se fijan en el punto 123; este punto no inventa todavía un límite numérico.
- Los errores de mensaje distinguen, como mínimo, versión de protocolo no admitida, tipo desconocido, versión de mensaje no admitida y límite excedido. Los errores de framing y de decodificación se detallan en serialización.
- El orden solo es el que garantice el transporte seleccionado. Este envelope no añade secuencias, confirmaciones, entrega exactamente-una-vez ni deduplicación.
- Un mensaje válido no implica autenticación, autorización, confianza en el emisor ni cifrado. La aplicación decide la política de seguridad y valida los datos del dominio.
- La identidad de conexión, el estado de sesión, los mensajes de control de ciclo de vida, los ACK/reintentos y la reconexión quedan fuera del envelope base y se deciden en los puntos de conexiones/estado o en un protocolo de aplicación explícito.

### Criterios de aceptación del punto 122

- Se definen los campos semánticos del envelope y se separan versión de protocolo y versión de mensaje.
- Se fijan reglas para identificadores estables, tipos/versiones desconocidos, compatibilidad y límites sin decidir prematuramente la codificación binaria.
- Se distinguen los errores de mensaje de los errores de framing/serialización y no se prometen garantías de entrega que el transporte no proporciona.
- Se declara expresamente que el ejemplo de Rust es conceptual y que no hay tipos de mensaje implementados todavía.

## Contratos obligatorios

- **Ownership:** buffers prestados durante la llamada, salvo que una API futura documente explícitamente una transferencia de propiedad. No retener referencias a buffers del llamador.
- **Framing:** el stream necesita delimitación propia (longitud prefijada u otro framing definido); las fronteras de lectura no son fronteras de mensajes. Datagramas conservan los límites del datagrama, sujetos al máximo configurado.
- **Límites:** longitud máxima de frame y de mensaje configurable/constante documentada; rechazar overflow y entradas truncadas antes de asignar memoria en función de datos remotos.
- **Errores:** distinguir configuración inválida, fallo de transporte, cierre/EOF, frame inválido, serialización inválida y límite excedido. No convertir errores en payloads vacíos ni reintentar silenciosamente.
- **Tiempo:** los timeouts y deadlines deben ser explícitos. La primera API no debe consultar un reloj global oculto ni prometer latencia máxima.
- **Concurrencia:** no compartir una conexión mutable entre hilos sin sincronización explícita. El orden solo se garantiza donde lo prometa el transporte seleccionado.
- **Seguridad:** el framing no es autenticación ni cifrado. No confiar en el peer; no exponer claves/tokens en logs; no habilitar deserialización de tipos arbitrarios.
- **Cierre:** definir cuándo una conexión está abierta/cerrada y cómo se propagan EOF, cierre local y cierre remoto antes de prometer recuperación automática.
- **Determinismo:** los tests de protocolo usan buffers y transporte falso; los tests con sockets no deben depender de timings exactos ni de disponibilidad externa.
- **Portabilidad:** la API de alto nivel no debe filtrar tipos de socket específicos del sistema operativo si puede expresar el contrato de forma portable.

## Decisiones aplazadas y orden de implementación

- **121 Transporte (decisión tomada):** stream de bytes bloqueante; lecturas parciales permitidas, EOF definido, escritura completa con errores de progreso parcial, timeout explícito del proveedor y sin reintento automático. No se mezcla stream con datagramas ni se afirma que existan sockets implementados.
- **122 Mensajes (decisión tomada):** envelope conceptual con versión de protocolo, identificador estable de tipo, versión de mensaje y payload opaco; codificación binaria y framing quedan para 123.
- **123 Serialización:** elegir formato acotado, con versiones y errores; medirlo antes de introducir dependencias.
- **124–126 Conexiones/endpoints/estado:** definir ownership, estados y cierre antes de añadir recuperación.
- **127–131 Prediction/reconciliation/history/resimulation:** documentar primero el modelo temporal, snapshots, autoridad, orden de inputs y límites de memoria; no declarar determinismo de red como garantía de transporte.
- **132–134 Pérdida/latencia/desconexión:** construir un simulador controlado y tests deterministas antes de implementar políticas de producción.
- **135–136 Integración Runtime/Simulation:** solo mediante adaptadores explícitos, sin acoplamiento circular ni efectos secundarios ocultos.
- **137–139 Tests/latencia/benchmarks:** tests unitarios con transportes falsos, escenarios de integración locales y benchmarks reproducibles con entorno archivado.
- **140 Documentación:** documentar exclusivamente comportamiento implementado y límites medidos.

## Criterios de aceptación del punto 120

El punto 121 queda definido cuando el contrato de transporte fija semántica de stream, lectura parcial/EOF, escritura parcial, bloqueo, timeout, cierre, ownership, límites y recuperación tras error, sin presentarlo como implementación.

- Responsabilidades separadas de Core, Runtime, Simulation y Networking.
- Capas lógicas de mensaje, serialización, transporte, conexión e integración identificadas.
- Contratos iniciales de ownership, framing, límites, errores, tiempo, concurrencia, cierre, seguridad y determinismo definidos.
- Decisiones no resueltas etiquetadas como propuestas, no como APIs existentes.
- Ningún transporte, socket, serializador o característica de predicción se considera implementado por este documento.

## Criterios de aceptación del punto 121

El punto 121 queda definido cuando el contrato de transporte fija semántica de stream, lectura parcial/EOF, escritura parcial, bloqueo, timeout, cierre, ownership, límites y recuperación tras error.

## Criterios de aceptación del punto 123 (pendiente)

La serialización debe elegir una codificación binaria/versionada, framing, representación de longitudes, límites numéricos y errores de decodificación. Estos detalles no se consideran decididos por el punto 122.
