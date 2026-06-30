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
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

---

## Requisitos Previos

Para utilizar PalAgent AI, asegúrate de cumplir con los siguientes requerimientos:

1. **Cliente de Asistente de IA o Plataforma CLI**: Debes tener instalado un entorno de desarrollo guiado por IA. Algunos ejemplos:
   - Antigravity CLI (`agy`)
   - Claude Code
   - OpenCode
   - VS Code Copilot
   - Cursor
   - Windsurf
   - Codex, Qwen, Kiro, etc.
   *Nota: No requiere suscripciones costosas. Puedes usar modelos que ofrezcan cuotas gratuitas o tokens de prueba en tu CLI favorita. Si quieres probar esto completamente gratis sin gastar nada, te recomendamos usar **OpenCode con el modelo ZEN**, que es gratuito.*

2. **Sistema Operativo**: **Windows** (64 bits).
   *Nota: Aunque Rust es multi-plataforma (Multi-OS), PalAgent AI actualmente solo está probado en Windows. ¡La ayuda de la comunidad para testear y dar soporte a otras plataformas es bienvenida!*

3. **Juego y Plataforma**:
   - **Palworld** debe estar instalado y actualizado.
   - Por el momento, solo se soporta la versión de **Steam**.

---

## Instalación Fácil en 1 Clic usando tu IA

Para instalar y configurar PalAgent AI de forma automática, simplemente copia y pega el siguiente prompt directamente en tu asistente de IA o chat de CLI favorito:

```text
sigue estas instrucciones https://raw.githubusercontent.com/sewandev/palagent-ai/main/instructions/system_prompt.md
```

### Qué hará este instructivo (a grandes rasgos):
1. **Configuración Interactiva del Idioma**: El asistente de IA te preguntará primero en qué idioma prefieres comunicarte.
2. **Preguntas de Aclaración**: El asistente te preguntará por tu editor/IDE de IA y el modo de juego:
   - **Singleplayer / Co-op Host Local**: Si juegas solo o en partidas cooperativas temporales en tu propia computadora.
   - **Host de Servidor Dedicado**: Si tienes un servidor dedicado 24/7 montado en tu máquina y deseas ejecutar el servidor de telemetría de forma persistente en segundo plano.
   - **Cliente Remoto (Multijugador)**: Si juegas en un servidor alojado por un amigo o máquina remota.
3. **Instalación Automatizada**:
   - Descarga automáticamente el ejecutable `palagent-ai.exe` más reciente.
   - Copia el archivo a una ubicación permanente y registra el servidor MCP correspondiente en tu cliente.
   - Detecta de forma automática tu Player UID real (ejecutando `local-uid` para calcularlo matemáticamente a partir de tu sesión activa de Steam).
   - Si montas un servidor dedicado, crea y registra una tarea en el Programador de Tareas de Windows para asegurar el inicio persistente del servidor en segundo plano.
   - Valida los datos y la conexión con el servidor para descubrir tu nickname real y darte la bienvenida.
