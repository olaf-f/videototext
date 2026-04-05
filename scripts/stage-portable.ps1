param(
  [Parameter(Mandatory = $true)]
  [string]$ExecutablePath,
  [string]$AssetRoot = "src-tauri/resources/ocr",
  [string]$OutputRoot = "release/portable",
  [string]$PackageName = "SmartOCR-Pro",
  [string]$PortableReadmePath = "README-PORTABLE.txt",
  [switch]$CreateZip
)

$ErrorActionPreference = "Stop"

function Get-RequiredAssetDirectory {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Root,
    [Parameter(Mandatory = $true)]
    [string]$Name
  )

  $path = Join-Path $Root $Name
  if (-not (Test-Path -LiteralPath $path -PathType Container)) {
    throw "Missing OCR asset directory: $path"
  }

  $files = Get-ChildItem -LiteralPath $path -Recurse -File | Where-Object { $_.Name -ne ".gitkeep" }
  if ($files.Count -eq 0) {
    throw "OCR asset directory is empty: $path"
  }

  return $path
}

$resolvedExe = (Resolve-Path -LiteralPath $ExecutablePath).Path
$resolvedAssetRoot = (Resolve-Path -LiteralPath $AssetRoot).Path
$resolvedReadme = (Resolve-Path -LiteralPath $PortableReadmePath).Path

[void](Get-RequiredAssetDirectory -Root $resolvedAssetRoot -Name "runtime")
[void](Get-RequiredAssetDirectory -Root $resolvedAssetRoot -Name "models")

$resolvedOutputRoot = if ([System.IO.Path]::IsPathRooted($OutputRoot)) {
  [System.IO.Path]::GetFullPath($OutputRoot)
} else {
  [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $OutputRoot))
}
$packageRoot = Join-Path $resolvedOutputRoot $PackageName
$resourceRoot = Join-Path $packageRoot "resources"
$ocrTargetRoot = Join-Path $resourceRoot "ocr"
$zipPath = Join-Path $resolvedOutputRoot "SmartOCR-Pro-windows-portable.zip"

if (Test-Path -LiteralPath $packageRoot) {
  Remove-Item -LiteralPath $packageRoot -Recurse -Force
}

if (Test-Path -LiteralPath $zipPath) {
  Remove-Item -LiteralPath $zipPath -Force
}

New-Item -ItemType Directory -Path $ocrTargetRoot -Force | Out-Null
Copy-Item -LiteralPath $resolvedExe -Destination (Join-Path $packageRoot "SmartOCR Pro.exe") -Force
Copy-Item -LiteralPath $resolvedReadme -Destination (Join-Path $packageRoot "README-PORTABLE.txt") -Force
Copy-Item -Path (Join-Path $resolvedAssetRoot "*") -Destination $ocrTargetRoot -Recurse -Force

if ($CreateZip) {
  Compress-Archive -Path $packageRoot -DestinationPath $zipPath -CompressionLevel Optimal -Force
  Write-Host "Portable zip created at $zipPath"
} else {
  Write-Host "Portable directory staged at $packageRoot"
}
