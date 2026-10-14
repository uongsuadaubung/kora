@echo off
chcp 65001 >nul
cd /d "%~dp0"

echo [1/2] Building benchmark in release mode (opt-level=3, LTO=true)...
cargo build --release --example benchmark
if %errorlevel% neq 0 (
    echo [ERROR] Build failed!
    exit /b %errorlevel%
)

echo.
echo [2/2] Running target\release\examples\benchmark.exe...
timeout /t 1 /nobreak >nul
echo.
"target\release\examples\benchmark.exe"

