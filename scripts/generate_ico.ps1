Add-Type -AssemblyName System.Drawing

function Optimize-Png([byte[]]$pngBytes) {
    $ms = New-Object System.IO.MemoryStream
    $ms.Write($pngBytes, 0, 8)
    
    $i = 8
    while ($i -lt $pngBytes.Length) {
        $len = [BitConverter]::ToUInt32(@($pngBytes[$i+3], $pngBytes[$i+2], $pngBytes[$i+1], $pngBytes[$i]), 0)
        $type = [System.Text.Encoding]::ASCII.GetString($pngBytes[($i+4)..($i+7)])
        $chunkTotal = 8 + $len + 4
        if ($type -eq "IHDR" -or $type -eq "IDAT" -or $type -eq "IEND") {
            $ms.Write($pngBytes, $i, $chunkTotal)
        }
        $i += $chunkTotal
    }
    return $ms.ToArray()
}

function Create-Ico([byte[][]]$pngImages, [int[]]$dimensions) {
    $ms = New-Object System.IO.MemoryStream
    $bw = New-Object System.IO.BinaryWriter($ms)

    # 1. ICONDIR (6 bytes)
    $bw.Write([uint16]0) # Reserved
    $bw.Write([uint16]1) # Type 1 = Icon
    $bw.Write([uint16]$pngImages.Length) # Image count

    # 2. ICONDIRENTRY (16 bytes per image)
    $offset = 6 + $pngImages.Length * 16
    for ($i = 0; $i -lt $pngImages.Length; $i++) {
        $dim = $dimensions[$i]
        $bDim = if ($dim -ge 256) { [byte]0 } else { [byte]$dim }
        $bw.Write($bDim) # Width
        $bw.Write($bDim) # Height
        $bw.Write([byte]0) # Color count
        $bw.Write([byte]0) # Reserved
        $bw.Write([uint16]1) # Color planes
        $bw.Write([uint16]32) # Bits per pixel
        $bw.Write([uint32]$pngImages[$i].Length) # Image size in bytes
        $bw.Write([uint32]$offset) # Image offset
        $offset += $pngImages[$i].Length
    }

    # 3. Image data
    for ($i = 0; $i -lt $pngImages.Length; $i++) {
        $bw.Write($pngImages[$i])
    }

    $bytes = $ms.ToArray()
    $bw.Dispose()
    $ms.Dispose()
    return $bytes
}

function Build-PixelSharpIcon([int]$dim) {
    $bmp = New-Object System.Drawing.Bitmap($dim, $dim, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

    $cWhite = [System.Drawing.Color]::FromArgb(255, 255, 255, 255)
    $cWhiteSub = [System.Drawing.Color]::FromArgb(210, 255, 255, 255)
    $cBg = [System.Drawing.Color]::FromArgb(255, 22, 24, 31)

    # Base squircle from 256
    $base256 = [System.Drawing.Bitmap]::FromFile((Resolve-Path "landing-page/images/icon.png"))
    $g256 = [System.Drawing.Graphics]::FromImage($base256)
    $bgBrush = New-Object System.Drawing.SolidBrush($cBg)
    $g256.FillRectangle($bgBrush, 45, 45, 166, 166)
    $g256.Dispose()
    $bgBrush.Dispose()

    $g.DrawImage($base256, 0, 0, $dim, $dim)
    $base256.Dispose()
    $g.Dispose()

    if ($dim -eq 16) {
        # 16x16: 100% DPI small title & tray
        for ($y = 3; $y -le 12; $y++) {
            for ($x = 3; $x -le 12; $x++) { $bmp.SetPixel($x, $y, $cBg) }
        }
        for ($y = 3; $y -le 12; $y++) {
            $bmp.SetPixel(4, $y, $cWhite)
            $bmp.SetPixel(5, $y, $cWhite)
        }
        $bmp.SetPixel(6, 7, $cWhite)
        $bmp.SetPixel(7, 6, $cWhite);  $bmp.SetPixel(8, 6, $cWhiteSub)
        $bmp.SetPixel(8, 5, $cWhite);  $bmp.SetPixel(9, 5, $cWhiteSub)
        $bmp.SetPixel(9, 4, $cWhite);  $bmp.SetPixel(10, 4, $cWhiteSub)
        $bmp.SetPixel(10, 3, $cWhite); $bmp.SetPixel(11, 3, $cWhite)

        $bmp.SetPixel(6, 8, $cWhite);  $bmp.SetPixel(7, 8, $cWhiteSub)
        $bmp.SetPixel(7, 9, $cWhite);  $bmp.SetPixel(8, 9, $cWhiteSub)
        $bmp.SetPixel(8, 10, $cWhite); $bmp.SetPixel(9, 10, $cWhiteSub)
        $bmp.SetPixel(9, 11, $cWhite); $bmp.SetPixel(10, 11, $cWhiteSub)
        $bmp.SetPixel(10, 12, $cWhite); $bmp.SetPixel(11, 12, $cWhite)
    }
    elseif ($dim -eq 20) {
        # 20x20: 125% DPI small title & tray
        for ($y = 3; $y -le 16; $y++) {
            for ($x = 4; $x -le 16; $x++) { $bmp.SetPixel($x, $y, $cBg) }
        }
        for ($y = 4; $y -le 15; $y++) {
            $bmp.SetPixel(5, $y, $cWhite)
            $bmp.SetPixel(6, $y, $cWhite)
            $bmp.SetPixel(7, $y, $cWhite)
        }
        for ($step = 0; $step -le 5; $step++) {
            $x = 8 + $step
            $y = 9 - $step
            if ($y -ge 4) {
                $bmp.SetPixel($x, $y, $cWhite)
                $bmp.SetPixel($x + 1, $y, $cWhite)
                if ($x + 2 -le 15) { $bmp.SetPixel($x + 2, $y, $cWhiteSub) }
            }
        }
        for ($step = 0; $step -le 6; $step++) {
            $x = 8 + $step
            $y = 10 + $step
            if ($y -le 15) {
                $bmp.SetPixel($x, $y, $cWhite)
                $bmp.SetPixel($x + 1, $y, $cWhite)
                if ($x + 2 -le 15) { $bmp.SetPixel($x + 2, $y, $cWhiteSub) }
            }
        }
    }
    elseif ($dim -eq 24) {
        # 24x24: 150% DPI small title & tray
        for ($y = 4; $y -le 19; $y++) {
            for ($x = 5; $x -le 19; $x++) { $bmp.SetPixel($x, $y, $cBg) }
        }
        for ($y = 5; $y -le 19; $y++) {
            $bmp.SetPixel(6, $y, $cWhite)
            $bmp.SetPixel(7, $y, $cWhite)
            $bmp.SetPixel(8, $y, $cWhite)
        }
        for ($step = 0; $step -le 7; $step++) {
            $x = 9 + $step
            $y = 11 - $step
            if ($y -ge 5 -and $x -le 17) {
                $bmp.SetPixel($x, $y, $cWhite)
                $bmp.SetPixel($x + 1, $y, $cWhite)
                if ($x + 2 -le 18) { $bmp.SetPixel($x + 2, $y, $cWhiteSub) }
            }
        }
        for ($step = 0; $step -le 7; $step++) {
            $x = 9 + $step
            $y = 12 + $step
            if ($y -le 19 -and $x -le 17) {
                $bmp.SetPixel($x, $y, $cWhite)
                $bmp.SetPixel($x + 1, $y, $cWhite)
                if ($x + 2 -le 18) { $bmp.SetPixel($x + 2, $y, $cWhiteSub) }
            }
        }
    }
    elseif ($dim -eq 32) {
        # 32x32: 200% DPI small / 100% large
        for ($y = 5; $y -le 26; $y++) {
            for ($x = 7; $x -le 25; $x++) { $bmp.SetPixel($x, $y, $cBg) }
        }
        for ($y = 6; $y -le 25; $y++) {
            $bmp.SetPixel(8, $y, $cWhite)
            $bmp.SetPixel(9, $y, $cWhite)
            $bmp.SetPixel(10, $y, $cWhite)
            $bmp.SetPixel(11, $y, $cWhite)
        }
        for ($step = 0; $step -le 10; $step++) {
            $x = 12 + $step
            $y = 15 - $step
            if ($y -ge 6 -and $x -le 23) {
                $bmp.SetPixel($x, $y, $cWhite)
                $bmp.SetPixel($x + 1, $y, $cWhite)
                $bmp.SetPixel($x + 2, $y, $cWhite)
                if ($x + 3 -le 24) { $bmp.SetPixel($x + 3, $y, $cWhiteSub) }
            }
        }
        for ($step = 0; $step -le 10; $step++) {
            $x = 12 + $step
            $y = 16 + $step
            if ($y -le 25 -and $x -le 23) {
                $bmp.SetPixel($x, $y, $cWhite)
                $bmp.SetPixel($x + 1, $y, $cWhite)
                $bmp.SetPixel($x + 2, $y, $cWhite)
                if ($x + 3 -le 24) { $bmp.SetPixel($x + 3, $y, $cWhiteSub) }
            }
        }
    }
    elseif ($dim -eq 48) {
        # 48x48: 150% DPI large
        for ($y = 8; $y -le 39; $y++) {
            for ($x = 10; $x -le 38; $x++) { $bmp.SetPixel($x, $y, $cBg) }
        }
        for ($y = 9; $y -le 38; $y++) {
            for ($x = 12; $x -le 17; $x++) { $bmp.SetPixel($x, $y, $cWhite) }
        }
        for ($step = 0; $step -le 16; $step++) {
            $x = 18 + $step
            $y = 23 - $step
            if ($y -ge 9 -and $x -le 34) {
                $bmp.SetPixel($x, $y, $cWhite)
                $bmp.SetPixel($x + 1, $y, $cWhite)
                $bmp.SetPixel($x + 2, $y, $cWhite)
                $bmp.SetPixel($x + 3, $y, $cWhite)
                if ($x + 4 -le 35) { $bmp.SetPixel($x + 4, $y, $cWhiteSub) }
            }
        }
        for ($step = 0; $step -le 16; $step++) {
            $x = 18 + $step
            $y = 24 + $step
            if ($y -le 38 -and $x -le 34) {
                $bmp.SetPixel($x, $y, $cWhite)
                $bmp.SetPixel($x + 1, $y, $cWhite)
                $bmp.SetPixel($x + 2, $y, $cWhite)
                $bmp.SetPixel($x + 3, $y, $cWhite)
                if ($x + 4 -le 35) { $bmp.SetPixel($x + 4, $y, $cWhiteSub) }
            }
        }
    }

    return $bmp
}

# 1. 256x256 Base Image
$raw256 = [System.IO.File]::ReadAllBytes("landing-page/images/icon.png")
$opt256 = Optimize-Png $raw256

$pngList = New-Object 'System.Collections.Generic.List[byte[]]'
$dimList = New-Object 'System.Collections.Generic.List[int]'

$pngList.Add($opt256)
$dimList.Add(256)

# 2. Render and optimize sizes 48, 32, 24, 20, 16
$sizes = @(48, 32, 24, 20, 16)
foreach ($sz in $sizes) {
    $bmp = Build-PixelSharpIcon $sz
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $rawBytes = $ms.ToArray()
    $ms.Dispose()
    $bmp.Dispose()

    $optBytes = Optimize-Png $rawBytes
    $pngList.Add($optBytes)
    $dimList.Add($sz)
    Write-Host "Size ${sz}x${sz} size: $($optBytes.Length) bytes"
}

# 3. Create .ico
$icoBytes = Create-Ico -pngImages ($pngList.ToArray()) -dimensions ($dimList.ToArray())
Write-Host ">>> Final Ultra-Sharp ICO total length: $($icoBytes.Length) bytes <<<"

[System.IO.File]::WriteAllBytes("assets/icon.ico", $icoBytes)
[System.IO.File]::WriteAllBytes("landing-page/favicon.ico", $icoBytes)

Write-Host "Successfully generated razor-sharp multi-resolution icon.ico!"
