# dinah Networking — diseño de API y límites de v0.1

Estado: implementación incremental (puntos 120–127). Los contratos de transporte, mensaje y wire format están documentados; existen una conexión TCP bloqueante, un endpoint TCP de escucha/aceptación y seguimiento local del ciclo de vida de conexión. El encoder/decoder de wire sigue sin implementarse. La investigación de predicción cliente está en `docs/client-side-prediction.md`; no hay prediction/reconciliation implementadas.

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
4. **Conexión y endpoint:** identidad local/remota y estado de conexión observado localmente, sin confundir dirección, sesión autenticada y autorización.
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

Estos nombres ilustran el contrato conceptual; no existen aún como tipos públicos de Rust. La representación y el orden exactos de los campos, el endianness, la codificación de enteros y el framing se fijan en el punto 123.

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

## Endpoint TCP — punto 125

`dinah::network::Endpoint` envuelve un `std::net::TcpListener` y permite crear un socket de escucha, consultar la dirección local, configurar explícitamente el modo no bloqueante y aceptar conexiones entrantes como `Connection`. No inicia tareas en segundo plano ni mantiene un registro de las conexiones aceptadas.

- `Endpoint::bind(SocketAddr)` enlaza la dirección solicitada; el puerto `0` permite solicitar un puerto disponible al sistema operativo.
- `local_addr()` informa la dirección local efectiva, incluido el puerto asignado.
- `accept()` devuelve una `Connection` propietaria del `TcpStream` aceptado y la dirección del peer. Por defecto bloquea hasta una conexión o error del sistema operativo.
- `set_nonblocking(true)` hace explícito el modo no bloqueante; `accept()` puede entonces devolver `WouldBlock` si no hay conexiones pendientes. No se añade polling ni scheduler.
- El endpoint no añade framing, serialización, autenticación, cifrado, reconexión ni gestión de estado.

### Criterios de aceptación del punto 125

- API pública para bind, dirección local, aceptación y modo no bloqueante explícito.
- Las conexiones aceptadas reutilizan el wrapper `Connection` ya implementado.
- Pruebas loopback cubren bind/accept/intercambio de bytes y el comportamiento `WouldBlock` explícito.
- No se crean tareas de fondo ni se afirma que exista gestión de estado de conexión.

## Conexiones TCP — punto 124

La primera implementación concreta es `dinah::network::Connection`, un wrapper propietario de `std::net::TcpStream`. Solo proporciona operaciones de byte stream; no convierte escrituras en mensajes ni incorpora todavía el framing v1 del punto 123.

- `Connection::connect(SocketAddr)` conecta usando el comportamiento de timeout predeterminado del sistema operativo.
- `Connection::connect_timeout(SocketAddr, Duration)` permite solicitar un timeout explícito de conexión y rechaza una duración cero como entrada inválida.
- `Connection::from_tcp_stream(TcpStream)` toma ownership de un stream ya establecido, como uno aceptado por un listener.
- `read(&mut [u8])` conserva la semántica de lectura parcial y EOF; los errores, incluidos los timeouts configurados, se devuelven como `ConnectionError::Io`.
- `write_all(&[u8])` solo devuelve éxito si todos los bytes fueron escritos. Ante error parcial, el estado del stream puede ser incierto; no hay reintento automático de la trama completa.
- `set_read_timeout` y `set_write_timeout` configuran explícitamente los timeouts de operaciones. `None` desactiva el timeout, sujeto al comportamiento de la plataforma.
- `local_addr`, `peer_addr` y `shutdown` exponen las direcciones del socket y el cierre explícito. No hay reconexión automática, tareas de fondo, autenticación, cifrado ni sincronización interna.

Los métodos requieren acceso mutable para leer/escribir. El wrapper no implementa `Clone` ni duplica sockets implícitamente. Las pruebas usan loopback y un listener local, sin servicios externos ni aserciones de latencia exacta. Esta API TCP inicial no implica soporte UDP ni estabilidad de API.

### Criterios de aceptación del punto 124

- Wrapper público propietario de `TcpStream` con conexión normal y conexión con timeout explícito.
- Operaciones de bytes, EOF, errores, direcciones y cierre explícitas y documentadas.
- Pruebas loopback para ida/vuelta de bytes, EOF, stream preexistente y rechazo de timeout cero.
- Sin afirmar framing, serialización, autenticación, reconexión o endpoints como capacidades ya implementadas.

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
- **123 Serialización (decisión tomada):** framing por longitud prefijada de 32 bits big-endian; cabecera de 6 bytes con versión de protocolo, tipo y versión de mensaje como enteros de 16 bits big-endian; frame máximo de 1 MiB incluido el encabezado; rechazos explícitos de longitud inválida, truncamiento y versiones/tipos inválidos. Formato propio mínimo, sin dependencia externa ni serialización de structs Rust.
- **124 Conexiones (implementado inicialmente):** `dinah::network::Connection` envuelve `TcpStream` con lectura/escritura bloqueante, timeout explícito, direcciones y shutdown; sin framing integrado ni reconexión automática. **125 Endpoints (implementado inicialmente):** `dinah::network::Endpoint` envuelve `TcpListener` con bind, dirección local, accept y modo no bloqueante explícito. **126 Gestión de estado (implementado inicialmente):** `ConnectionState` registra el estado local observable (`Open`, `ReadClosed`, `WriteClosed`, `Closed`); no detecta inmediatamente un cierre remoto ni gestiona reconexión. **127 Prediction (investigación completada):** el modelo recomendado está en `docs/client-side-prediction.md`; no existe aún implementación.
- **128–131 Prediction/reconciliation/history/resimulation:** implementar incrementalmente con ticks explícitos, snapshots autoritativos, inputs numerados e historial acotado; validar replay antes de afirmar determinismo.
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

## Serialización y wire format — punto 123

Este contrato fija una codificación de wire pequeña y explícita para el envelope definido en el punto 122. Es una decisión de diseño; no hay todavía encoder/decoder ni API pública implementados.

### Formato binario v1

Todos los enteros se codifican en **big-endian**. No se transmiten structs Rust en memoria, padding, discriminantes implícitos, `usize`, ni representaciones dependientes de la plataforma.

Cada frame en el byte stream tiene esta forma:

| Campo | Tamaño | Significado |
|---|---:|---|
| `frame_length` | 4 bytes | Longitud big-endian de todo lo que sigue, sin incluir este prefijo |
| `protocol_version` | 2 bytes | Versión del protocolo; `1` es la versión inicial definida por este formato |
| `message_type` | 2 bytes | Identificador estable de tipo; `0` es inválido/reservado |
| `message_version` | 2 bytes | Versión del esquema del mensaje; `0` es inválido/reservado |
| `payload` | 0 o más bytes | Bytes opacos del mensaje, definidos por el tipo y la versión |

`frame_length` cuenta los seis bytes de cabecera y el payload, pero no sus propios cuatro bytes. El mínimo válido es 6 bytes. El máximo de frame es **1 MiB (1.048.576 bytes)**, incluyendo la cabecera pero excluyendo el prefijo de longitud; por tanto, el payload máximo es 1.048.570 bytes. El tamaño total máximo en el stream es 1.048.580 bytes contando el prefijo. Estos límites son constantes del formato v1, no datos confiables enviados por el peer.

### Decodificación y framing

- El decoder lee exactamente los cuatro bytes del prefijo, aceptando lecturas parciales del stream mediante acumulación explícita.
- Valida `frame_length` contra el mínimo y máximo antes de reservar o acumular el cuerpo. Una longitud fuera de rango se rechaza; no se trunca, satura ni intenta una asignación basada en ella.
- Después lee exactamente el cuerpo declarado. EOF antes de completar prefijo o cuerpo se informa como frame truncado; no se entrega un mensaje parcial.
- El decoder valida la versión de protocolo, el tipo y la versión de mensaje antes de despachar el payload. La versión de protocolo no admitida, el tipo cero/desconocido y la versión de mensaje cero/no admitida se rechazan mediante errores diferenciados.
- Un frame válido consume exactamente el prefijo y el cuerpo declarados. Los bytes siguientes pertenecen al próximo frame; no se acepta que una longitud incluya bytes de un frame posterior.
- El encoder calcula la longitud con comprobación de overflow, rechaza mensajes que excedan el máximo y emite el prefijo y la cabecera en el orden definido. No puede depender del endianess nativo.
- El payload permanece opaco en la capa de wire. La decodificación de campos propios de cada mensaje se hará con reglas específicas y límites propios; no se habilita deserialización de tipos arbitrarios ni se interpreta memoria Rust directamente.
- No se define checksum, compresión, cifrado, autenticación, ACK, secuencias ni recuperación tras pérdida. Esas funciones requieren contratos separados y no deben inferirse de este formato.

### Errores mínimos de serialización

La futura API debe distinguir al menos: longitud de frame demasiado pequeña, longitud de frame excedida, overflow al calcular longitud, prefijo/cuerpo truncado, versión de protocolo no admitida, tipo inválido o desconocido y versión de mensaje inválida o no admitida. Los fallos de transporte subyacentes se conservan como fallos de transporte, no se disfrazan de errores de payload.

### Compatibilidad y evolución

El layout descrito es el wire format v1. Cambiar tamaño, orden, endianness o semántica de los campos de cabecera es un cambio incompatible y requiere una nueva versión de protocolo o una decisión explícita de migración. Los nuevos tipos usan identificadores no reutilizados; cambios incompatibles en un esquema de payload incrementan su versión de mensaje o reciben un nuevo identificador según la política del protocolo. No se permite adivinar versiones ni ignorar campos de cabecera desconocidos.

### Criterios de aceptación del punto 123

- Formato binario, orden de bytes, framing y significado de longitud definidos sin ambigüedad.
- Límites mínimo/máximo validados antes de asignaciones basadas en datos remotos.
- Lecturas parciales, EOF/truncamiento, overflow y errores de versión/tipo tienen comportamiento explícito.
- Se prohíbe depender de layout Rust nativo o añadir dependencias sin necesidad justificada.
- Se declara que este punto solo decide el contrato: la implementación del encoder/decoder y las pruebas de código se harán en puntos posteriores de implementación/testing.
