# VideoToText Desktop (MVP)

跨平台桌面工具（Windows/macOS）：
- 图片 OCR 提取文字
- 本地视频提取语音转文字
- 视频链接 + Cookie 下载后转文字
- 导出 TXT / DOCX

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

## 4. 打包 EXE / APP（本地）
```bash
pip install pyinstaller
pyinstaller --noconfirm --windowed --name VideoToText --paths src main.py
```

生成目录：`dist/VideoToText`

## 5. 注意事项
- 首次运行 Whisper 模型会自动下载，速度取决于网络。
- 链接下载依赖 `yt-dlp` 支持的平台与内容权限。
- Cookie 仅在本地任务中临时使用。

## 6. 下一步规划
- 增加任务队列与取消。
- 增加 SRT 导出。
- 增加 Tauri 前端替换 tkinter UI。
