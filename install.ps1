$ErrorActionPreference = "Stop"

function Get-PalworldInstallPath {
    $steamPath = $null
    try {
        $steamPath = Get-ItemPropertyValue -Path "HKCU:\Software\Valve\Steam" -Name "SteamPath" -ErrorAction SilentlyContinue
    } catch {}
    if (-not $steamPath) {
        try {
            $steamPath = Get-ItemPropertyValue -Path "HKLM:\Software\Wow6432Node\Valve\Steam" -Name "InstallPath" -ErrorAction SilentlyContinue
        } catch {}
    }
    
    $libraries = [System.Collections.Generic.List[string]]::new()
    if ($steamPath -and (Test-Path $steamPath)) {
        $libraries.Add($steamPath)
        
        $vdfPath = Join-Path $steamPath "steamapps\libraryfolders.vdf"
        if (Test-Path $vdfPath) {
            $vdfContent = Get-Content -Path $vdfPath -Raw -ErrorAction SilentlyContinue
            if ($vdfContent) {
                $matches = [regex]::Matches($vdfContent, '"path"\s+"([^"]+)"')
                foreach ($match in $matches) {
                    $path = $match.Groups[1].Value.Replace("\\", "\")
                    if ($path -and (Test-Path $path) -and -not $libraries.Contains($path)) {
                        $libraries.Add($path)
                    }
                }
            }
        }
    }

    $backupDrives = @("C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z")
    foreach ($drive in $backupDrives) {
        $standardPath = "${drive}:\Program Files (x86)\Steam"
        if ((Test-Path $standardPath) -and -not $libraries.Contains($standardPath)) {
            $libraries.Add($standardPath)
        }
        $libraryPath = "${drive}:\SteamLibrary"
        if ((Test-Path $libraryPath) -and -not $libraries.Contains($libraryPath)) {
            $libraries.Add($libraryPath)
        }
    }

    foreach ($lib in $libraries) {
        $palworldPath = Join-Path $lib "steamapps\common\Palworld"
        if (Test-Path $palworldPath) {
            return $palworldPath
        }
    }
    return $null
}

$repo = "sewandev/palagent-ai"
$installDir = Join-Path $env:USERPROFILE ".palagent-ai"
$destExe = Join-Path $installDir "palagent-ai.exe"
$destDll = Join-Path $installDir "oo2core_9_win64.dll"

# Detect previous installation
$hasPreviousInstall = Test-Path $destExe

# --- LOCALIZATION STRINGS ---
$msgs = @{
    en = @{
        title = "PalAgent AI Manager"
        selectLang = "Please select your language / Seleccione su idioma:"
        prevInstall = "      [!] Previous installation detected at: {0}"
        mainMenu = "Please select an option:"
        optInstall = "  1. Install / Upgrade PalAgent AI"
        optUninstall = "  2. Uninstall and clean up all residues"
        optExit = "  3. Exit"
        enterChoice = "Enter your choice (1-3)"
        exiting = "Exiting installer."
        
        uninstallHeader = "=== Starting Uninstallation & Cleanup ==="
        unPSProfile = "[-] Cleaning up PowerShell profile helper..."
        unProfileRemoved = "      Removed /palworld helper from {0}"
        unProfileErr = "      Could not clean profile file: {0}"
        unMCPConfigs = "[-] Removing MCP server config registrations..."
        unMCPRemoved = "      Removed 'palagent-ai' registration from {0}"
        unMCPErr = "      Failed to clean MCP config {0}: {1}"
        unSkills = "[-] Deleting agent skill templates..."
        unSkillDeleted = "      Deleted skill directory: {0}"
        unBinaries = "[-] Deleting permanent binaries directory..."
        unBinFolderDeleted = "      Deleted installation folder: {0}"
        unBinLocked = "      Some files were locked. Attempting to delete individual files..."
        unBinFolderMarked = "      Binary folder marked for deletion on next boot or release."
        unCompleted = "     Uninstallation and cleanup completed!"
        
        installHeader = "=== Starting Installation & Upgrade ==="
        stepExecutable = "[1/4] Resolving PalAgent AI executable..."
        localDevBuild = "      Detected local development build. Copying from: {0}"
        downloadingGitHub = "      Downloading latest precompiled release from GitHub..."
        errNoAsset = "Error: No precompiled executable was found in the latest GitHub release."
        downloadingRelease = "      Downloading {0} ({1})..."
        errDownload = "Failed to resolve or download the precompiled release from GitHub: {0}"
        errInternet = "Ensure you are connected to the internet and try again."
        stepCopy = "[2/4] Installing executable to user profile..."
        binLocked = "      Binary is locked. Renaming old file to resolve lock..."
        binInstalled = "      PalAgent AI binary installed to: {0}"
        stepOodle = "[3/4] Resolving oo2core_9_win64.dll for GVAS decompression..."
        warnNoPalworld = "      [!] Warning: Palworld (Steam version) was not detected in your Steam libraries."
        warnNoPalworldSave = "          Ensure Palworld is installed on Steam on this PC, as PalAgent AI requires local save files to function."
        oodleResolved = "      oo2core_9_win64.dll is already resolved."
        oodleCopied = "      Found and copied oo2core_9_win64.dll from: {0}"
        warnNoOodle = "      Warning: oo2core_9_win64.dll not found in standard Steam paths."
        warnOodleManual = "      You might need to copy it manually to: {0}"
        
        stepConfig = "=== Configure Game Type & Client ==="
        gameChoiceHeader = "Select your Palworld setup:"
        gameChoice1 = "  1. Singleplayer / Co-op Host (Local game on this PC)"
        gameChoice2 = "  2. Dedicated Server Host (Admin of a dedicated server on this PC)"
        gameChoice3 = "  3. Remote Client (You play on a remote server/friend's dedicated host)"
        gameChoiceEnter = "Select game type (1-3)"
        
        dedicatedHeader = "=== Dedicated Server Host Configuration ==="
        dedicatedPasscode = "Set a secure server access passcode (or press Enter for none)"
        dedicatedTask = "Registering background server service via Windows Task Scheduler..."
        dedicatedTaskDone = "Dedicated server service registered to launch on boot."
        
        remoteHeader = "=== Remote Client Configuration ==="
        remoteUrl = "Enter the host server IP and Port (e.g. 192.168.1.50:8080)"
        remotePasscode = "Enter the server access passcode"
        remoteUrlErr = "Error: Host URL is required for remote client configuration."
        
        mcpHeader = "=== Configure Developer Integrations (MCP) ==="
        mcpSelect = "Select the developer environments to configure the MCP server for (separate with commas, e.g. 1, 3):"
        mcpOptAll = "  A. Configure ALL available environments"
        mcpOptSkip = "  S. Skip MCP configuration"
        mcpChoiceEnter = "Your selection"
        mcpConfiguring = "Configuring MCP for {0}..."
        mcpSkipped = "Skipping MCP client setup."
        
        completeHeader = "         Installation completed successfully!         "
        completeFolder = "  [+] Executable folder: {0}"
        completeModeHost = "  [+] Telemetry server: Registered on boot (Task Scheduler)"
        completeModeRemote = "  [+] Client connection: Pointed to {0}"
        completeModeLocal = "  [+] Mode: Local Singleplayer / Co-op"
        completeNextSteps = "  NEXT STEPS:"
        completeStep1 = "  1. Restart your chosen AI editor / CLI chat window."
        completeStep2 = "  2. Start asking live questions to test the connection:"
        completeQ1 = "     - 'What are my Pals IVs?'"
        completeQ2 = "     - 'Where in my base did I save the Wood?'"
        completeQ3 = "     - 'How can I breed an Anubis with my current Pals?'"
        completeQ4 = "     - 'Are there any Pals depressed or hungry in my camp?'"
    }
    es = @{
        title = "Gestor de PalAgent AI"
        selectLang = "Seleccione su idioma / Please select your language:"
        prevInstall = "      [!] Se detectó una instalación previa en: {0}"
        mainMenu = "Seleccione una opción:"
        optInstall = "  1. Instalar / Actualizar PalAgent AI"
        optUninstall = "  2. Desinstalar y limpiar todos los residuos"
        optExit = "  3. Salir"
        enterChoice = "Ingrese su opción (1-3)"
        exiting = "Saliendo del instalador."
        
        uninstallHeader = "=== Iniciando Desinstalación y Limpieza ==="
        unPSProfile = "[-] Limpiando el asistente de perfil de PowerShell..."
        unProfileRemoved = "      Se eliminó el asistente /palworld de {0}"
        unProfileErr = "      No se pudo limpiar el perfil: {0}"
        unMCPConfigs = "[-] Removiendo registros de configuración de servidores MCP..."
        unMCPRemoved = "      Se eliminó el registro 'palagent-ai' de {0}"
        unMCPErr = "      Error al limpiar configuración MCP {0}: {1}"
        unSkills = "[-] Eliminando plantillas de skills del agente..."
        unSkillDeleted = "      Directorio de skill eliminado: {0}"
        unBinaries = "[-] Eliminando directorio permanente de binarios..."
        unBinFolderDeleted = "      Carpeta de instalación eliminada: {0}"
        unBinLocked = "      Algunos archivos están bloqueados. Intentando eliminarlos individualmente..."
        unBinFolderMarked = "      Carpeta marcada para eliminación en el próximo arranque."
        unCompleted = "     ¡Desinstalación y limpieza completadas con éxito!"
        
        installHeader = "=== Iniciando Instalación y Actualización ==="
        stepExecutable = "[1/4] Resolviendo el ejecutable de PalAgent AI..."
        localDevBuild = "      Se detectó una compilación de desarrollo local. Copiando desde: {0}"
        downloadingGitHub = "      Descargando la última release precompilada desde GitHub..."
        errNoAsset = "Error: No se encontró ningún ejecutable precompilado en la última release de GitHub."
        downloadingRelease = "      Descargando {0} ({1})..."
        errDownload = "Error al resolver o descargar la release de GitHub: {0}"
        errInternet = "Asegúrate de estar conectado a internet e inténtalo de nuevo."
        stepCopy = "[2/4] Instalando el ejecutable en el perfil de usuario..."
        binLocked = "      El binario está bloqueado. Renombrando archivo antiguo para resolver el bloqueo..."
        binInstalled = "      Binario de PalAgent AI instalado en: {0}"
        stepOodle = "[3/4] Resolviendo oo2core_9_win64.dll para descompresión GVAS..."
        warnNoPalworld = "      [!] Advertencia: No se detectó Palworld (versión Steam) en tus bibliotecas."
        warnNoPalworldSave = "          Asegúrate de tener Palworld instalado en Steam en este PC, ya que PalAgent AI requiere las partidas guardadas locales."
        oodleResolved = "      oo2core_9_win64.dll ya está resuelto."
        oodleCopied = "      Se encontró y copió oo2core_9_win64.dll desde: {0}"
        warnNoOodle = "      Advertencia: oo2core_9_win64.dll no se encontró en las rutas estándar de Steam."
        warnOodleManual = "      Es posible que debas copiarlo manualmente a: {0}"
        
        stepConfig = "=== Configurar Tipo de Juego y Cliente ==="
        gameChoiceHeader = "Selecciona tu configuración de Palworld:"
        gameChoice1 = "  1. Singleplayer / Host Co-op (Juego local en este PC)"
        gameChoice2 = "  2. Host de Servidor Dedicado (Administrador de un servidor dedicado en este PC)"
        gameChoice3 = "  3. Cliente Remoto (Juegas en un servidor remoto o dedicado de un amigo)"
        gameChoiceEnter = "Selecciona el tipo de juego (1-3)"
        
        dedicatedHeader = "=== Configuración de Host de Servidor Dedicado ==="
        dedicatedPasscode = "Establece una contraseña segura para el servidor (o presiona Enter para ninguna)"
        dedicatedTask = "Registrando el servicio de servidor de fondo mediante el Programador de tareas de Windows..."
        dedicatedTaskDone = "Servicio de servidor dedicado registrado para iniciar en el arranque del sistema."
        
        remoteHeader = "=== Configuración de Cliente Remoto ==="
        remoteUrl = "Ingresa la IP y puerto del servidor host (ej. 192.168.1.50:8080)"
        remotePasscode = "Ingresa la contraseña de acceso al servidor"
        remoteUrlErr = "Error: La URL del host es obligatoria para la configuración del cliente remoto."
        
        mcpHeader = "=== Configurar Integración de Desarrollador (MCP) ==="
        mcpSelect = "Selecciona los editores de desarrollo para configurar el servidor MCP (separa con comas, ej. 1, 3):"
        mcpOptAll = "  A. Configurar TODOS los entornos disponibles"
        mcpOptSkip = "  S. Omitir configuración de MCP"
        mcpChoiceEnter = "Tu selección"
        mcpConfiguring = "Configurando MCP para {0}..."
        mcpSkipped = "Omitiendo la configuración de MCP."
        
        completeHeader = "         ¡Instalación completada con éxito!         "
        completeFolder = "  [+] Carpeta del ejecutable: {0}"
        completeModeHost = "  [+] Servidor de telemetría: Registrado al arrancar (Programador de tareas)"
        completeModeRemote = "  [+] Conexión de cliente: Apuntando a {0}"
        completeModeLocal = "  [+] Modo: Juego Local Singleplayer / Co-op"
        completeNextSteps = "  SIGUIENTES PASOS:"
        completeStep1 = "  1. Reinicia tu editor de IA o ventana de chat CLI."
        completeStep2 = "  2. Empieza a hacer preguntas en vivo para probar la conexión:"
        completeQ1 = "     - '¿Cuáles son los IVs de mis Pals?'"
        completeQ2 = "     - '¿En qué cofre de mi base guardé la Madera?'"
        completeQ3 = "     - '¿Cómo puedo criar un Anubis con mis Pals actuales?'"
        completeQ4 = "     - '¿Hay algún Pal en mi campamento deprimido o hambriento?'"
    }
}

# --- LANGUAGE SELECTOR SCREEN ---
Clear-Host
Write-Host "=====================================================" -ForegroundColor Green
Write-Host "Please select your language / Seleccione su idioma:" -ForegroundColor Yellow
Write-Host "  1. English"
Write-Host "  2. Español"
Write-Host "=====================================================" -ForegroundColor Green
Write-Host ""

$langChoice = Read-Host "Selection / Selección (1-2)"
$lang = "en"
if ($langChoice -eq "2") {
    $lang = "es"
}

$t = $msgs[$lang]

# --- MAIN MENU SCREEN ---
Clear-Host
Write-Host "  =====================================" -ForegroundColor Green
Write-Host "         $($t.title)                   " -ForegroundColor Green
Write-Host "  =====================================" -ForegroundColor Green
Write-Host ""

if ($hasPreviousInstall) {
    Write-Host ($t.prevInstall -f $installDir) -ForegroundColor Yellow
}

Write-Host $t.mainMenu
Write-Host $t.optInstall -ForegroundColor Green
Write-Host $t.optUninstall -ForegroundColor Red
Write-Host $t.optExit
Write-Host ""

$menuSelection = Read-Host $t.enterChoice

if ($menuSelection -eq "2") {
    Clear-Host
    Write-Host $t.uninstallHeader -ForegroundColor Red
    
    # 1. Remove PowerShell profile function helper
    Write-Host $t.unPSProfile -ForegroundColor Cyan
    $profilePath = $PROFILE
    if (Test-Path $profilePath) {
        try {
            $content = Get-Content -Path $profilePath -Raw -ErrorAction SilentlyContinue
            if ($content -and $content -like "*/palworld*") {
                $cleanContent = $content -replace "(?s)# PalAgent AI Custom Console Helper.*", ""
                Set-Content -Path $profilePath -Value $cleanContent -Force
                Write-Host ($t.unProfileRemoved -f $profilePath) -ForegroundColor DarkGray
            }
        } catch {
            Write-Host ($t.unProfileErr -f $_.Exception.Message) -ForegroundColor Yellow
        }
    }

    # 2. Clean MCP configuration server entries
    Write-Host $t.unMCPConfigs -ForegroundColor Cyan
    $mcpConfigs = @(
        Join-Path $env:USERPROFILE ".gemini\config\mcp_config.json",
        Join-Path $env:USERPROFILE ".gemini\antigravity-cli\mcp_config.json"
    )

    foreach ($configPath in $mcpConfigs) {
        if (Test-Path $configPath) {
            try {
                $json = Get-Content -Path $configPath -Raw | ConvertFrom-Json
                if ($json.mcpServers -and $json.mcpServers."palagent-ai") {
                    $json.mcpServers.PSObject.Properties.Remove("palagent-ai")
                    $cleanJson = $json | ConvertTo-Json -Depth 10
                    Set-Content -Path $configPath -Value $cleanJson -Force
                    Write-Host ($t.unMCPRemoved -f $configPath) -ForegroundColor DarkGray
                }
            } catch {
                Write-Host ($t.unMCPErr -f $configPath, $_.Exception.Message) -ForegroundColor Yellow
            }
        }
    }

    # 3. Remove agent skills
    Write-Host $t.unSkills -ForegroundColor Cyan
    $skillPaths = @(
        Join-Path $env:USERPROFILE ".gemini\config\skills\palagent-ai",
        Join-Path $env:USERPROFILE ".gemini\antigravity-cli\skills\palagent-ai"
    )
    foreach ($skillPath in $skillPaths) {
        if (Test-Path $skillPath) {
            Remove-Item -Path $skillPath -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host ($t.unSkillDeleted -f $skillPath) -ForegroundColor DarkGray
        }
    }

    # 4. Remove installation binary folder
    Write-Host $t.unBinaries -ForegroundColor Cyan
    if (Test-Path $installDir) {
        try {
            Remove-Item -Path (Join-Path $installDir "palagent-ai.exe.old") -Force -ErrorAction SilentlyContinue
            Remove-Item -Path $installDir -Recurse -Force
            Write-Host ($t.unBinFolderDeleted -f $installDir) -ForegroundColor DarkGray
        } catch {
            Write-Host $t.unBinLocked -ForegroundColor Yellow
            Remove-Item -Path $destDll -Force -ErrorAction SilentlyContinue
            Rename-Item -Path $destExe -NewName "palagent-ai.exe.trash" -Force -ErrorAction SilentlyContinue
            Remove-Item -Path (Join-Path $installDir "palagent-ai.exe.trash") -Force -ErrorAction SilentlyContinue
            Write-Host $t.unBinFolderMarked -ForegroundColor DarkGray
        }
    }

    Write-Host ""
    Write-Host "======================================================" -ForegroundColor Green
    Write-Host $t.unCompleted -ForegroundColor Green
    Write-Host "======================================================" -ForegroundColor Green
    Write-Host ""
    exit 0
}

if ($menuSelection -ne "1") {
    Write-Host $t.exiting
    exit 0
}

# --- INSTALLATION FLOW ---
Clear-Host
Write-Host $t.installHeader -ForegroundColor Green
Write-Host ""

$localExePath = Join-Path "target\release" "palagent-ai.exe"
$downloadedNew = $false

# Step 1: Resolving/Downloading the executable
Write-Host $t.stepExecutable -ForegroundColor Cyan

if (Test-Path $localExePath) {
    Write-Host ($t.localDevBuild -f $localExePath) -ForegroundColor DarkGray
    $sourceExe = $localExePath
} else {
    Write-Host $t.downloadingGitHub -ForegroundColor DarkGray
    $releaseUrl = "https://api.github.com/repos/$repo/releases/latest"
    
    try {
        $releaseData = Invoke-RestMethod -Uri $releaseUrl -UseBasicParsing -Headers @{"Cache-Control"="no-cache"}
        $asset = $releaseData.assets | Where-Object { $_.name -match "\.exe$" } | Select-Object -First 1
        
        if (-not $asset) {
            Write-Host $t.errNoAsset -ForegroundColor Red
            exit 1
        }
        
        $downloadUrl = $asset.browser_download_url
        $tempExe = Join-Path $env:TEMP $asset.name
        
        Write-Host ($t.downloadingRelease -f $releaseData.tag_name, $asset.name) -ForegroundColor DarkGray
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempExe -UseBasicParsing
        Unblock-File -Path $tempExe -ErrorAction SilentlyContinue
        
        $sourceExe = $tempExe
        $downloadedNew = $true
    } catch {
        Write-Host ($t.errDownload -f $_.Exception.Message) -ForegroundColor Red
        Write-Host $t.errInternet -ForegroundColor Yellow
        exit 1
    }
}

# Step 2: Copying binary to destination
Write-Host $t.stepCopy -ForegroundColor Cyan
if (Test-Path $destExe) {
    try {
        Copy-Item -Path $sourceExe -Destination $destExe -Force
    } catch {
        Write-Host $t.binLocked -ForegroundColor DarkGray
        $oldExe = Join-Path $installDir "palagent-ai.exe.old"
        Remove-Item -Path $oldExe -Force -ErrorAction SilentlyContinue
        Rename-Item -Path $destExe -NewName "palagent-ai.exe.old" -Force
        Copy-Item -Path $sourceExe -Destination $destExe -Force
    }
} else {
    Copy-Item -Path $sourceExe -Destination $destExe -Force
}

if ($downloadedNew) {
    Remove-Item -Path $sourceExe -Force -ErrorAction SilentlyContinue
}
Write-Host ($t.binInstalled -f $destExe) -ForegroundColor DarkGray

# Step 3: Resolving oo2core_9_win64.dll
Write-Host $t.stepOodle -ForegroundColor Cyan

$palworldInstallPath = Get-PalworldInstallPath
$gameDetected = $null -ne $palworldInstallPath

if (-not $gameDetected) {
    Write-Host $t.warnNoPalworld -ForegroundColor Yellow
    Write-Host $t.warnNoPalworldSave -ForegroundColor Yellow
}

if (-not (Test-Path $destDll)) {
    $standardDllPaths = [System.Collections.Generic.List[string]]::new()
    
    $standardDllPaths.Add((Join-Path $PSScriptRoot "oo2core_9_win64.dll"))
    $standardDllPaths.Add("oo2core_9_win64.dll")

    if ($gameDetected) {
        $standardDllPaths.Add((Join-Path $palworldInstallPath "Binaries\Win64\oo2core_9_win64.dll"))
    }

    $standardDllPaths.Add("C:\Program Files (x86)\Steam\steamapps\common\Palworld\Binaries\Win64\oo2core_9_win64.dll")
    $standardDllPaths.Add("D:\SteamLibrary\steamapps\common\Palworld\Binaries\Win64\oo2core_9_win64.dll")

    $foundDll = $null
    foreach ($path in $standardDllPaths) {
        if (Test-Path $path) {
            $foundDll = $path
            break
        }
    }

    if ($foundDll) {
        Copy-Item -Path $foundDll -Destination $destDll -Force
        Write-Host ($t.oodleCopied -f $foundDll) -ForegroundColor DarkGray
    } else {
        Write-Host $t.warnNoOodle -ForegroundColor Yellow
        Write-Host ($t.warnOodleManual -f $installDir) -ForegroundColor Yellow
    }
} else {
    Write-Host $t.oodleResolved -ForegroundColor DarkGray
}

# Step 4: Configuring Game Type & MCP Client
Clear-Host
Write-Host $t.stepConfig -ForegroundColor Green
Write-Host ""
Write-Host $t.gameChoiceHeader
Write-Host $t.gameChoice1 -ForegroundColor Green
Write-Host $t.gameChoice2 -ForegroundColor Cyan
Write-Host $t.gameChoice3 -ForegroundColor Yellow
Write-Host ""

$gameTypeChoice = Read-Host $t.gameChoiceEnter

$serverUrl = ""
$serverPasscode = ""

if ($gameTypeChoice -eq "2") {
    Clear-Host
    Write-Host $t.dedicatedHeader -ForegroundColor Cyan
    $serverPasscode = Read-Host $t.dedicatedPasscode
    
    Write-Host $t.dedicatedTask -ForegroundColor DarkGray
    if ($serverPasscode) {
        & $destExe setup-service --passcode $serverPasscode
    } else {
        & $destExe setup-service
    }
    Write-Host $t.dedicatedTaskDone -ForegroundColor DarkGray
} elseif ($gameTypeChoice -eq "3") {
    Clear-Host
    Write-Host $t.remoteHeader -ForegroundColor Yellow
    $serverUrl = Read-Host $t.remoteUrl
    $serverPasscode = Read-Host $t.remotePasscode
    
    if ([string]::IsNullOrWhiteSpace($serverUrl)) {
        Write-Host $t.remoteUrlErr -ForegroundColor Red
        exit 1
    }
}

# Choose MCP editor configurations
Clear-Host
Write-Host $t.mcpHeader -ForegroundColor Green
Write-Host ""

$choices = @(
    "antigravity-cli",
    "vscode-copilot",
    "cursor",
    "windsurf",
    "opencode",
    "claude-code",
    "gemini-cli",
    "codex",
    "qwen",
    "kiro",
    "pi",
    "kilocode"
)

Write-Host $t.mcpSelect
for ($i = 0; $i -lt $choices.Count; $i++) {
    Write-Host "  $($i + 1). $($choices[$i])"
}
Write-Host $t.mcpOptAll
Write-Host $t.mcpOptSkip
Write-Host ""

$inputSelection = Read-Host $t.mcpChoiceEnter

$setupParams = @()
if ($gameTypeChoice -eq "3") {
    $setupParams += "--host"
    $setupParams += $serverUrl
    if ($serverPasscode) {
        $setupParams += "--passcode"
        $setupParams += $serverPasscode
    }
}

# Pass the chosen language argument to setup so it propagates to SKILL.md and rules
$setupParams += "--lang"
$setupParams += $lang

if ($inputSelection -match "A" -or $inputSelection -match "a") {
    foreach ($choice in $choices) {
        Write-Host ($t.mcpConfiguring -f $choice)
        & $destExe setup $choice $setupParams
    }
} elseif ($inputSelection -match "S" -or $inputSelection -match "s" -or [string]::IsNullOrWhiteSpace($inputSelection)) {
    Write-Host $t.mcpSkipped
} else {
    $indices = $inputSelection -split ","
    foreach ($index in $indices) {
        $idx = [int]($index.Trim()) - 1
        if ($idx -ge 0 -and $idx -lt $choices.Count) {
            $choice = $choices[$idx]
            Write-Host ($t.mcpConfiguring -f $choice)
            & $destExe setup $choice $setupParams
        }
    }
}

# Final Completion screen & next steps summary
Clear-Host
Write-Host "======================================================" -ForegroundColor Green
Write-Host $t.completeHeader -ForegroundColor Green
Write-Host "======================================================" -ForegroundColor Green
Write-Host ""
Write-Host ($t.completeFolder -f $installDir) -ForegroundColor DarkGray
if ($gameTypeChoice -eq "2") {
    Write-Host $t.completeModeHost -ForegroundColor DarkGray
} elseif ($gameTypeChoice -eq "3") {
    Write-Host ($t.completeModeRemote -f $serverUrl) -ForegroundColor DarkGray
} else {
    Write-Host $t.completeModeLocal -ForegroundColor DarkGray
}
Write-Host ""
Write-Host "------------------------------------------------------" -ForegroundColor Yellow
Write-Host $t.completeNextSteps -ForegroundColor Yellow
Write-Host "------------------------------------------------------" -ForegroundColor Yellow
Write-Host $t.completeStep1
Write-Host $t.completeStep2
Write-Host $t.completeQ1 -ForegroundColor Cyan
Write-Host $t.completeQ2 -ForegroundColor Cyan
Write-Host $t.completeQ3 -ForegroundColor Cyan
Write-Host $t.completeQ4 -ForegroundColor Cyan
Write-Host ""
