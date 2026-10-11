# DINAH — política de automatización y bots

## Autoridad sobre el repositorio

- Los bots pueden analizar, proponer cambios, crear ramas y abrir Pull Requests.
- Ningún bot puede fusionar Pull Requests ni habilitar auto-merge. El merge de cambios de código y configuración corresponde a una persona responsable del proyecto.
- Excepción operativa: el workflow de archivado de benchmarks conserva su comportamiento existente y puede registrar automáticamente los artefactos generados en `main`, una vez validados CI y benchmarks para el mismo commit. Esta excepción no autoriza a otros bots a escribir en `main`.
- No se conceden permisos de administración a bots. Cada integración debe usar los permisos mínimos que necesita.
- Los cambios generados por bots deben explicar el motivo, el alcance, los riesgos y las comprobaciones ejecutadas.
- El workflow de archivado de artefactos se mantiene automático: tras validar los workflows requeridos para el mismo SHA, guarda los artefactos bajo `artifacts/benchmarks/<commit-sha>/run-<benchmark-run-id>/` y registra los cambios en `main`.
- Una tarea del roadmap no se marca como completada sin evidencia revisable en código, documentación, pruebas o resultados.

## Validación

- Los cambios de código deben pasar la CI del proyecto antes del merge.
- Las actualizaciones de dependencias deben revisarse como cambios normales; no se habilita auto-merge.
- Si una comprobación falla o no se ha ejecutado, la PR debe indicarlo y no presentarse como lista para integrar.
- Los bots no deben modificar credenciales, secretos, permisos de workflows ni reglas de protección como parte de una PR ordinaria.

## Limitación actual de GitHub

El repositorio es privado y la API de GitHub informa que la protección de ramas y los rulesets requieren un plan superior o que el repositorio sea público. Hasta resolverlo, la política documentada no equivale a una barrera técnica impuesta por GitHub. Las credenciales de automatización no deben tener permiso de administración ni de escritura directa en `main`; cualquier configuración adicional se debe verificar por separado.

## Archivado de benchmarks

El workflow existente de archivado de resultados permanece sin cambios: espera a que CI y benchmarks hayan terminado correctamente para el mismo commit, descarga y organiza los paquetes y registra automáticamente los artefactos bajo `artifacts/benchmarks/<commit-sha>/run-<benchmark-run-id>/`. No convertir este archivado en una Pull Request ni exigir aprobación manual para cada ejecución.
