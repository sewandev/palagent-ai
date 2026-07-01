$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "  =====================================" -ForegroundColor Green
Write-Host "         PalAgent AI Manager           " -ForegroundColor Green
Write-Host "  =====================================" -ForegroundColor Green
Write-Host ""

$repo = "sewandev/palagent-ai"
$installDir = Join-Path $env:USERPROFILE ".palagent-ai"
$destExe = Join-Path $installDir "palagent-ai.exe"
$destDll = Join-Path $installDir "oo2core_9_win64.dll"

# Detect previous installation
$hasPreviousInstall = Test-Path $destExe
if ($hasPreviousInstall) {
    Write-Host "      [!] Previous installation detected at: $installDir" -ForegroundColor Yellow
}

Write-Host "Please select an option:"
Write-Host "  1. Install / Upgrade PalAgent AI" -ForegroundColor Green
Write-Host "  2. Uninstall and clean up all residues" -ForegroundColor Red
Write-Host "  3. Exit"
Write-Host ""

$menuSelection = Read-Host "Enter your choice (1-3)"

if ($menuSelection -eq "2") {
    Write-Host ""
    Write-Host "=== Starting Uninstallation & Cleanup ===" -ForegroundColor Red
    
    # 1. Remove PowerShell profile function helper
    Write-Host "[-] Cleaning up PowerShell profile helper..." -ForegroundColor Cyan
    $profilePath = $PROFILE
    if (Test-Path $profilePath) {
        try {
            $content = Get-Content -Path $profilePath -Raw -ErrorAction SilentlyContinue
            if ($content -and $content -like "*/palworld*") {
                $cleanContent = $content -replace "(?s)# PalAgent AI Custom Console Helper.*", ""
                Set-Content -Path $profilePath -Value $cleanContent -Force
                Write-Host "      Removed /palworld helper from $profilePath" -ForegroundColor DarkGray
            }
        } catch {
            Write-Host "      Could not clean profile file: $($_.Exception.Message)" -ForegroundColor Yellow
        }
    }

    # 2. Clean MCP configuration server entries from files without breaking other tools
    Write-Host "[-] Removing MCP server config registrations..." -ForegroundColor Cyan
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
                    Write-Host "      Removed 'palagent-ai' registration from $configPath" -ForegroundColor DarkGray
                }
            } catch {
                Write-Host "      Failed to clean MCP config ${configPath}: $($_.Exception.Message)" -ForegroundColor Yellow
            }
        }
    }

    # 3. Remove agent skills and rules directories
    Write-Host "[-] Deleting agent skill templates..." -ForegroundColor Cyan
    $skillPaths = @(
        Join-Path $env:USERPROFILE ".gemini\config\skills\palagent-ai",
        Join-Path $env:USERPROFILE ".gemini\antigravity-cli\skills\palagent-ai"
    )
    foreach ($skillPath in $skillPaths) {
        if (Test-Path $skillPath) {
            Remove-Item -Path $skillPath -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "      Deleted skill directory: $skillPath" -ForegroundColor DarkGray
        }
    }

    # 4. Remove installation binary folder
    Write-Host "[-] Deleting permanent binaries directory..." -ForegroundColor Cyan
    if (Test-Path $installDir) {
        try {
            # Try to release lock first by deleting old executables
            Remove-Item -Path (Join-Path $installDir "palagent-ai.exe.old") -Force -ErrorAction SilentlyContinue
            Remove-Item -Path $installDir -Recurse -Force
            Write-Host "      Deleted installation folder: $installDir" -ForegroundColor DarkGray
        } catch {
            Write-Host "      Some files were locked. Attempting to delete individual files..." -ForegroundColor Yellow
            Remove-Item -Path $destDll -Force -ErrorAction SilentlyContinue
            Rename-Item -Path $destExe -NewName "palagent-ai.exe.trash" -Force -ErrorAction SilentlyContinue
            Remove-Item -Path (Join-Path $installDir "palagent-ai.exe.trash") -Force -ErrorAction SilentlyContinue
            Write-Host "      Binary folder marked for deletion on next boot or release." -ForegroundColor DarkGray
        }
    }

    Write-Host ""
    Write-Host "======================================================" -ForegroundColor Green
    Write-Host "     Uninstallation and cleanup completed!           " -ForegroundColor Green
    Write-Host "======================================================" -ForegroundColor Green
    Write-Host ""
    exit 0
}

if ($menuSelection -ne "1") {
    Write-Host "Exiting installer."
    exit 0
}

# --- INSTALLATION FLOW ---
Write-Host ""
Write-Host "=== Starting Installation & Upgrade ===" -ForegroundColor Green

$localExePath = Join-Path "target\release" "palagent-ai.exe"
$downloadedNew = $false

# Step 1: Resolving/Downloading the executable
Write-Host "[1/4] Resolving PalAgent AI executable..." -ForegroundColor Cyan

if (Test-Path $localExePath) {
    Write-Host "      Detected local development build. Copying from: $localExePath" -ForegroundColor DarkGray
    $sourceExe = $localExePath
} else {
    Write-Host "      Downloading latest precompiled release from GitHub..." -ForegroundColor DarkGray
    $releaseUrl = "https://api.github.com/repos/$repo/releases/latest"
    
    try {
        $releaseData = Invoke-RestMethod -Uri $releaseUrl -UseBasicParsing -Headers @{"Cache-Control"="no-cache"}
        $asset = $releaseData.assets | Where-Object { $_.name -match "\.exe$" } | Select-Object -First 1
        
        if (-not $asset) {
            Write-Host "Error: No precompiled executable was found in the latest GitHub release." -ForegroundColor Red
            exit 1
        }
        
        $downloadUrl = $asset.browser_download_url
        $tempExe = Join-Path $env:TEMP $asset.name
        
        Write-Host "      Downloading $($releaseData.tag_name) ($($asset.name))..." -ForegroundColor DarkGray
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempExe -UseBasicParsing
        Unblock-File -Path $tempExe -ErrorAction SilentlyContinue
        
        $sourceExe = $tempExe
        $downloadedNew = $true
    } catch {
        Write-Host "Failed to resolve or download the precompiled release from GitHub: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "Ensure you are connected to the internet and try again." -ForegroundColor Yellow
        exit 1
    }
}

# Step 2: Copying binary to destination
Write-Host "[2/4] Installing executable to user profile..." -ForegroundColor Cyan
if (Test-Path $destExe) {
    try {
        Copy-Item -Path $sourceExe -Destination $destExe -Force
    } catch {
        Write-Host "      Binary is locked. Renaming old file to resolve lock..." -ForegroundColor DarkGray
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
Write-Host "      PalAgent AI binary installed to: $destExe" -ForegroundColor DarkGray

# Step 3: Resolving oo2core_9_win64.dll
Write-Host "[3/4] Resolving oo2core_9_win64.dll for GVAS decompression..." -ForegroundColor Cyan

$palworldFolders = @(
    "C:\Program Files (x86)\Steam\steamapps\common\Palworld",
    "C:\Program Files\Steam\steamapps\common\Palworld",
    "D:\SteamLibrary\steamapps\common\Palworld",
    "E:\SteamLibrary\steamapps\common\Palworld",
    "F:\SteamLibrary\steamapps\common\Palworld"
)

$gameDetected = $false
foreach ($folder in $palworldFolders) {
    if (Test-Path $folder) {
        $gameDetected = $true
        break
    }
}

if (-not $gameDetected) {
    Write-Host "      [!] Warning: Palworld (Steam version) was not detected in standard installation paths." -ForegroundColor Yellow
    Write-Host "          Ensure Palworld is installed on Steam on this PC, as PalAgent AI requires" -ForegroundColor Yellow
    Write-Host "          local save files to function." -ForegroundColor Yellow
}

if (-not (Test-Path $destDll)) {
    $standardDllPaths = @(
        "C:\Program Files (x86)\Steam\steamapps\common\Palworld\Binaries\Win64\oo2core_9_win64.dll",
        "C:\Program Files\Steam\steamapps\common\Palworld\Binaries\Win64\oo2core_9_win64.dll",
        "D:\SteamLibrary\steamapps\common\Palworld\Binaries\Win64\oo2core_9_win64.dll",
        "E:\SteamLibrary\steamapps\common\Palworld\Binaries\Win64\oo2core_9_win64.dll",
        "F:\SteamLibrary\steamapps\common\Palworld\Binaries\Win64\oo2core_9_win64.dll"
    )

    $foundDll = $null
    foreach ($path in $standardDllPaths) {
        if (Test-Path $path) {
            $foundDll = $path
            break
        }
    }

    if ($foundDll) {
        Copy-Item -Path $foundDll -Destination $destDll -Force
        Write-Host "      Found and copied oo2core_9_win64.dll from: $foundDll" -ForegroundColor DarkGray
    } else {
        Write-Host "      Warning: oo2core_9_win64.dll not found in standard Steam paths." -ForegroundColor Yellow
        Write-Host "      You might need to copy it manually to: $installDir" -ForegroundColor Yellow
    }
} else {
    Write-Host "      oo2core_9_win64.dll is already resolved." -ForegroundColor DarkGray
}

# Step 4: Configuring MCP Clients
Write-Host "[4/4] Configuring MCP integrations for developers..." -ForegroundColor Cyan

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

Write-Host "Select the developer environments to configure the MCP server for (separate with commas, e.g. 1, 3):"
for ($i = 0; $i -lt $choices.Count; $i++) {
    Write-Host "  $($i + 1). $($choices[$i])"
}
Write-Host "  A. Configure ALL available environments"
Write-Host "  S. Skip MCP configuration"

$inputSelection = Read-Host "Your selection"

if ($inputSelection -match "A" -or $inputSelection -match "a") {
    foreach ($choice in $choices) {
        Write-Host "Configuring MCP for $choice..."
        & $destExe setup $choice
    }
} elseif ($inputSelection -match "S" -or $inputSelection -match "s" -or [string]::IsNullOrWhiteSpace($inputSelection)) {
    Write-Host "Skipping MCP client setup."
} else {
    $indices = $inputSelection -split ","
    foreach ($index in $indices) {
        $idx = [int]($index.Trim()) - 1
        if ($idx -ge 0 -and $idx -lt $choices.Count) {
            $choice = $choices[$idx]
            Write-Host "Configuring MCP for $choice..."
            & $destExe setup $choice
        }
    }
}

Write-Host ""
Write-Host "======================================================" -ForegroundColor Green
Write-Host "         Installation completed successfully!         " -ForegroundColor Green
Write-Host "======================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Your PalAgent AI MCP Server is configured and ready." -ForegroundColor Yellow
Write-Host ""
