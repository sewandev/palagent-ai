$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "  =====================================" -ForegroundColor Green
Write-Host "         PalAgent AI Installer         " -ForegroundColor Green
Write-Host "  =====================================" -ForegroundColor Green
Write-Host ""

$repo = "sewandev/palagent-ai"
$installDir = Join-Path $env:USERPROFILE ".palagent-ai"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

$destExe = Join-Path $installDir "palagent-ai.exe"
$destDll = Join-Path $installDir "oo2core_9_win64.dll"
$localExePath = Join-Path "target\release" "palagent-ai.exe"

# Step 1: Resolving/Downloading the executable
Write-Host "[1/4] Resolving PalAgent AI executable..." -ForegroundColor Cyan

$downloadedNew = $false

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
