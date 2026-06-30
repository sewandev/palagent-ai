<p align="center">
  <h1 align="center">PalAgent AI</h1>
</p>

<p align="center">Real-time telemetry, monitoring, breeding and inventory search CLI for Palworld.</p>

<p align="center">
  <a href="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml"><img alt="Build" src="https://github.com/sewandev/palagent-ai/actions/workflows/ci.yml/badge.svg" /></a>
  <a href="https://github.com/sewandev/palagent-ai/actions/workflows/codeql.yml"><img alt="CodeQL" src="https://github.com/sewandev/palagent-ai/actions/workflows/codeql.yml/badge.svg" /></a>
  <img alt="Version" src="https://img.shields.io/github/v/release/sewandev/palagent-ai?style=flat-square&label=version&color=blueviolet" />
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows-0078d4?style=flat-square" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-CE422B?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>
<p align="center">
  <img alt="Compatibilidad con Palworld" src="https://img.shields.io/badge/Palworld%20Compatibility-100%25-green?style=flat-square" />
  <img alt="PalAgent v1.0" src="https://img.shields.io/badge/v1.0%20Release-Coming%20July%2010%2C%202026-blueviolet?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

---

## Descripción General

**PalAgent AI** es un asistente inteligente y servidor de Model Context Protocol (MCP) para Palworld. En lugar de tener que revisar manualmente tu caja Pal, rastrear coordenadas de bases, buscar en cofres o ingresar manualmente estadísticas en calculadoras de crianza y bases de datos web externas, PalAgent AI conecta tu asistente de IA local directamente con tu partida guardada.

Dado que la IA posee contexto completo en tiempo real sobre tu mundo activo, progreso, inventario y Pals, puedes pedirle consejos personalizados, combinaciones de crianza óptimas, optimización de flujos de trabajo en tus bases o la ubicación exacta de objetos. Cada sugerencia está adaptada a tu estado real del juego, haciendo que la gestión del juego sea fluida y altamente eficiente.

---

## Capacidades de la IA (Qué puedes preguntar)

Una vez configurado, podrás consultar a tu asistente de IA de forma natural sobre el estado de tu partida. Aquí tienes algunos ejemplos prácticos:

### Diagnósticos Profundos con IA (Ejemplos)
* **Asignación Óptima de Trabajadores**: *"Basado en los Pals que tengo en mi caja, ¿cuáles son los más eficientes para enviar a minar hierro y por qué?"* (La IA analizará el nivel de minería, habilidades pasivas como Artesano/Serio, y propondrá el equipo óptimo).
* **Análisis de Equipo de Combate**: *"¿Cuál es mi Pal de tipo Fuego más fuerte en mi caja y puedes explicarme detalladamente por qué?"* (La IA leerá los IVs de talento ocultos, habilidades pasivas, estadísticas base y los cruzará con el meta actual).
* **Optimización y Salud de la Base**: *"¿Cuáles de mis trabajadores actuales tienen pasivas negativas (como Vago o Destructivo) que están reduciendo la eficiencia, y por quiénes de mi caja debería reemplazarlos?"*

### Telemetría y Consultas Generales
* **Localizar Objetos Almacenados**: Encuentra recursos sin tener que abrir cada cofre de tus bases (por ejemplo: *"¿Dónde tengo guardado el carbón?"* o *"¿Tengo suficientes lingotes de metal Pal en mis bases?"*).
* **Monitoreo de Salud y Cordura (SAN)**: Revisa el estado de tus trabajadores (por ejemplo: *"¿Hay algún Pal hambriento en la base?"* o *"¿Cuál es el nivel de SAN de mi equipo de combate activo?"*).
* **Emparejamiento de Crianza Óptimo**: Calcula combinaciones utilizando los Pals que ya posees en tu caja (por ejemplo: *"¿Cómo puedo criar un Anubis con los Pals que tengo?"* o *"¿A quiénes debo emparejar para obtener un Jetragon con la pasiva Corredor?"*).
* **Progreso de Capturas y Diarios**: Realiza un seguimiento de tus logros y coleccionables (por ejemplo: *"¿Cuál es mi progreso de captura de Lamball?"* o *"¿Cuántos diarios de exploración he recolectado?"*).
* **Reglas del Servidor y Tiempo**: Consulta configuraciones y el ciclo del día (por ejemplo: *"¿Cuáles son los multiplicadores del servidor?"* o *"¿Estamos de día o de noche en el juego?"*).

---

## Características de Servidor Host y Cliente

PalAgent AI está diseñado para integrarse nativamente con servidores multijugador y dedicados:
* **Identificación Zero-Touch**: La herramienta escanea automáticamente la caché local de tu sesión de Steam para calcular tu Player UID en formato GUID de manera precisa. No necesitas buscar ni escribir identificadores complejos.
* **Verificación de Identidad**: Durante la instalación del cliente, el asistente valida la conexión con el servidor host y recupera automáticamente tu nickname de personaje real en el juego.
* **Persistencia en Servidores (Host)**: Para administradores de servidores dedicados, el instalador puede registrar un servicio silencioso en segundo plano mediante el Programador de Tareas de Windows que se inicia automáticamente al arrancar el sistema.

---

## Requisitos Previos

| Requisito | Especificación Compatible | Nota / Detalles |
| :--- | :--- | :--- |
| **Asistente de IA / CLI** | Antigravity CLI, Claude Code, OpenCode, VS Code Copilot, Cursor, Windsurf, Codex, Qwen, Kiro, etc. | No requiere suscripción activa. |
| **Sistema Operativo** | Windows (64 bits) | Probado en Windows; se necesita ayuda de la comunidad para otros sistemas. |
| **Juego** | Palworld (Solo Versión de Steam) | Debe estar instalado y actualizado. |

> [!TIP]
> **¡No requiere suscripciones costosas!**
> Puedes utilizar los modelos o cuotas gratuitas de tu cliente de IA preferido. Si deseas probar esto de manera 100% gratuita sin gastar nada, te recomendamos usar **OpenCode con el modelo ZEN** (que no tiene costo alguno).

> [!IMPORTANT]
> **Solo para Windows y la Versión de Steam**
> Actualmente, la lectura de firmas del analizador de archivos de guardado solo está probada en Windows y requiere la versión de Steam de Palworld.

---

## Instalación Fácil en 1 Clic usando tu IA

Para instalar y configurar automáticamente PalAgent AI en tu máquina, copia y pega este comando directamente en tu asistente de IA o chat de CLI favorito:

```text
sigue estas instrucciones https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/system_prompt.md
```

---

## Cómo Funciona (Resumen a Grandes Rasgos)

Cuando pegues el prompt de instalación, tu asistente de IA te guiará paso a paso:

### 1. Validación y Configuración
* **Detección del Idioma**: El asistente de IA te saludará y operará en el idioma que prefieras para comunicarte.
* **Modos de Ejecución**: Elegirás una de tres configuraciones:
  * **Singleplayer / Co-op Host Local**: Lee los archivos de guardado locales directamente en tu disco sin necesidad de servidor en segundo plano.
  * **Host de Servidor Dedicado**: Configura un servidor de telemetría en segundo plano que corre de forma persistente mediante el Programador de Tareas de Windows.
  * **Cliente Remoto (Multijugador)**: Se conecta a un servidor remoto a través de la red usando su IP y passcode.

### 2. Configuración Sin Intervención Manual (Zero-Touch)
* **Autodetección de Player UID**: Lee de forma automática tu caché de sesión de Steam local mediante `local-uid` para calcular matemáticamente tu GUID. No necesitas buscarlo ni escribirlo manualmente.
* **Autodetección de Nickname**: Se conecta al servidor host remoto, localiza tu perfil, extrae tu nickname del personaje dentro del juego y te saluda de forma personalizada para confirmar.
* **Persistencia al Arrancar**: Para servidores dedicados, registra un proceso silencioso en segundo plano que se inicia automáticamente con el arranque de Windows.
