param(
  [string]$ExecutablePath,
  [string]$AssetRoot = "src-tauri/resources/ocr",
  [string]$OutputRoot = "release/portable",
  [switch]$SkipAssetDownload,
  [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

& (Join-Path $PSScriptRoot "build-portable.ps1") `
  -ExecutablePath $ExecutablePath `
  -AssetRoot $AssetRoot `
  -OutputRoot $OutputRoot `
  -SkipAssetDownload:$SkipAssetDownload `
  -SkipBuild:$SkipBuild
