@echo off
setlocal enabledelayedexpansion

echo ======================================================
echo             Kora - Build Release Script
echo ======================================================

echo [1/4] Kiem tra tien trinh Kora.exe dang chay...
tasklist /FI "IMAGENAME eq Kora.exe" 2>NUL | find /I /N "Kora.exe" >NUL
if %ERRORLEVEL% equ 0 (
    echo [!] Phat hien Kora.exe dang chay. Dang tat tien trinh...
    taskkill /F /IM Kora.exe >NUL 2>&1
    ping 127.0.0.1 -n 2 >NUL
    echo [OK] Da tat Kora.exe thanh cong.
)

echo [OK] San sang bien dich.

echo.
echo [2/4] Dang build Kora (Release mode)...
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo.
    echo [!] Build that bai! Vui long kiem tra loi phia tren.
    exit /b %ERRORLEVEL%
)

echo.
echo [3/4] Build Release thanh cong!
echo Duong dan file thuc thi: target\release\Kora.exe

echo.
echo [4/4] Dang khoi chay Kora.exe...
start "" "target\release\Kora.exe"
echo [OK] Kora da duoc khoi chay thanh cong.
echo ======================================================

endlocal
