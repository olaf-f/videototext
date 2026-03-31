param(
  [string]$RepoRoot = (Get-Location).Path
)

$payload = Join-Path $RepoRoot "installer_payload"
New-Item -ItemType Directory -Force -Path $payload | Out-Null

$sourceExe = Join-Path $RepoRoot "dist\VideoToText.exe"
if (-not (Test-Path $sourceExe)) {
  Write-Error "Windows exe not found"
  exit 1
}

Copy-Item $sourceExe (Join-Path $payload "VideoToText.exe") -Force
Copy-Item (Join-Path $RepoRoot "installer\install.cmd") (Join-Path $payload "install.cmd") -Force
Copy-Item (Join-Path $RepoRoot "installer\uninstall.cmd") (Join-Path $payload "uninstall.cmd") -Force

$target = Join-Path $RepoRoot "VideoToText-windows-x64-setup.exe"
$sed = @"
[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=1
HideExtractAnimation=0
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=
DisplayLicense=
FinishMessage=VideoToText installation completed.
TargetName=$target
FriendlyName=VideoToText Setup
AppLaunched=install.cmd
PostInstallCmd=<None>
AdminQuietInstCmd=install.cmd
UserQuietInstCmd=install.cmd
SourceFiles=SourceFiles
[Strings]
FILE0=VideoToText.exe
FILE1=install.cmd
FILE2=uninstall.cmd
[SourceFiles]
SourceFiles0=$payload
[SourceFiles0]
%FILE0%=
%FILE1%=
%FILE2%=
"@

$sedPath = Join-Path $RepoRoot "installer\VideoToText.sed"
Set-Content -Path $sedPath -Value $sed -Encoding ASCII

& "$env:SystemRoot\System32\iexpress.exe" /N $sedPath
if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}

if (-not (Test-Path $target)) {
  Write-Error "Windows setup exe not found"
  exit 1
}

Copy-Item $sourceExe (Join-Path $RepoRoot "VideoToText-windows-x64.exe") -Force
