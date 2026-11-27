param(
    [Parameter(Mandatory = $true)]
    [string]$ExePath,

    [Parameter(Mandatory = $true)]
    [string]$OutputDir,

    [int]$Width = 1440,
    [int]$Height = 900
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class Win32Capture {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    public struct DEVMODE {
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string dmDeviceName;
        public short dmSpecVersion;
        public short dmDriverVersion;
        public short dmSize;
        public short dmDriverExtra;
        public int dmFields;
        public int dmPositionX;
        public int dmPositionY;
        public int dmDisplayOrientation;
        public int dmDisplayFixedOutput;
        public short dmColor;
        public short dmDuplex;
        public short dmYResolution;
        public short dmTTOption;
        public short dmCollate;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
        public string dmFormName;
        public short dmLogPixels;
        public int dmBitsPerPel;
        public int dmPelsWidth;
        public int dmPelsHeight;
        public int dmDisplayFlags;
        public int dmDisplayFrequency;
        public int dmICMMethod;
        public int dmICMIntent;
        public int dmMediaType;
        public int dmDitherType;
        public int dmReserved1;
        public int dmReserved2;
        public int dmPanningWidth;
        public int dmPanningHeight;
    }

    public const int ENUM_CURRENT_SETTINGS = -1;
    public const int DM_PELSWIDTH = 0x80000;
    public const int DM_PELSHEIGHT = 0x100000;
    public const int CDS_UPDATEREGISTRY = 0x01;
    public const int DISP_CHANGE_SUCCESSFUL = 0;
    public const uint SWP_SHOWWINDOW = 0x0040;
    public const uint PW_RENDERFULLCONTENT = 0x00000002;
    public const int SW_RESTORE = 9;

    [DllImport("user32.dll")]
    public static extern bool SetProcessDpiAwarenessContext(IntPtr dpiContext);

    [DllImport("user32.dll")]
    public static extern int GetSystemMetrics(int nIndex);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern bool EnumDisplaySettings(string deviceName, int modeNum, ref DEVMODE devMode);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int ChangeDisplaySettings(ref DEVMODE devMode, int flags);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdcBlt, uint nFlags);

    public static IntPtr FindMainWindow(int processId) {
        IntPtr found = IntPtr.Zero;
        int bestArea = -1;
        EnumWindows((hWnd, lParam) => {
            uint pid;
            GetWindowThreadProcessId(hWnd, out pid);
            if ((int)pid == processId && IsWindowVisible(hWnd)) {
                RECT rect;
                if (GetWindowRect(hWnd, out rect)) {
                    int w = rect.Right - rect.Left;
                    int h = rect.Bottom - rect.Top;
                    int area = w * h;
                    if (w >= 400 && h >= 300 && area > bestArea) {
                        bestArea = area;
                        found = hWnd;
                    }
                }
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
"@

try {
    # DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = -4
    [Win32Capture]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
} catch {
    Write-Host "Note: SetProcessDpiAwarenessContext skipped: $_"
}

function Ensure-DisplayResolution([int]$MinWidth, [int]$MinHeight) {
    $curW = [Win32Capture]::GetSystemMetrics(0)
    $curH = [Win32Capture]::GetSystemMetrics(1)
    Write-Host "Current Windows display resolution: ${curW}x${curH}"
    if ($curW -ge $MinWidth -and $curH -ge $MinHeight) {
        return
    }
    $candidates = @(@(1920, 1080), @(1600, 900), @(1440, 900))
    foreach ($cand in $candidates) {
        $dm = New-Object Win32Capture+DEVMODE
        $dm.dmSize = [System.Runtime.InteropServices.Marshal]::SizeOf($dm)
        if ([Win32Capture]::EnumDisplaySettings($null, [Win32Capture]::ENUM_CURRENT_SETTINGS, [ref]$dm)) {
            $dm.dmPelsWidth = $cand[0]
            $dm.dmPelsHeight = $cand[1]
            $dm.dmFields = [Win32Capture]::DM_PELSWIDTH -bor [Win32Capture]::DM_PELSHEIGHT
            $res = [Win32Capture]::ChangeDisplaySettings([ref]$dm, 0)
            if ($res -eq [Win32Capture]::DISP_CHANGE_SUCCESSFUL) {
                Write-Host "Switched display resolution to $($cand[0])x$($cand[1])"
                Start-Sleep -Milliseconds 500
                return
            }
        }
    }
    Write-Host "Display resolution unchanged (${curW}x${curH}); PrintWindow will capture window at requested size."
}

function Test-ImageNotBlank([System.Drawing.Bitmap]$Bitmap) {
    $colors = New-Object 'System.Collections.Generic.HashSet[int]'
    $stepX = [Math]::Max(1, [int]($Bitmap.Width / 24))
    $stepY = [Math]::Max(1, [int]($Bitmap.Height / 16))
    for ($y = $stepY; $y -lt $Bitmap.Height - $stepY; $y += $stepY) {
        for ($x = $stepX; $x -lt $Bitmap.Width - $stepX; $x += $stepX) {
            $c = $Bitmap.GetPixel($x, $y)
            $rgb = ($c.R -shl 16) -bor ($c.G -shl 8) -bor $c.B
            [void]$colors.Add($rgb)
        }
    }
    return $colors.Count -ge 12
}

function Capture-WindowPng([IntPtr]$Hwnd, [string]$Path) {
    $rect = New-Object Win32Capture+RECT
    if (-not [Win32Capture]::GetWindowRect($Hwnd, [ref]$rect)) {
        throw "GetWindowRect failed for hwnd $Hwnd"
    }
    $w = $rect.Right - $rect.Left
    $h = $rect.Bottom - $rect.Top
    if ($w -lt 1366 -or $h -lt 768) {
        Write-Host "Window rect was ${w}x${h}; resizing to ${Width}x${Height}..."
        [Win32Capture]::SetWindowPos($Hwnd, [IntPtr]::Zero, 0, 0, $Width, $Height, [Win32Capture]::SWP_SHOWWINDOW) | Out-Null
        Start-Sleep -Milliseconds 450
        [Win32Capture]::GetWindowRect($Hwnd, [ref]$rect) | Out-Null
        $w = $rect.Right - $rect.Left
        $h = $rect.Bottom - $rect.Top
    }

    $bmp = New-Object System.Drawing.Bitmap($w, $h, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $captured = $false
    try {
        $gfx = [System.Drawing.Graphics]::FromImage($bmp)
        try {
            $hdc = $gfx.GetHdc()
            try {
                $ok = [Win32Capture]::PrintWindow($Hwnd, $hdc, [Win32Capture]::PW_RENDERFULLCONTENT)
            } finally {
                $gfx.ReleaseHdc($hdc)
            }
            if ($ok -and (Test-ImageNotBlank $bmp)) {
                $captured = $true
            } else {
                Write-Host "PrintWindow returned blank/false (ok=$ok); falling back to CopyFromScreen..."
                $gfx.CopyFromScreen($rect.Left, $rect.Top, 0, 0, (New-Object System.Drawing.Size($w, $h)), [System.Drawing.CopyPixelOperation]::SourceCopy)
                if (Test-ImageNotBlank $bmp) {
                    $captured = $true
                }
            }
        } finally {
            $gfx.Dispose()
        }
        if (-not $captured) {
            throw "Captured screenshot for $Path appears blank/solid color."
        }
        $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $bmp.Dispose()
    }

    $fileInfo = Get-Item $Path
    if ($fileInfo.Length -lt 15000) {
        throw "Screenshot $Path is unexpectedly small ($($fileInfo.Length) bytes)."
    }
    if ($w -lt 1366 -or $h -lt 768) {
        throw "Screenshot $Path dimensions ${w}x${h} are below Microsoft Store minimum 1366x768."
    }
    $hash = (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Host "Captured $Path (${w}x${h}, $($fileInfo.Length) bytes, sha256=$hash)"
    return [ordered]@{
        file = $fileInfo.Name
        width = $w
        height = $h
        bytes = $fileInfo.Length
        sha256 = $hash
    }
}

$resolvedExe = (Resolve-Path $ExePath).Path
if (Test-Path $OutputDir) {
    Remove-Item -Recurse -Force $OutputDir
}
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
$resolvedOut = (Resolve-Path $OutputDir).Path

Ensure-DisplayResolution -MinWidth $Width -MinHeight $Height

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $resolvedExe
$psi.Arguments = "--store-screenshots `"$resolvedOut`""
$psi.UseShellExecute = $false
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.CreateNoWindow = $false

$proc = New-Object System.Diagnostics.Process
$proc.StartInfo = $psi
[void]$proc.Start()

$records = @()
try {
    while ($true) {
        $line = $proc.StandardOutput.ReadLine()
        if ($null -eq $line) {
            break
        }
        Write-Host "[app] $line"
        if ($line -match '^STORE_CAPTURE_READYs+([A-Za-z0-9_-]+)$') {
            $slug = $Matches[1]
            $hwnd = [IntPtr]::Zero
            for ($attempt = 0; $attempt -lt 30; $attempt++) {
                $hwnd = [Win32Capture]::FindMainWindow($proc.Id)
                if ($hwnd -ne [IntPtr]::Zero) { break }
                Start-Sleep -Milliseconds 150
            }
            if ($hwnd -eq [IntPtr]::Zero) {
                throw "Could not locate visible MOTAA Desktop window for PID $($proc.Id)"
            }
            [Win32Capture]::ShowWindow($hwnd, [Win32Capture]::SW_RESTORE) | Out-Null
            [Win32Capture]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, $Width, $Height, [Win32Capture]::SWP_SHOWWINDOW) | Out-Null
            [Win32Capture]::SetForegroundWindow($hwnd) | Out-Null
            Start-Sleep -Milliseconds 350

            $pngPath = Join-Path $resolvedOut "$slug.png"
            $records += Capture-WindowPng -Hwnd $hwnd -Path $pngPath

            $donePath = Join-Path $resolvedOut "$slug.done"
            Set-Content -Path $donePath -Value "ok" -Encoding ASCII
        }
    }
    if (-not $proc.WaitForExit(30000)) {
        $proc.Kill()
        throw "motaa-desktop did not exit cleanly after screenshot capture."
    }
    if ($proc.ExitCode -ne 0) {
        $stderr = $proc.StandardError.ReadToEnd()
        throw "motaa-desktop exited with code $($proc.ExitCode): $stderr"
    }
} finally {
    if (-not $proc.HasExited) {
        $proc.Kill()
    }
    Get-ChildItem -Path $resolvedOut -Filter "*.done" -ErrorAction SilentlyContinue | Remove-Item -Force
}

if ($records.Count -lt 6) {
    throw "Expected 6 Microsoft Store screenshots, but captured $($records.Count)."
}

$manifest = [ordered]@{
    platform = "windows-x64"
    count = $records.Count
    screenshots = $records
}
$manifestPath = Join-Path $resolvedOut "screenshots-manifest.json"
$manifest | ConvertTo-Json -Depth 4 | Set-Content -Path $manifestPath -Encoding UTF8
Write-Host "Successfully captured $($records.Count) Windows Store screenshots into $resolvedOut"
