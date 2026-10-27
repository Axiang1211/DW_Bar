@echo off
chcp 65001 >nul
title DW Bar - 開發工具安裝
echo.
echo  ==========================================
echo    DW Bar 開發工具一鍵安裝
echo    需要 10~30 分鐘，過程中請按「是」允許安裝
echo  ==========================================
echo.
where winget >nul 2>nul || (echo [錯誤] 找不到 winget，請先到 Microsoft Store 更新「應用程式安裝程式」& pause & exit /b 1)

echo [1/4] Visual Studio 2022 Build Tools (C++)...
winget install -e --id Microsoft.VisualStudio.2022.BuildTools --accept-package-agreements --accept-source-agreements --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

echo [2/4] Rust...
winget install -e --id Rustlang.Rustup --accept-package-agreements --accept-source-agreements
if exist "%USERPROFILE%\.cargo\bin\rustup.exe" "%USERPROFILE%\.cargo\bin\rustup.exe" default stable-msvc

echo [3/4] Node.js LTS...
winget install -e --id OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements

echo [4/4] GitHub Desktop...
winget install -e --id GitHub.GitHubDesktop --accept-package-agreements --accept-source-agreements

echo.
echo  完成！請「重新開機」一次，讓新工具生效。
echo.
pause
