<p align="center">
  <h1 align="center">PalAgent AI</h1>
</p>

<p align="center">CLI y Servidor MCP en tiempo real para telemetría, monitoreo, crianza y búsqueda en inventarios de Palworld.</p>

<p align="center">
  <a href="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml"><img alt="Build" src="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml/badge.svg" /></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows-0078d4?style=flat-square" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-CE422B?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

---

## 🚀 Descripción General

**PalAgent AI** conecta tu asistente de IA local (Antigravity, Cursor, VS Code Copilot, Windsurf, Claude Code, Gemini CLI, etc.) directamente con tus partidas guardadas de Palworld.

Al exponer datos en tiempo real de tu mundo, bases, inventarios y caja Pal mediante el **Model Context Protocol (MCP)**, tu asistente puede analizar estadísticas de combate, calcular rutas de crianza óptimas, estimar probabilidades de captura, localizar objetos en cofres y ayudarte a gestionar eficientemente tus campamentos desde la ventana del chat.

---

## 🛠️ Opciones de Instalación

Elige el método de instalación que prefieras:

| Método | Dificultad | Comando | Detalles |
| :--- | :--- | :--- | :--- |
| **1. Asistente de IA (Chat)** | 🟢 **Súper Fácil** | Pega este prompt en el chat de tu IA:<br>`sigue estas instrucciones https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/system_prompt.md` | Tu asistente de desarrollo clonará, compilará y configurará la base de datos y el servidor MCP de forma automática en el chat. |
| **2. PowerShell Interactivo** | 🟡 **Rápido y Automático** | Ejecuta en PowerShell:<br>`Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12; iex (New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/sewandev/palagent-ai/main/install.ps1')` | Compila el binario en modo release, vincula la DLL de Oodle, registra el comando `/palworld` en tu consola y ofrece un menú para configurar tus editores. |
| **3. Compilación Manual (Rust)** | 🔴 **Avanzado** | `git clone https://github.com/sewandev/palagent-ai`<br>`cargo build --release`<br>`target/release/palagent-ai setup <editor>` | Clona, compila el binario manualmente en Rust, resuelve la DLL `oo2core_9_win64.dll`, genera la base de datos SQLite y configura los clientes MCP. |

> [!TIP]
> **Comando de PowerShell Integrado**:
> Si instalas con la **Opción 2**, el script registrará un comando global `/palworld` en tu perfil de PowerShell. Podrás escribir `/palworld help`, `/palworld analyzer`, `/palworld monitor` o `/palworld breeding` directamente en cualquier terminal.

---

## 🧬 Capacidades de la IA y Prompts

Una vez configurado el servidor MCP en tu editor, tu asistente de IA podrá acceder a los datos de tu partida en tiempo real para responder a preguntas como:

> [!NOTE]
> * **Asignación Óptima**: *"¿Cuáles de mis Pals en la caja son los más eficientes para minar metal en la base?"* (La IA lee estadísticas de trabajo, habilidades pasivas como Artesano/Serio y propone el mejor equipo).
> * **Emparejamiento de Crianza**: *"¿Cómo puedo criar un Anubis? Revisa si tengo a los padres necesarios en mi caja Pal."* (La IA calcula los cruces, comprueba excepciones y busca compatibilidades en tu partida activa).
> * **Porcentaje de Captura**: *"¿Cuál es mi probabilidad de capturar a un Chillet nivel 33 usando una Megasfera con mi nivel actual de estatua Lifmunk?"*
> * **Rastreo de Objetos**: *"¿En qué cofre de mi base guardé los Fragmentos de Paldio? ¿Tengo suficiente madera para una Megaesfera?"*
> * **Monitoreo**: *"¿Hay algún Pal en la base campamento que esté deprimido o hambriento?"*

---

## 📦 Referencia de Herramientas y Base de Datos

La base de datos SQLite local se genera en el primer inicio y expone las siguientes herramientas del Model Context Protocol (MCP):

*   **`list_worlds`**: Lista todas las rutas de guardado locales detectadas.
*   **`query_time`**: Ciclos de día y noche de la partida.
*   **`query_settings`**: Multiplicadores de dificultad y configuraciones globales del servidor.
*   **`search_chest`**: Busca objetos en todos los contenedores y cofres del jugador.
*   **`query_breeding`**: Sugerencias de crianza dinámica basadas en los Pals de tu caja.
*   **`query_target_breeding`**: Encuentra los combos de padres para obtener un Pal específico.
*   **`query_progress`**: Progreso de viajes rápidos, diarios de diarios y efigies de Lifmunk.
*   **`monitor_pals`**: Monitoreo de HP, hambre y nivel de cordura (SAN) en tiempo real.
*   **`query_recipes`**: Materiales y costes para fabricar esferas y estructuras.
*   **`query_active_skills`**: Cooldown, elemento y daño de las habilidades de combate activas.
*   **`query_drops`**: Tabla y probabilidad de obtención de objetos al derrotar o capturar Pals.

---

## ⚖️ Licencia

Distribuido bajo la Licencia MIT. Consulta [LICENSE](LICENSE) para más información.
