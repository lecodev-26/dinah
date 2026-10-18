# DINAH — Revisión inicial de seguridad (puntos 195–198)

Fecha de revisión: 2026-10-18. Alcance: superficie actual del crate, lector de manifiestos del CLI, formatos de salida JSON y ABI C escalar. Esta revisión estática no es una auditoría externa ni demuestra ausencia de vulnerabilidades.

## 195 — Superficie de ataque

- El crate no declara dependencias de runtime en `[dependencies]`; la dependencia `wide` se usa solo como dependencia de desarrollo. Esto reduce la superficie de dependencias, pero no sustituye la auditoría periódica de lockfile/toolchain.
- `dinah init` usa `create_new(true)` y no reemplaza un manifiesto existente. Las operaciones de archivos se limitan al path que el usuario proporciona o al directorio actual; no se ejecutan comandos definidos en el manifiesto.
- `dinah run` solo valida el manifiesto. `dinah task` enumera claves; ninguna de las dos ejecuta las tareas declaradas.
- La entrada del manifiesto queda limitada a 1 MiB. No se afirma protección general frente a enlaces simbólicos, carreras del filesystem o paths maliciosos en todos los sistemas operativos.

## 196 — Entradas externas

- `DINAH.toml` se lee como UTF-8; errores de lectura se devuelven como errores de E/S. El parser soporta una sintaxis deliberadamente reducida y no es compatible con TOML completo.
- Se aplica un límite de 1 MiB antes de cargar el contenido completo. Los nombres de proyecto vacíos o no entrecomillados se rechazan.
- Las opciones desconocidas, formatos desconocidos y argumentos posicionales sobrantes se rechazan con salida no cero.
- Riesgo residual: el parser básico no valida todas las reglas TOML ni normaliza rutas; no debe usarse como validador de seguridad de configuraciones complejas.

## 197 — Serialización

- La salida JSON del CLI escapa comillas, barras inversas y saltos de línea; los controles restantes se sustituyen por espacios. La nueva exportación de métricas produce un array JSON determinista desde el snapshot ordenado y escapa nombres arbitrarios de métricas.
- No hay deserializador JSON ni decoder de wire format de networking declarado como implementado en esta revisión. No se debe asumir que un formato documentado ya tiene encoder/decoder.
- Riesgo residual: la exportación de métricas no fija un esquema de compatibilidad versionado; los consumidores deben tratarla como formato de diagnóstico local de esta versión.

## 198 — FFI

- `include/dinah.h` y `dinah::native::ffi` exponen solo consultas escalares de enteros de ancho fijo: versión ABI, flags y familia de plataforma. No aceptan punteros, buffers, callbacks ni ownership transferido.
- El crate mantiene `#![deny(unsafe_code)]`; el módulo FFI solo permite nombres de símbolos y no contiene bloques unsafe.
- El consumidor debe comprobar `dinah_abi_version()` antes de usar otras funciones. Los bits no definidos en `dinah_abi_features()` están reservados.
- Riesgo residual: sigue pendiente un smoke test con compilador y linker C reales; la revisión del header y pruebas Rust no sustituyen esa integración.

## Acciones pendientes

- Añadir fuzzing para parser de manifiesto, futuros codecs de networking y formatos persistidos.
- Añadir pruebas de corrupción, estrés y concurrencia, y ejecutar una auditoría formal de dependencias.
- Ejecutar el smoke test C/link en CI antes de declarar estable el ABI.
- Repetir esta revisión cuando se introduzcan ejecución de tareas, deserialización de red, acceso a archivos externos o APIs FFI basadas en punteros.
