# PalAgent AI Agent Integration & Setup

This guide explains how to register the PalAgent AI MCP server and its system instructions across various AI-native development interfaces and command-line agents.

## Automated Installation

You can automatically set up PalAgent AI for any supported agent using the following command:

```bash
palagent-ai.exe setup <agent-slug>
```

Replace `<agent-slug>` with one of the supported agents listed below.

---

## Supported Agents

* [Antigravity CLI](#antigravity-cli)
* [VS Code Copilot](#vscode-copilot)
* [Cursor](#cursor)
* [Windsurf](#windsurf)
* [OpenCode](#opencode)
* [Claude Code](#claude-code)
* [Gemini CLI](#gemini-cli)
* [Codex](#codex)
* [Qwen Code](#qwen)
* [Kiro IDE](#kiro)
* [Pi](#pi)
* [Kilo Code](#kilocode)

---

### Antigravity CLI

To set up automatically:
```bash
palagent-ai.exe setup antigravity-cli
```

**MCP Configuration File**: `~/.gemini/config/mcp_config.json`  
**Instructions Surface**: `~/.gemini/config/AGENTS.md` and `~/.gemini/GEMINI.md`  
**Format**: `mcpServers` object  

---

### VS Code Copilot

To set up automatically:
```bash
palagent-ai.exe setup vscode-copilot
```

**MCP Configuration File**: `%APPDATA%/Code/User/mcp.json`  
**Instructions Surface**: `%APPDATA%/Code/User/prompts/palagent-ai.instructions.md`  
**Format**: `servers` object  

---

### Cursor

To set up automatically:
```bash
palagent-ai.exe setup cursor
```

**MCP Configuration File**: `~/.cursor/mcp.json`  
**Instructions Surface**: `~/.cursor/palagent-ai-rules.md`  
**Format**: `mcpServers` object  

---

### Windsurf

To set up automatically:
```bash
palagent-ai.exe setup windsurf
```

**MCP Configuration File**: `~/.codeium/windsurf/mcp_config.json`  
**Instructions Surface**: `~/.codeium/windsurf/memories/global_rules.md`  
**Format**: `mcpServers` object  

---

### OpenCode

To set up automatically:
```bash
palagent-ai.exe setup opencode
```

**MCP Configuration File**: `~/.config/opencode/opencode.json`  
**Instructions Surface**: `~/.config/opencode/AGENTS.md`  
**Format**: `mcp` object  

---

### Claude Code

To set up automatically:
```bash
palagent-ai.exe setup claude-code
```

**MCP Configuration File**: `~/.claude/settings.json`  
**Format**: `mcpServers` object  

---

### Gemini CLI

To set up automatically:
```bash
palagent-ai.exe setup gemini-cli
```

**MCP Configuration File**: `~/.gemini/settings.json`  
**Instructions Surface**: `~/.gemini/system.md`  
**Format**: `mcpServers` object  

---

### Codex

To set up automatically:
```bash
palagent-ai.exe setup codex
```

**MCP Configuration File**: `~/.codex/config.toml`  
**Instructions Surface**: `~/.codex/palagent-ai-instructions.md`  
**Format**: TOML Append  

---

### Qwen Code

To set up automatically:
```bash
palagent-ai.exe setup qwen
```

**MCP Configuration File**: `~/.qwen/settings.json`  
**Instructions Surface**: `~/.qwen/QWEN.md`  
**Format**: `mcpServers` object  

---

### Kiro IDE

To set up automatically:
```bash
palagent-ai.exe setup kiro
```

**MCP Configuration File**: `~/.kiro/settings/mcp.json`  
**Instructions Surface**: `~/.kiro/steering/palagent-ai.md`  
**Format**: `mcpServers` object  

---

### Pi

To set up automatically:
```bash
palagent-ai.exe setup pi
```

**MCP Configuration File**: `~/.pi/config/mcp.json`  
**Format**: `mcpServers` object  

---

### Kilo Code

To set up automatically:
```bash
palagent-ai.exe setup kilocode
```

**MCP Configuration File**: `~/.config/kilo/opencode.json`  
**Instructions Surface**: `~/.config/kilo/AGENTS.md`  
**Format**: `mcp` object  
