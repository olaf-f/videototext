# VideoToText Desktop (MVP)

跨平台桌面工具（Windows/macOS）：
- 图片 OCR 提取文字
- 本地视频提取语音转文字
- 视频链接 + Cookie 下载后转文字
- 导出 TXT / DOCX / SRT
- 任务进度显示与取消

## 1. 环境要求
- Python 3.10+
- FFmpeg（必须在 PATH 中）

## 2. 本地运行
```bash
pip install -r requirements.txt
python main.py
```

## 3. 本地打包
Windows PowerShell:
```powershell
./scripts/build.ps1
```

macOS:
```bash
bash ./scripts/build.sh
```

## 4. GitHub 自动发布安装包
1. 推送代码到 `main`
2. 创建并推送 tag（例如 `v0.1.8`）
3. GitHub Actions 执行 `.github/workflows/build-desktop.yml`
4. 自动创建 Release，并附带：
- `VideoToText-windows-x64.exe`
- `VideoToText-windows-x64-setup.exe`
- `VideoToText-macos-x64.zip`

下载路径：仓库 `Releases` 页面。

## 5. 注意事项
- Windows 现在同时提供单文件 `exe` 和安装器 `setup.exe`。
- Windows `setup.exe` 会安装到 `%LocalAppData%\Programs\VideoToText`，并创建桌面/开始菜单快捷方式。
- Windows 开始菜单同时会创建 `VideoToText Uninstall` 快捷方式用于卸载。
- 首次运行 Whisper 模型会自动下载，速度取决于网络。
- 链接下载依赖 `yt-dlp` 支持的平台与内容权限。
- Cookie 仅在本地任务中临时使用（写入临时目录）。
- SRT 导出依赖 ASR 时间戳，OCR 结果无法导出 SRT。
