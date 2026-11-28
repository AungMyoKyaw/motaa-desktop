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

function Get-PeImportedDlls {
    param(
        [Parameter(Mandatory = $true)]
        [string]$BinaryPath
    )

    [byte[]]$bytes = [System.IO.File]::ReadAllBytes($BinaryPath)
    if ($bytes.Length -lt 0x40 -or $bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) {
        throw "Not a valid PE binary (missing MZ header): $BinaryPath"
    }

    [int]$peOffset = [System.BitConverter]::ToInt32($bytes, 0x3C)
    if ($peOffset -lt 0 -or ($peOffset + 24) -gt $bytes.Length) {
        throw "Invalid PE header offset in $BinaryPath"
    }
    if ($bytes[$peOffset] -ne 0x50 -or $bytes[$peOffset + 1] -ne 0x45 -or $bytes[$peOffset + 2] -ne 0x00 -or $bytes[$peOffset + 3] -ne 0x00) {
        throw "Not a valid PE binary (missing PE signature): $BinaryPath"
    }

    [uint16]$numSections = [System.BitConverter]::ToUInt16($bytes, $peOffset + 6)
    [uint16]$sizeOfOptionalHeader = [System.BitConverter]::ToUInt16($bytes, $peOffset + 20)
    [int]$optOffset = $peOffset + 24
    [uint16]$magic = [System.BitConverter]::ToUInt16($bytes, $optOffset)

    if ($magic -eq 0x20B) {
        [int]$dataDirOffset = $optOffset + 112
    } elseif ($magic -eq 0x10B) {
        [int]$dataDirOffset = $optOffset + 96
    } else {
        throw ("Unsupported PE optional header magic 0x{0:X} in {1}" -f $magic, $BinaryPath)
    }

    [uint32]$importRva = [System.BitConverter]::ToUInt32($bytes, $dataDirOffset + 8)
    if ($importRva -eq 0) {
        return @()
    }

    [int]$secOffset = $optOffset + [int]$sizeOfOptionalHeader
    $sections = New-Object 'System.Collections.Generic.List[object]'
    for ($i = 0; $i -lt $numSections; $i++) {
        [int]$o = $secOffset + ($i * 40)
        [uint64]$vsize = [System.BitConverter]::ToUInt32($bytes, $o + 8)
        [uint64]$va = [System.BitConverter]::ToUInt32($bytes, $o + 12)
        [uint64]$rsize = [System.BitConverter]::ToUInt32($bytes, $o + 16)
        [uint64]$roff = [System.BitConverter]::ToUInt32($bytes, $o + 20)
        $sections.Add([pscustomobject]@{
            VirtualAddress   = $va
            Span             = [Math]::Max($vsize, $rsize)
            PointerToRawData = $roff
        })
    }

    $resolveRva = {
        param([uint64]$rva)
        foreach ($s in $sections) {
            if ($rva -ge $s.VirtualAddress -and $rva -lt ($s.VirtualAddress + $s.Span)) {
                return [int]($s.PointerToRawData + ($rva - $s.VirtualAddress))
            }
        }
        throw ("Could not map PE RVA 0x{0:X} to file offset in {1}" -f $rva, $BinaryPath)
    }

    [int]$impOff = & $resolveRva ([uint64]$importRva)
    $dlls = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)

    while (($impOff + 20) -le $bytes.Length) {
        [uint32]$origFirstThunk = [System.BitConverter]::ToUInt32($bytes, $impOff)
        [uint32]$nameRva = [System.BitConverter]::ToUInt32($bytes, $impOff + 12)
        if ($origFirstThunk -eq 0 -and $nameRva -eq 0) {
            break
        }
        if ($nameRva -ne 0) {
            [int]$nameOff = & $resolveRva ([uint64]$nameRva)
            [int]$endOff = $nameOff
            while ($endOff -lt $bytes.Length -and $bytes[$endOff] -ne 0) {
                $endOff++
            }
            $dllName = [System.Text.Encoding]::ASCII.GetString($bytes, $nameOff, $endOff - $nameOff)
            [void]$dlls.Add($dllName)
        }
        $impOff += 20
    }

    return @($dlls | Sort-Object)
}

function Assert-NoVcRuntimeImports {
    param(
        [Parameter(Mandatory = $true)]
        [string]$BinaryPath
    )

    $dlls = @(Get-PeImportedDlls -BinaryPath $BinaryPath)
    Write-Host "PE imported DLLs: $($dlls -join ', ')"
    $forbidden = @($dlls | Where-Object { $_ -match '^(vcruntime|msvcp|concrt|vcomp|mfc|atl)\d*(_\d+)?\.dll$' })
    if ($forbidden.Count -gt 0) {
        throw "Forbidden non-static MSVC runtime DLL import(s) detected in ${BinaryPath}: $($forbidden -join ', '). Build with -C target-feature=+crt-static (--target x86_64-pc-windows-msvc)."
    }
}

$resolvedExe = (Resolve-Path $ExePath).Path
Assert-NoVcRuntimeImports -BinaryPath $resolvedExe

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir "..\..")
$stageDir = Join-Path $repoRoot "target\msix-stage"

if (Test-Path $stageDir) {
    Remove-Item -Recurse -Force $stageDir
}
New-Item -ItemType Directory -Path $stageDir | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "assets") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageDir "third_party") | Out-Null

Copy-Item $resolvedExe (Join-Path $stageDir "motaa-desktop.exe")
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
