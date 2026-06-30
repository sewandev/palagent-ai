Eres un asistente de codificación de IA. El usuario quiere que realices una instalación automática y sin intervención manual de "PalAgent AI" en su sistema local.

PalAgent AI es una herramienta de telemetría y análisis de archivos de guardado de Palworld de alto rendimiento escrita en Rust puro. Funciona como un servidor de Model Context Protocol (MCP), lo que permite a los asistentes de IA realizar consultas en tiempo real sobre los datos del juego (como niveles de los jugadores, Pals en el equipo activo, contenido de la caja Pal, combinaciones de crianza, coordenadas de las bases y el inventario de los cofres de almacenamiento).

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

5. Indícale al usuario que debe reiniciar o recargar su IDE/CLI para que se cargue el nuevo servidor MCP.

6. PROPÓN UNA CONSULTA DE PRUEBA: Una vez que el reinicio se haya completado, sugiérele al usuario realizar una consulta rápida de prueba para verificar que el servidor MCP esté funcionando correctamente. Dale ejemplos concretos de lo que puede preguntar, tales como:
   - "¿Cuál es el nickname de mi personaje?"
   - "¿Cuál es el nombre de mi mundo/partida activa?"
   - "¿Qué objetos tengo en los primeros 5 slots de mi inventario?"
   - "¿Cuál es mi Pal con el nivel más alto?"
   Ofrécete a ejecutar la consulta de herramienta MCP correspondiente de inmediato si el usuario está de acuerdo, y muestra el resultado obtenido.
