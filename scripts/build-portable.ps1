param(
  [string]$ExecutablePath,
  [string]$AssetRoot = "src-tauri/resources/ocr",
  [string]$OutputRoot = "release/portable",
  [switch]$SkipAssetDownload,
  [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

function Resolve-BuiltExecutablePath {
  $candidates = @(
    "src-tauri/target/release/smartocr-pro.exe",
    "src-tauri/target/release/SmartOCR Pro.exe"
  )

  foreach ($candidate in $candidates) {
    if (Test-Path -LiteralPath $candidate -PathType Leaf) {
      return $candidate
    }
  }

  throw "Unable to find built SmartOCR Pro executable under src-tauri/target/release."
}

if (-not $SkipBuild) {
  if (-not $SkipAssetDownload) {
    powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "download-ocr-assets.ps1") -OutputRoot $AssetRoot
  }

  pnpm.cmd tauri build --no-bundle
}

if (-not $ExecutablePath) {
  $ExecutablePath = Resolve-BuiltExecutablePath
}

& (Join-Path $PSScriptRoot "stage-portable.ps1") `
  -ExecutablePath $ExecutablePath `
  -AssetRoot $AssetRoot `
  -OutputRoot $OutputRoot `
  -CreateZip
