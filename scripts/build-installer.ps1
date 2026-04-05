param(
  [string]$AssetRoot = "src-tauri/resources/ocr",
  [string]$OutputRoot = "release/installer",
  [switch]$SkipAssetDownload,
  [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

function Resolve-AbsolutePath {
  param(
    [Parameter(Mandatory = $true)]
    [string]$PathValue
  )

  if ([System.IO.Path]::IsPathRooted($PathValue)) {
    return [System.IO.Path]::GetFullPath($PathValue)
  }

  return [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $PathValue))
}

if (-not $SkipAssetDownload) {
  powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "download-ocr-assets.ps1") -OutputRoot $AssetRoot
}

if (-not $SkipBuild) {
  pnpm.cmd tauri build --bundles nsis
}

$bundleRoot = Resolve-AbsolutePath -PathValue "src-tauri/target/release/bundle/nsis"
if (-not (Test-Path -LiteralPath $bundleRoot -PathType Container)) {
  throw "Installer output directory not found: $bundleRoot"
}

$installer = Get-ChildItem -LiteralPath $bundleRoot -File -Filter "*.exe" |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1

if (-not $installer) {
  throw "No NSIS installer found under $bundleRoot"
}

$outputRootAbsolute = Resolve-AbsolutePath -PathValue $OutputRoot
New-Item -ItemType Directory -Path $outputRootAbsolute -Force | Out-Null

$targetInstaller = Join-Path $outputRootAbsolute "SmartOCR-Pro-setup.exe"
Copy-Item -LiteralPath $installer.FullName -Destination $targetInstaller -Force

Write-Host "Installer ready: $targetInstaller"
