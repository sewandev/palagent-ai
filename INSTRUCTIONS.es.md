# Prompt de Instalación Asistida por IA para PalAgent AI

Copia y pega el bloque de texto (prompt) que aparece a continuación directamente en tu asistente de IA (como Antigravity CLI, Claude Code, Cursor, Windsurf, etc.) para automatizar todo el proceso de instalación y configuración.

***

```text
Eres un asistente de codificación de IA. El usuario quiere que realices una instalación automática y sin intervención manual de "PalAgent AI" (un servidor de telemetría y monitoreo MCP para Palworld) en su sistema local.

Por favor, sigue estas instrucciones paso a paso:

1. PRIMERO, realiza las siguientes preguntas de aclaración al usuario en su idioma preferido (por defecto, el idioma en el que te está hablando):
   - Pregunta qué cliente CLI o IDE de desarrollo de IA está utilizando (clientes compatibles: antigravity-cli, vscode-copilot, cursor, windsurf, opencode, claude-code, gemini-cli, codex, qwen, kiro, pi, kilocode).
   - Pregunta si actualmente tiene instalado Palworld en Windows y si tiene al menos una partida guardada o mundo activo.

2. Una vez que tengas las respuestas, descarga el ejecutable binario `palagent-ai.exe` más reciente desde el repositorio oficial:
   - Consulta la API de GitHub en: https://api.github.com/repos/sewandev/palagent-ai/releases/latest
   - Analiza los assets, busca la URL de descarga de `palagent-ai.exe` y descárgalo en un directorio temporal del espacio de trabajo o ejecuta las herramientas correspondientes.

3. Ejecuta el comando de configuración correspondiente al cliente seleccionado por el usuario:
   - Comando: `palagent-ai.exe setup <agent-slug>`
   - Por ejemplo, si utiliza Cursor: `palagent-ai.exe setup cursor`
   - Espera a que se complete la configuración e inspecciona la salida en consola.

4. Verifica que las configuraciones del servidor MCP y las habilidades (skills) se hayan instalado correctamente.
   - Por ejemplo, si utiliza Antigravity, comprueba que se haya creado el directorio de habilidades `~/.gemini/config/skills/palagent-ai` y que el archivo `SKILL.md` esté presente.

5. Finalmente, indícale al usuario que debe reiniciar o recargar su IDE/CLI para que se cargue el nuevo servidor MCP, e infórmale que ya puede preguntar por sus estadísticas de Palworld directamente (por ejemplo, consultando el estado de sus bases, los IVs de sus Pals, su equipo activo, etc.).
```
***
