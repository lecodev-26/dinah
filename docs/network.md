# dinah Networking — diseño de API y límites de v0.1

Estado: diseño inicial (punto 120). Este documento fija las fronteras y contratos para que los siguientes puntos puedan implementar transporte, mensajes y conexiones sin comprometer prematuramente una API de red demasiado amplia. No declara que haya sockets, serialización ni conexiones implementados.

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

Esta firma es un punto de discusión, no una promesa de API final. Antes de implementarla hay que decidir si el contrato es stream o datagrama, cómo se representa `WouldBlock`, cómo se distinguen EOF y desconexión, si la lectura puede ser parcial y quién posee los buffers. No se debe tratar una llamada `send` de stream como si enviara un mensaje completo.

Capas previstas:

1. **Mensaje lógico:** tipo/versión de mensaje y payload, sin acoplarlo a TCP, UDP ni a una estructura Rust en memoria.
2. **Serialización:** codifica/decodifica bytes con límites explícitos, validación de longitud y errores tipados. Nunca confiar en tamaños proporcionados por el peer.
3. **Transporte:** entrega bytes y declara semántica de stream/datagrama, bloqueo, orden, errores y cierre.
4. **Conexión y endpoint:** identidad local/remota y estado de conexión, sin confundir dirección, sesión autenticada y autorización.
5. **Integración:** adaptadores explícitos hacia Runtime y Simulation; ninguna mutación oculta ni callback de fondo implícito.

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

- **121 Transporte:** elegir primero un contrato concreto (preferencia inicial: transporte de stream bloqueante explícito para la primera implementación, salvo que el caso de uso exija datagramas). Especificar parcialidad, EOF, timeouts y errores.
- **122 Mensajes:** decidir envelope/versionado, tipos de mensaje y límites.
- **123 Serialización:** elegir formato acotado, con versiones y errores; medirlo antes de introducir dependencias.
- **124–126 Conexiones/endpoints/estado:** definir ownership, estados y cierre antes de añadir recuperación.
- **127–131 Prediction/reconciliation/history/resimulation:** documentar primero el modelo temporal, snapshots, autoridad, orden de inputs y límites de memoria; no declarar determinismo de red como garantía de transporte.
- **132–134 Pérdida/latencia/desconexión:** construir un simulador controlado y tests deterministas antes de implementar políticas de producción.
- **135–136 Integración Runtime/Simulation:** solo mediante adaptadores explícitos, sin acoplamiento circular ni efectos secundarios ocultos.
- **137–139 Tests/latencia/benchmarks:** tests unitarios con transportes falsos, escenarios de integración locales y benchmarks reproducibles con entorno archivado.
- **140 Documentación:** documentar exclusivamente comportamiento implementado y límites medidos.

## Criterios de aceptación del punto 120

- Responsabilidades separadas de Core, Runtime, Simulation y Networking.
- Capas lógicas de mensaje, serialización, transporte, conexión e integración identificadas.
- Contratos iniciales de ownership, framing, límites, errores, tiempo, concurrencia, cierre, seguridad y determinismo definidos.
- Decisiones no resueltas etiquetadas como propuestas, no como APIs existentes.
- Ningún transporte, socket, serializador o característica de predicción se considera implementado por este documento.
