@echo off
setlocal
set "TARGET=%LocalAppData%\Programs\VideoToText"
if not exist "%TARGET%" mkdir "%TARGET%"
copy /Y "VideoToText.exe" "%TARGET%\VideoToText.exe" >nul
copy /Y "uninstall.cmd" "%TARGET%\uninstall.cmd" >nul
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ws=New-Object -ComObject WScript.Shell; $desktop=[System.IO.Path]::Combine($env:USERPROFILE,'Desktop','VideoToText.lnk'); $app=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Programs','VideoToText','VideoToText.exe'); $work=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Programs','VideoToText'); $shortcut=$ws.CreateShortcut($desktop); $shortcut.TargetPath=$app; $shortcut.WorkingDirectory=$work; $shortcut.Save(); $startMenu=[System.IO.Path]::Combine($env:APPDATA,'Microsoft','Windows','Start Menu','Programs','VideoToText.lnk'); $shortcut2=$ws.CreateShortcut($startMenu); $shortcut2.TargetPath=$app; $shortcut2.WorkingDirectory=$work; $shortcut2.Save(); $uninstall=[System.IO.Path]::Combine($env:APPDATA,'Microsoft','Windows','Start Menu','Programs','VideoToText Uninstall.lnk'); $shortcut3=$ws.CreateShortcut($uninstall); $shortcut3.TargetPath=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Programs','VideoToText','uninstall.cmd'); $shortcut3.WorkingDirectory=$work; $shortcut3.Save()"
start "" "%TARGET%\VideoToText.exe"
endlocal
