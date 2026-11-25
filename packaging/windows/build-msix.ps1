param(
    [Parameter(Mandatory = $true)]
    [string]$ExePath,

    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$OutputMsix,

    [string]$Publisher = "CN=F9A61C2C-016B-4569-AF3E-D8FB4CBCB150"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir "..\..")
$stageDir = Join-Path $repoRoot "target\msix-stage"

if (Test-Path $stageDir) {
    Remove-Item -Recurse -Force $stageDir
}
New-Item -ItemType Directory -Path $stageDir | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "assets") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "third_party") | Out-Null

Copy-Item $ExePath (Join-Path $stageDir "motaa-desktop.exe")
Copy-Item (Join-Path $repoRoot "LICENSE") (Join-Path $stageDir "LICENSE")
Copy-Item (Join-Path $repoRoot "third_party\*LICENSE*") (Join-Path $stageDir "third_party\")
Copy-Item (Join-Path $repoRoot "third_party\xattr\LICENSE-*") (Join-Path $stageDir "third_party\")

$assetFiles = @(
    "StoreLogo.png",
    "Square44x44Logo.png",
    "Square150x150Logo.png",
    "Square310x310Logo.png",
    "Wide310x150Logo.png",
    "SplashScreen.png"
)
foreach ($asset in $assetFiles) {
    Copy-Item (Join-Path $scriptDir "assets\$asset") (Join-Path $stageDir "assets\$asset")
}

$cleanVersion = $Version.TrimStart("v")
$manifest = Get-Content (Join-Path $scriptDir "AppxManifest.xml") -Raw
$manifest = $manifest.Replace("{{PUBLISHER}}", $Publisher).Replace("{{VERSION}}", $cleanVersion)
Set-Content -Path (Join-Path $stageDir "AppxManifest.xml") -Value $manifest -Encoding UTF8

$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter makeappx.exe -ErrorAction SilentlyContinue |
    Where-Object { $_.Directory.Name -eq "x64" } |
    Sort-Object FullName -Descending |
    Select-Object -First 1

if (-not $makeappx) {
    throw "Windows SDK makeappx.exe was not found."
}

$outParent = Split-Path -Parent $OutputMsix
if ($outParent -and -not (Test-Path $outParent)) {
    New-Item -ItemType Directory -Path $outParent | Out-Null
}

& $makeappx.FullName pack /d $stageDir /p $OutputMsix /o
if ($LASTEXITCODE -ne 0) {
    throw "makeappx pack failed with exit code $LASTEXITCODE"
}

Write-Host "Built MSIX package: $OutputMsix"
