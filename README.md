# SmartOCR Pro

SmartOCR Pro is a Tauri 2 + Vue 3 Windows desktop application for offline OCR, DeepSeek-based text structuring, and portable export workflows.

## Scope

- Drag-and-drop + picker-based image import (up to 30 images per batch), with drag sorting
- Optional direct image URL import (auto-hidden after selecting local images)
- Offline OCR with bundled runtime/model assets
- DeepSeek structuring through the Rust backend
- Markdown rendering, Word-friendly rich copy, and accumulator export
- Windows portable `.zip` packaging first

## Prerequisites

- Node.js 20+
- `pnpm`
- Rust toolchain with `cargo`
- Windows native build tools for Tauri: Visual Studio Build Tools with MSVC v143+ and the Windows 10/11 SDK

## Development commands

```powershell
pnpm install
pnpm dev
pnpm build
pnpm typecheck
cargo test --manifest-path src-tauri/Cargo.toml
```

## Portable package

Build the portable package:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\package-portable.ps1
```

Prepare OCR runtime/model assets before packaging:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\download-ocr-assets.ps1
```

Build the NSIS installer:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-installer.ps1
```

Artifacts are emitted to:
- `release/portable/SmartOCR-Pro-windows-portable.zip`
- `release/installer/SmartOCR-Pro-setup.exe`

## Portable enablement

1. Extract the full portable zip into a writable folder.
2. Keep `SmartOCR Pro.exe` and `resources/ocr` in the same extracted directory tree.
3. Open Settings and save a `DeepSeek API Key` before using AI structuring.
4. Launch `SmartOCR Pro.exe`.

## Notes

- Bundle icons include both `src-tauri/icons/icon.ico` and `src-tauri/icons/icon.svg`.
