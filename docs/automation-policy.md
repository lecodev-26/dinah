# DINAH — política de automatización y bots

## Autoridad sobre el repositorio

- Los bots pueden analizar, proponer cambios, crear ramas y abrir Pull Requests.
- Ningún bot puede fusionar Pull Requests, habilitar auto-merge ni escribir directamente en `main`.
- El merge final corresponde a una persona responsable del proyecto.
- No se conceden permisos de administración a bots. Cada integración debe usar los permisos mínimos que necesita.
- Los cambios generados por bots deben explicar el motivo, el alcance, los riesgos y las comprobaciones ejecutadas.
- Una tarea del roadmap no se marca como completada sin evidencia revisable en código, documentación, pruebas o resultados.

## Validación

- Los cambios de código deben pasar la CI del proyecto antes del merge.
- Las actualizaciones de dependencias deben revisarse como cambios normales; no se habilita auto-merge.
- Si una comprobación falla o no se ha ejecutado, la PR debe indicarlo y no presentarse como lista para integrar.
- Los bots no deben modificar credenciales, secretos, permisos de workflows ni reglas de protección como parte de una PR ordinaria.

## Limitación actual de GitHub

El repositorio es privado y la API de GitHub informa que la protección de ramas y los rulesets requieren un plan superior o que el repositorio sea público. Hasta resolverlo, la política documentada no equivale a una barrera técnica impuesta por GitHub. Las credenciales de automatización no deben tener permiso de administración ni de escritura directa en `main`; cualquier configuración adicional se debe verificar por separado.

## Archivado de benchmarks

El archivador de resultados debe preparar una rama y abrir una PR después de que CI y benchmarks hayan terminado correctamente para el mismo commit. No debe empujar los artefactos directamente a `main`. El merge de esa PR queda en manos humanas.
