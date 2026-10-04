<#
.SYNOPSIS
    Full Windows bundle chain: core -> app -> dist/Fluyer-windows-x64.zip
.DESCRIPTION
    Mirrors scripts/bundle_macos_app.sh for Windows:
      1. cargo build -p fluyer_core (--release with -Release)
      2. Stage fluyer_core.dll + BASS DLLs next to the cargo output
         (libs/windows is the source of truth).
      3. dotnet publish the WinUI app (self-contained unpackaged — the
         Windows App SDK framework ships inside the folder, like the macOS
         bundle's Frameworks dir).
      4. Zip the publish folder into dist/.
.EXAMPLE
    ./scripts/bundle_windows_app.ps1
    ./scripts/bundle_windows_app.ps1 -Release
#>
param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$Profile = if ($Release) { "release" } else { "debug" }
$DotnetConfig = if ($Release) { "Release" } else { "Debug" }
$DistDir = Join-Path $RepoRoot "dist"

Write-Host "==> Building fluyer_core ($Profile)..." -ForegroundColor Green
$cargoArgs = @("build", "-p", "fluyer_core")
if ($Release) { $cargoArgs += "--release" }
& cargo $cargoArgs
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

Write-Host "==> Staging native DLLs..." -ForegroundColor Green
$TargetDir = Join-Path $RepoRoot "target\$Profile"
$LibsDir = Join-Path $RepoRoot "libs\windows"
Copy-Item (Join-Path $LibsDir "*.dll") $TargetDir -Force

Write-Host "==> Publishing WinUI app ($DotnetConfig)..." -ForegroundColor Green
& dotnet publish "$RepoRoot\ui\windows\Fluyer.csproj" `
    -c $DotnetConfig `
    -r win-x64 `
    --self-contained false `
    -o "$DistDir\Fluyer-windows-x64-tmp"
if ($LASTEXITCODE -ne 0) { throw "dotnet publish failed" }

Write-Host "==> Zipping bundle..." -ForegroundColor Green
$ZipPath = "$DistDir\Fluyer-windows-x64.zip"
if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }
Compress-Archive -Path "$DistDir\Fluyer-windows-x64-tmp\*" -DestinationPath $ZipPath
Remove-Item "$DistDir\Fluyer-windows-x64-tmp" -Recurse -Force

Write-Host "==> Bundle ready: $ZipPath" -ForegroundColor Green
Get-ChildItem $ZipPath | Format-Table Name, @{N="MB";E={[math]::Round($_.Length / 1MB, 1)}}
