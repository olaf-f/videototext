@echo off
setlocal
set "TARGET=%LocalAppData%\Programs\VideoToText"
if not exist "%TARGET%" mkdir "%TARGET%"
copy /Y "VideoToText.exe" "%TARGET%\VideoToText.exe" >nul
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ws=New-Object -ComObject WScript.Shell; $shortcut=$ws.CreateShortcut([System.IO.Path]::Combine($env:USERPROFILE,'Desktop','VideoToText.lnk')); $shortcut.TargetPath=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Programs','VideoToText','VideoToText.exe'); $shortcut.WorkingDirectory=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Programs','VideoToText'); $shortcut.Save(); $startMenu=[System.IO.Path]::Combine($env:APPDATA,'Microsoft','Windows','Start Menu','Programs','VideoToText.lnk'); $shortcut2=$ws.CreateShortcut($startMenu); $shortcut2.TargetPath=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Programs','VideoToText','VideoToText.exe'); $shortcut2.WorkingDirectory=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Programs','VideoToText'); $shortcut2.Save()"
start "" "%TARGET%\VideoToText.exe"
endlocal
