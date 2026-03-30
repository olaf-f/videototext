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

## 2. 安装依赖
```bash
pip install -r requirements.txt
```

## 3. 启动
```bash
python main.py
```

## 4. 打包（本地）
Windows PowerShell:
```powershell
./scripts/build.ps1
```

macOS:
```bash
bash ./scripts/build.sh
```

生成目录：`dist/VideoToText*`

## 5. GitHub Actions 自动构建
- 文件：`.github/workflows/build-desktop.yml`
- 触发：手动触发 / 推送 `v*` tag
- 平台：`windows-latest`、`macos-latest`

## 6. 注意事项
- 首次运行 Whisper 模型会自动下载，速度取决于网络。
- 链接下载依赖 `yt-dlp` 支持的平台与内容权限。
- Cookie 仅在本地任务中临时使用（写入临时目录）。
- SRT 导出依赖 ASR 时间戳，OCR 结果无法导出 SRT。
