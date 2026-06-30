Eres un asistente de codificación de IA. El usuario quiere que realices una instalación automática y sin intervención manual de "PalAgent AI" en su sistema local.

PalAgent AI es una herramienta de telemetría y análisis de archivos de guardado de Palworld de alto rendimiento escrita en Rust puro. Funciona como un servidor de Model Context Protocol (MCP), lo que permite a los asistentes de IA realizar consultas en tiempo real sobre los datos del juego (como niveles de los jugadores, Pals en el equipo activo, contenido de la caja Pal, combinaciones de crianza, coordenadas de las bases y el inventario de los cofres de almacenamiento).

Por favor, sigue estas instrucciones paso a paso:

1. PRIMERO, realiza las siguientes preguntas de aclaración al usuario en su idioma preferido (por defecto, el idioma en el que te está hablando):
   - Pregunta qué cliente CLI o IDE de desarrollo de IA está utilizando (clientes compatibles: antigravity-cli, vscode-copilot, cursor, windsurf, opencode, claude-code, gemini-cli, codex, qwen, kiro, pi, kilocode).
   - Pregunta qué modo de ejecución necesita:
     * **Singleplayer / Servidor Local (Local)**: Los archivos de guardado están en esta misma máquina local.
     * **Servidor Dedicado / Cliente Remoto (Remoto)**: Se conecta a una máquina host remota. Si elige esta opción, solicítale:
       - La IP y puerto del Host (por ejemplo, `192.168.1.100:8212`).
       - El Passcode de seguridad configurado en el host.
       - El Player UID privado del jugador (por ejemplo, `00000000-0000-0000-0000-000000000001`).
       - *Nota*: Recuérdale que la máquina remota debe tener `palagent-ai` ejecutándose en modo `--host` para que la conexión funcione.
       - *Cómo obtener el Player UID*: Si el usuario no conoce su Player UID, puedes buscarlo automáticamente realizando primero una consulta de cliente al host sin especificar un UID:
         `palagent-ai.exe --connect <HOST_IP_PORT> --passcode <PASSCODE>`
         Esto te devolverá el reporte completo de todos los jugadores. Busca el nickname del usuario en la lista para extraer su Player UID correspondiente. Alternativamente, el usuario puede solicitárselo al administrador del servidor.

2. Once you have the answers, download the latest release binary `palagent-ai.exe` from the official repository:
   - Consulta la API de GitHub en: https://api.github.com/repos/sewandev/palagent-ai/releases/latest
   - Analiza los assets, busca la URL de descarga de `palagent-ai.exe` y descárgalo en un directorio temporal del espacio de trabajo o ejecuta las herramientas correspondientes.

3. Ejecuta el comando de configuración correspondiente al cliente seleccionado por el usuario:
   - Comando: `palagent-ai.exe setup <agent-slug>`
   - Por ejemplo, si utiliza Cursor: `palagent-ai.exe setup cursor`
   - Espera a que se complete la configuración e inspecciona la salida en consola.

4. Configura los argumentos de ejecución MCP:
   - Para el **Modo Local**, la configuración se realiza automáticamente al ejecutar el instalador.
   - Para el **Modo Cliente Remoto**, modifica el archivo de configuración de servidores MCP registrado por el instalador. Debes añadir los argumentos necesarios para conectarse al host remoto. Por ejemplo:
     `"args": ["mcp", "--connect", "<HOST_IP_PORT>", "--passcode", "<PASSCODE>", "--player-uid", "<PLAYER_UID>"]`
     Verifica que el archivo de configuración (como `mcp.json` o `mcp_config.json` según el cliente) sea modificado correctamente.

5. Verifica que las configuraciones del servidor MCP y las habilidades (skills) se hayan instalado correctamente.
   - Comprueba que se haya creado el directorio de habilidades `~/.gemini/config/skills/palagent-ai` (para Antigravity) y que el archivo `SKILL.md` esté presente.

6. Finalmente, indícale al usuario que debe reiniciar o recargar su IDE/CLI para que se cargue el nuevo servidor MCP. Al mismo tiempo, sugiérele realizar una consulta rápida de prueba tan pronto como reinicie para verificar que el servidor esté funcionando (dándole ejemplos concretos de lo que puede preguntar, como el nickname de su personaje, el nombre de su mundo/partida activa, los primeros 5 slots de su inventario o su Pal con el nivel más alto).
