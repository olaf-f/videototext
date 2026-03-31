@echo off
setlocal
set "TARGET=%LocalAppData%\Programs\VideoToText"
del /F /Q "%USERPROFILE%\Desktop\VideoToText.lnk" >nul 2>nul
del /F /Q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\VideoToText.lnk" >nul 2>nul
del /F /Q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\VideoToText Uninstall.lnk" >nul 2>nul
if exist "%TARGET%\uninstall.cmd" del /F /Q "%TARGET%\uninstall.cmd" >nul 2>nul
if exist "%TARGET%\VideoToText.exe" del /F /Q "%TARGET%\VideoToText.exe" >nul 2>nul
if exist "%TARGET%" rmdir "%TARGET%" >nul 2>nul
echo VideoToText has been removed.
pause
endlocal
