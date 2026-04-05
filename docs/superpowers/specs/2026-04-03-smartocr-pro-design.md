# SmartOCR Pro Design

**Date:** 2026-04-03

**Goal:** Build a new Windows desktop application, `SmartOCR Pro`, delivered first as a portable `.zip` package that works offline for OCR and supports optional DeepSeek-based AI structuring.

## 1. Scope

### In Scope for V1

- Build a new desktop app from scratch using `Tauri 2 + Vue 3 + TypeScript + Rust`
- Windows 10/11 64-bit as the primary target
- Portable green package delivery: `.zip`, unzip and run
- Offline OCR with bundled runtime and models
- Local image upload and drag-and-drop
- Clipboard image paste/import
- Image direct-link URL import
- OCR result display as plain text
- Async OCR execution without freezing the window
- DeepSeek-based AI structuring
- Custom prompt input
- Markdown rendering of AI output
- Copy structured result as rich HTML for Word paste
- Append OCR or AI result to an accumulator area
- Export accumulator content as `.md` or `.txt`
- Export current OCR result as `.txt`
- Export current AI result as `.md`
- Secure local storage of `DeepSeek API Key`
- Reserve a `Web OCR` button as a future extension entry

### Out of Scope for V1

- Generic webpage parsing or extracting images from arbitrary HTML pages
- Multi-model provider support
- `.docx` generation
- Windows installer `.exe`
- A complete browser-based OCR product or backend service
- macOS release packaging in V1

## 2. Product Behavior

### 2.1 Primary User Flows

1. User imports an image from local file, clipboard, or image direct-link URL.
2. The app renders an image preview and stores the current image as the active source.
3. User starts OCR.
4. Rust backend performs OCR in a blocking worker thread and returns normalized text.
5. User can:
   - read the OCR text
   - export the OCR text as `.txt`
   - append the OCR text to the accumulator
   - send OCR text plus prompt to DeepSeek for structuring
6. AI result is rendered as Markdown/HTML in the UI.
7. User can:
   - copy AI result as rich text for Word paste
   - export AI result as `.md`
   - append AI result to the accumulator
   - export the accumulator as `.md` or `.txt`

### 2.2 Future Expansion Flow

- The `Web OCR` button opens a configurable web URL.
- In V1 this is only an extension entry point, not a complete web OCR workflow.
- The button must be isolated in implementation so a future web product can be added without refactoring the OCR or AI pipeline.

## 3. Architecture

### 3.1 Stack

- Desktop shell: `Tauri 2`
- Frontend: `Vue 3`, `TypeScript`, `Vite`
- UI library: `Naive UI`
- Markdown render: `marked` + `DOMPurify`
- Backend: `Rust 2021`
- Async runtime: `Tokio`
- OCR engine: `rapidocr-onnxruntime`
- HTTP client: `reqwest`
- Credential storage: `keyring`

### 3.2 Logical Modules

#### Frontend

- `image-input`
  - file picker, drag-and-drop, clipboard import trigger, URL input
- `image-preview`
  - render current image, zoom and pan support
- `ocr-result`
  - display plain text OCR output
- `ai-result`
  - render Markdown output safely
- `accumulator`
  - append/edit/export combined notes
- `settings`
  - DeepSeek API key input and future web URL configuration
- `actions`
  - OCR, AI structuring, copy rich text, export current result, export accumulator, open web portal

#### Rust Backend

- `image_io`
  - validate input image bytes and normalize supported formats
- `clipboard_bridge`
  - load clipboard image data when requested
- `url_import`
  - fetch only direct image URLs and validate content type
- `ocr_service`
  - load runtime/model assets and execute OCR using `spawn_blocking`
- `ai_service`
  - load API key from keyring and call DeepSeek
- `export_service`
  - save OCR/AI/accumulator files via file dialog
- `settings_service`
  - save and read local settings
- `portal_service`
  - open the future web OCR page in the system browser

## 4. Data Contracts

### 4.1 Active Image

```ts
type ActiveImageSource = {
  id: string
  sourceType: 'file' | 'clipboard' | 'url'
  displayName: string
  mimeType: string
  previewUrl: string
  byteLength: number
}
```

### 4.2 OCR Result

```ts
type OcrResult = {
  text: string
  durationMs: number
  sourceId: string
}
```

### 4.3 AI Result

```ts
type AiResult = {
  markdown: string
  durationMs: number
  sourceId: string
}
```

### 4.4 Settings

```ts
type AppSettings = {
  deepseekApiKeySaved: boolean
  defaultPrompt: string
  webPortalUrl: string
}
```

## 5. Error Handling

### Import Errors

- Unsupported format: show a clear error and keep the last successful image
- Clipboard without image: show `No image detected in clipboard`
- URL import failure: show a direct-link-specific error for timeout, non-image content, invalid certificate, redirect failure, or unsupported content type

### OCR Errors

- Missing runtime/model assets: show a startup/runtime asset error
- Image decode failure: show an image decode error
- Worker execution failure: show OCR execution failed and allow retry

### AI Errors

- Missing API key: prompt user to open settings
- Network failure: keep OCR text and prompt intact
- API error or rate limit: surface readable backend error text

### Export Errors

- Clipboard write failure: show copy failure
- File save failure: show path write failure and preserve in-memory content

## 6. Security

- Frontend never sends DeepSeek requests directly
- API key is stored using OS credential storage through `keyring`
- AI requests originate only from Rust backend
- Markdown must be sanitized before rendering
- URL import must restrict to direct image content and reject generic HTML pages in V1

## 7. Portable Package Design

### 7.1 Delivery Form

- Primary release artifact: `SmartOCR-Pro-windows-portable.zip`
- User flow:
  1. Download zip
  2. Extract the full directory
  3. Run `SmartOCR Pro.exe`

### 7.2 Portable Directory Layout

```text
SmartOCR-Pro/
  SmartOCR Pro.exe
  resources/
    ocr/
      runtime/
      models/
  README-PORTABLE.txt
```

### 7.3 Offline Requirement

- OCR runtime and models must ship with the portable package
- No first-run model download is allowed
- The app must fail clearly if the user removes required runtime/model files from the extracted directory

### 7.4 Package Size and Runtime Impact

- Bundling runtime and models increases disk footprint, not normal idle memory in a proportional way
- Expected extra package size for offline OCR assets is roughly `20MB - 35MB`
- This affects download size and extracted directory size, but not the basic usage model
- OCR memory use rises during recognition, which is acceptable for a desktop OCR workflow

## 8. UX Notes

- Main layout: left preview, right tabbed result panel, bottom accumulator
- Preview should support zoom and pan
- OCR and AI actions show busy state and do not block window interaction
- Result actions are explicit:
  - copy structured result
  - export current OCR
  - export current AI
  - append current OCR
  - append current AI
  - export accumulator

## 9. Acceptance Criteria

### Functional

- Import image from local file, clipboard, and direct image URL
- Run OCR fully offline after extraction with no model download
- Display OCR result in plain text
- Submit OCR text plus prompt to DeepSeek and render returned Markdown
- Copy AI result to clipboard as rich HTML and plain text fallback
- Export current OCR as `.txt`
- Export current AI result as `.md`
- Append OCR and AI results to the accumulator independently
- Export accumulator as `.md` and `.txt`
- Open future web portal URL from the dedicated button

### Non-Functional

- UI remains responsive while OCR runs
- API key is not exposed in frontend code or network panel
- Portable package can be used after unzip without installation
- Missing portable resources produce actionable error messages

## 10. Implementation Constraints

- Prefer stable Windows behavior over aggressive package-size optimization
- Design must preserve a future path to:
  - Windows installer packaging
  - macOS packaging
  - browser-side web OCR portal integration
- Keep `Web OCR` entry isolated so V2 can replace the placeholder target without major refactor
