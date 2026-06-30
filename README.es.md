<p align="center">
  <h1 align="center">PalAgent AI</h1>
</p>

<p align="center">Telemetría en tiempo real, monitoreo, cruces y buscador de inventario por CLI para Palworld.</p>

<p align="center">
  <a href="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml"><img alt="Build" src="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml/badge.svg" /></a>
  <a href="https://github.com/sewandev/palagent-ai/actions/workflows/codeql.yml"><img alt="CodeQL" src="https://github.com/sewandev/palagent-ai/actions/workflows/codeql.yml/badge.svg" /></a>
  <img alt="Version" src="https://img.shields.io/github/v/release/sewandev/palagent-ai?style=flat-square&label=version&color=blueviolet" />
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows-0078d4?style=flat-square" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-CE422B?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

---

> [!TIP]
> **Instalación fácil usando la misma IA (la IA hará todo por ti)**
>
> Copia y pega el prompt de `https://raw.githubusercontent.com/sewandev/palagent-ai/main/INSTRUCTIONS.es.md` directamente en tu asistente de IA.

## Instalación y Configuración

```bash
# Compilar desde el código fuente (requiere Rust)
git clone https://github.com/sewandev/palagent-ai.git
cd palagent-ai
cargo build --release

# Ejecutar el reporte de análisis
./target/release/palagent-ai.exe

# Registrar como Servidor MCP e instalar skills automáticamente en Antigravity CLI
./target/release/palagent-ai.exe setup antigravity-cli
```

> [!IMPORTANT]
> **Descompresión de guardado de Palworld**: Esta herramienta requiere la biblioteca de descompresión Oodle (`oo2core_9_win64.dll`) para descomprimir los archivos de guardado GVAS de Palworld. La herramienta la busca automáticamente en el directorio de instalación del juego. Si no la encuentra, cópiala al lado del ejecutable compilado.

---

## Funcionalidades

- **Auto-detección Multi-mundo y Menú Interactivo** — Escanea automáticamente tu LocalAppData para encontrar el archivo de guardado de Palworld más recientemente actualizado. Si existen varios mundos, ejecútalo con `--select-world` para elegir interactivamente.
- **Resiliencia al Modo de Juego** — Detecta automáticamente si la partida guardada es un mundo **Singleplayer** (un jugador), una partida cooperativa **Co-op Multiplayer** o un **Servidor Dedicado**.
- **Sincronización en tiempo real Servidor/Cliente** — Permite que los clientes se conecten remotamente para consultar sus propias estadísticas desde la máquina Host sin necesidad de tener acceso directo a los archivos físicos.
- **Aislamiento y Privacidad del Cliente** — En modos multijugador/host, los jugadores pueden aislar sus reportes utilizando su `--player-uid`, protegiendo los inventarios y Pals de los demás jugadores de miradas no autorizadas.
- **Lectura de Palbox y Campamentos Base** — Extracción completa de los Pals almacenados en la Palbox offline, detalles de ubicación de Campamentos Base (coordenadas, nivel, gremio propietario) e integrantes de Gremios (miembros, líder).
- **Rendimiento de Subsegundo** — Escrito en Rust puro. Utiliza escaneo de firmas de bytes directamente sobre la memoria en lugar de parsear un pesado AST JSON, ejecutando cualquier consulta en menos de 1 segundo.
- **Salidas JSON para LLMs** — Cada comando puede exportar datos JSON formateados agregando `--json`, ideal para alimentar asistentes de desarrollo de IA.

---

## Modos de Ejecución

### 1. Modo Singleplayer (Local)
No requiere configuración ni servidores de red. Accede directamente a tus partidas locales guardadas.
```bash
# Reporte general
palagent-ai.exe

# Buscar "Bayas" en todos los cofres de las bases
palagent-ai.exe --search-chest "Berries"

# Monitorear estado de los Pals asignados a bases (SAN, Hambre, HP)
palagent-ai.exe --monitor
```

### 2. Modo Host (Servidor de Sincronización)
Levanta un servidor HTTP ultraligero en el puerto `8212` para transmitir de forma segura la telemetría a los clientes conectados.
```bash
# Iniciar servidor host con contraseña de seguridad
palagent-ai.exe --host --port 8212 --passcode CLAVE_SEGURA
```

### 3. Modo Cliente (Telemetría Remota)
Consulta información remotamente desde el servidor del Host. Filtra automáticamente todos los resultados por tu UID de jugador.
```bash
# Conectarse y obtener tu reporte privado de jugador
palagent-ai.exe --connect 192.168.1.100:8212 --passcode CLAVE_SEGURA --player-uid <TU_UID>

# Consultar el progreso de misiones activas de manera remota en formato JSON
palagent-ai.exe --connect 192.168.1.100:8212 --passcode CLAVE_SEGURA --player-uid <TU_UID> --progress --json
```

---

## Referencia de Comandos

| Flag | Subcomando | Descripción |
| --- | --- | --- |
| `-t`, `--time` | Hora del Juego | Día actual, reloj de juego y estado de día/noche. |
| `-s`, `--settings` | Configuraciones | Configuración del servidor y dificultad de la partida. |
| `-c`, `--search-chest` | Buscador de Cofres | Ubicación de ítems específicos en todos los cofres de las bases. |
| `-b`, `--breeding` | Asistente de Crianza | Muestra los Pals del equipo y calcula sus posibles combinaciones de cruce. |
| `-p`, `--progress` | Métricas de Progreso | Total de notas leídas, puntos de viaje rápido y bonus de capturas. |
| `-m`, `--monitor` | Monitor de Base | HP, saciación, cordura (SAN) y estado físico de tus Pals en bases. |
| `-a`, `--analyzer` | Analizador de IVs | Nivel, género, habilidades pasivas y estadísticas de talento/IVs (HP, Atk, Def). |
| `--list-worlds` | Directorio de Partidas | Lista todos los mundos locales detectados con sus fechas de actualización. |
