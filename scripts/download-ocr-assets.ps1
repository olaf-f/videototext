param(
  [string]$OutputRoot = "src-tauri/resources/ocr",
  [string]$RuntimeSource = "",
  [switch]$ForceDownload
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$modelSpecs = @(
  @{
    Name = "ch_PP-OCRv5_mobile_det.onnx"
    Url = "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.7.0/onnx/PP-OCRv5/det/ch_PP-OCRv5_mobile_det.onnx"
    Sha256 = "4d97c44a20d30a81aad087d6a396b08f786c4635742afc391f6621f5c6ae78ae"
  },
  @{
    Name = "ch_ppocr_mobile_v2.0_cls_infer.onnx"
    Url = "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.7.0/onnx/PP-OCRv4/cls/ch_ppocr_mobile_v2.0_cls_infer.onnx"
    Sha256 = "e47acedf663230f8863ff1ab0e64dd2d82b838fceb5957146dab185a89d6215c"
  },
  @{
    Name = "ch_PP-OCRv5_rec_mobile_infer.onnx"
    Url = "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.7.0/onnx/PP-OCRv5/rec/ch_PP-OCRv5_rec_mobile_infer.onnx"
    Sha256 = "5825fc7ebf84ae7a412be049820b4d86d77620f204a041697b0494669b1742c5"
  }
)

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

function Ensure-Directory {
  param(
    [Parameter(Mandatory = $true)]
    [string]$PathValue
  )

  New-Item -ItemType Directory -Path $PathValue -Force | Out-Null
}

function Get-Sha256Hex {
  param(
    [Parameter(Mandatory = $true)]
    [string]$PathValue
  )

  return (Get-FileHash -LiteralPath $PathValue -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Ensure-ModelFile {
  param(
    [Parameter(Mandatory = $true)]
    [hashtable]$Spec,
    [Parameter(Mandatory = $true)]
    [string]$ModelsDir
  )

  $destination = Join-Path $ModelsDir $Spec.Name
  if ((-not $ForceDownload) -and (Test-Path -LiteralPath $destination -PathType Leaf)) {
    $existingHash = Get-Sha256Hex -PathValue $destination
    if ($existingHash -eq $Spec.Sha256.ToLowerInvariant()) {
      Write-Host "Model already present: $($Spec.Name)"
      return
    }
  }

  Write-Host "Downloading model: $($Spec.Name)"
  Invoke-WebRequest -UseBasicParsing -Uri $Spec.Url -OutFile $destination

  $downloadedHash = Get-Sha256Hex -PathValue $destination
  if ($downloadedHash -ne $Spec.Sha256.ToLowerInvariant()) {
    throw "Checksum mismatch for $($Spec.Name). Expected $($Spec.Sha256), got $downloadedHash."
  }
}

function Copy-RuntimeAssets {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RuntimeDir,
    [Parameter(Mandatory = $false)]
    [string]$SourceDir
  )

  $copiedAny = $false
  $patterns = @("onnxruntime*.dll", "DirectML.dll")

  if (-not [string]::IsNullOrWhiteSpace($SourceDir)) {
    $resolvedSource = Resolve-AbsolutePath -PathValue $SourceDir
    if (Test-Path -LiteralPath $resolvedSource -PathType Container) {
      foreach ($pattern in $patterns) {
        $files = Get-ChildItem -LiteralPath $resolvedSource -File -Filter $pattern -ErrorAction SilentlyContinue
        foreach ($file in $files) {
          Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $RuntimeDir $file.Name) -Force
          $copiedAny = $true
        }
      }
    }
  }

  if ($copiedAny) {
    return
  }

  $fallbackRoots = @(
    "src-tauri/target/release",
    "src-tauri/target/debug"
  )

  foreach ($root in $fallbackRoots) {
    $absoluteRoot = Resolve-AbsolutePath -PathValue $root
    if (-not (Test-Path -LiteralPath $absoluteRoot -PathType Container)) {
      continue
    }

    foreach ($pattern in $patterns) {
      $files = Get-ChildItem -LiteralPath $absoluteRoot -File -Filter $pattern -ErrorAction SilentlyContinue
      foreach ($file in $files) {
        Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $RuntimeDir $file.Name) -Force
        $copiedAny = $true
      }
    }

    if ($copiedAny) {
      break
    }
  }

  if (-not $copiedAny) {
    Write-Host "No runtime DLLs found yet. Runtime directory will still be created."
  } else {
    Write-Host "Runtime assets prepared."
  }
}

$outputRootAbsolute = Resolve-AbsolutePath -PathValue $OutputRoot
$runtimeTarget = Join-Path $outputRootAbsolute "runtime"
$modelsTarget = Join-Path $outputRootAbsolute "models"

Ensure-Directory -PathValue $runtimeTarget
Ensure-Directory -PathValue $modelsTarget

foreach ($spec in $modelSpecs) {
  Ensure-ModelFile -Spec $spec -ModelsDir $modelsTarget
}

Copy-RuntimeAssets -RuntimeDir $runtimeTarget -SourceDir $RuntimeSource

$runtimeFiles = @(Get-ChildItem -LiteralPath $runtimeTarget -File -Recurse -ErrorAction SilentlyContinue | Where-Object { $_.Name -ne ".gitkeep" })
$modelFiles = @(Get-ChildItem -LiteralPath $modelsTarget -File -Recurse -ErrorAction SilentlyContinue | Where-Object { $_.Name -ne ".gitkeep" })

Write-Host "OCR assets ready: runtime files=$($runtimeFiles.Count), model files=$($modelFiles.Count)"
Write-Host "Output: $outputRootAbsolute"
