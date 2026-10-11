# ==============================================================================
# Kora — Script tự động tải thư mục switches từ GitHub về thư mục config
# ==============================================================================

[CmdletBinding()]
param(
    [string]$RepoZipUrl = "https://github.com/uongsuadaubung/kora/archive/refs/heads/main.zip"
)

$ErrorActionPreference = "Stop"

Write-Host "======================================================" -ForegroundColor Cyan
Write-Host "   Kora — Tự động cài đặt gói âm thanh phím cơ        " -ForegroundColor Cyan
Write-Host "======================================================" -ForegroundColor Cyan

$configDir = Join-Path $env:USERPROFILE ".config\kora"
$switchesDir = Join-Path $configDir "switches"
$tempGuid = [guid]::NewGuid().ToString("N")
$tempZip = Join-Path $env:TEMP "kora-repo-$tempGuid.zip"
$tempExtract = Join-Path $env:TEMP "kora-extract-$tempGuid"

try {
    # 1. Tạo thư mục cấu hình nếu chưa tồn tại
    if (-not (Test-Path $configDir)) {
        Write-Host "[1/3] Đang tạo thư mục cấu hình: $configDir..." -ForegroundColor Yellow
        New-Item -ItemType Directory -Force -Path $configDir | Out-Null
    }

    # 2. Tải repo từ GitHub (chứa thư mục switches)
    Write-Host "[2/3] Đang tải thư mục switches từ GitHub repo..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $RepoZipUrl -OutFile $tempZip -UseBasicParsing

    # 3. Giải nén và trích xuất thư mục switches vào config
    Write-Host "[3/3] Đang nạp 13 bộ switch vào $switchesDir..." -ForegroundColor Yellow
    Expand-Archive -Path $tempZip -DestinationPath $tempExtract -Force

    $foundSwitches = Get-ChildItem -Path $tempExtract -Filter "switches" -Directory -Recurse | Select-Object -First 1
    if (-not $foundSwitches) {
        throw "Không tìm thấy thư mục switches trong repo tải về."
    }

    if (Test-Path $switchesDir) {
        Remove-Item $switchesDir -Recurse -Force
    }

    Copy-Item -Path $foundSwitches.FullName -Destination $configDir -Recurse -Force

    Write-Host ""
    Write-Host "======================================================" -ForegroundColor Green
    Write-Host "   ĐÃ CÀI ĐẶT THÀNH CÔNG GÓI ÂM THANH CHO KORA!       " -ForegroundColor Green
    Write-Host "======================================================" -ForegroundColor Green
    Write-Host "Đường dẫn: $switchesDir" -ForegroundColor Gray
    Write-Host "Bây giờ bạn chỉ cần mở Bảng điều khiển Kora, bật 'Âm thanh gõ phím cơ' và trải nghiệm!" -ForegroundColor Cyan
    Write-Host ""
}
catch {
    Write-Host ""
    Write-Host "❌ Đã có lỗi xảy ra trong quá trình cài đặt:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}
finally {
    # Dọn dẹp file tạm
    if (Test-Path $tempZip) {
        Remove-Item $tempZip -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $tempExtract) {
        Remove-Item $tempExtract -Recurse -Force -ErrorAction SilentlyContinue
    }
}
