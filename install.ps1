$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "  =====================================" -ForegroundColor Green
Write-Host "         PalAgent AI Installer         " -ForegroundColor Green
Write-Host "  =====================================" -ForegroundColor Green
Write-Host ""

# Step 1: Compiling the Rust application
Write-Host "[1/4] Compiling PalAgent AI in release mode..." -ForegroundColor Cyan

$repoUrl = "https://github.com/sewandev/palagent-ai.git"
$tempBuildDir = Join-Path $env:TEMP "palagent-ai-build"
$mustPopLocation = $false

if (-not (Test-Path "Cargo.toml")) {
    Write-Host "      Cargo.toml not found in current directory. Cloning repository to temporary folder..." -ForegroundColor DarkGray
    Remove-Item -Path $tempBuildDir -Recurse -Force -ErrorAction SilentlyContinue
    try {
        & git clone $repoUrl $tempBuildDir
        Push-Location $tempBuildDir
        $mustPopLocation = $true
    } catch {
        Write-Host "Git clone failed. Ensure git is installed and online." -ForegroundColor Red
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
}

try {
    & cargo build --release
} catch {
    Write-Host "Compilation failed. Ensure Rust and Cargo are installed." -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    if ($mustPopLocation) {
        Pop-Location
        Remove-Item -Path $tempBuildDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    exit 1
}

# Step 2: Creating permanent folder and copying binary
Write-Host "[2/4] Installing executable to user profile..." -ForegroundColor Cyan
$installDir = Join-Path $env:USERPROFILE ".palagent-ai"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

$sourceExe = if ($mustPopLocation) { Join-Path $tempBuildDir "target\release\palagent-ai.exe" } else { Join-Path "target\release" "palagent-ai.exe" }
$destExe = Join-Path $installDir "palagent-ai.exe"

# If the file is locked, rename it first
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
Write-Host "      PalAgent AI binary copied to: $destExe" -ForegroundColor DarkGray

# Step 3: Finding oo2core_9_win64.dll
Write-Host "[3/5] Resolving oo2core_9_win64.dll for GVAS decompression..." -ForegroundColor Cyan
$destDll = Join-Path $installDir "oo2core_9_win64.dll"

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

if ($mustPopLocation) {
    Pop-Location
    Remove-Item -Path $tempBuildDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "======================================================" -ForegroundColor Green
Write-Host "         Installation completed successfully!         " -ForegroundColor Green
Write-Host "======================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Your PalAgent AI MCP Server is configured and ready." -ForegroundColor Yellow
Write-Host ""
