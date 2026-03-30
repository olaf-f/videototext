from __future__ import annotations

import threading
import traceback
from pathlib import Path
from tkinter import END, BOTH, LEFT, RIGHT, X, filedialog, messagebox, Tk, StringVar
from tkinter import ttk
from tkinter.scrolledtext import ScrolledText

from videototext.engine import run_ocr, run_transcribe_file, run_transcribe_url
from videototext.exporter import export_docx, export_txt
from videototext.models import AppConfig, UrlInput
from videototext.utils import timestamp


class VideoToTextApp:
    def __init__(self, root: Tk) -> None:
        self.root = root
        self.root.title("VideoToText")
        self.root.geometry("980x720")

        self.image_path = StringVar()
        self.video_path = StringVar()
        self.url = StringVar()
        self.cookie_file = StringVar()
        self.model_size = StringVar(value="base")
        self.language = StringVar(value="")

        self.status = StringVar(value="Ready")
        self._build_ui()

    def _build_ui(self) -> None:
        top = ttk.Frame(self.root, padding=8)
        top.pack(fill=X)

        ttk.Label(top, text="Model").pack(side=LEFT)
        ttk.Combobox(top, textvariable=self.model_size, values=["tiny", "base", "small", "medium"], width=12).pack(side=LEFT, padx=4)
        ttk.Label(top, text="Language(optional)").pack(side=LEFT, padx=(12, 0))
        ttk.Entry(top, textvariable=self.language, width=14).pack(side=LEFT, padx=4)
        ttk.Label(top, textvariable=self.status).pack(side=RIGHT)

        tabs = ttk.Notebook(self.root)
        tabs.pack(fill=BOTH, expand=True, padx=8, pady=8)

        self.tab_image = ttk.Frame(tabs, padding=10)
        self.tab_video = ttk.Frame(tabs, padding=10)
        self.tab_url = ttk.Frame(tabs, padding=10)
        tabs.add(self.tab_image, text="图片 OCR")
        tabs.add(self.tab_video, text="本地视频")
        tabs.add(self.tab_url, text="链接 + Cookie")

        self._build_image_tab()
        self._build_video_tab()
        self._build_url_tab()

        output_frame = ttk.LabelFrame(self.root, text="识别结果", padding=8)
        output_frame.pack(fill=BOTH, expand=True, padx=8, pady=(0, 8))

        self.output = ScrolledText(output_frame, wrap="word", height=14)
        self.output.pack(fill=BOTH, expand=True)

        btn_bar = ttk.Frame(self.root, padding=8)
        btn_bar.pack(fill=X)
        ttk.Button(btn_bar, text="导出 TXT", command=self.export_txt_action).pack(side=LEFT)
        ttk.Button(btn_bar, text="导出 DOCX", command=self.export_docx_action).pack(side=LEFT, padx=8)

        log_frame = ttk.LabelFrame(self.root, text="日志", padding=8)
        log_frame.pack(fill=BOTH, expand=False, padx=8, pady=(0, 8))
        self.logs = ScrolledText(log_frame, height=8, state="disabled")
        self.logs.pack(fill=BOTH, expand=True)

    def _build_image_tab(self) -> None:
        row = ttk.Frame(self.tab_image)
        row.pack(fill=X)
        ttk.Entry(row, textvariable=self.image_path).pack(side=LEFT, fill=X, expand=True)
        ttk.Button(row, text="选择图片", command=self.pick_image).pack(side=LEFT, padx=8)
        ttk.Button(self.tab_image, text="开始 OCR", command=self.run_image).pack(anchor="w", pady=10)

    def _build_video_tab(self) -> None:
        row = ttk.Frame(self.tab_video)
        row.pack(fill=X)
        ttk.Entry(row, textvariable=self.video_path).pack(side=LEFT, fill=X, expand=True)
        ttk.Button(row, text="选择视频", command=self.pick_video).pack(side=LEFT, padx=8)
        ttk.Button(self.tab_video, text="开始转写", command=self.run_video).pack(anchor="w", pady=10)

    def _build_url_tab(self) -> None:
        row1 = ttk.Frame(self.tab_url)
        row1.pack(fill=X)
        ttk.Label(row1, text="视频链接").pack(side=LEFT)
        ttk.Entry(row1, textvariable=self.url).pack(side=LEFT, fill=X, expand=True, padx=8)

        row2 = ttk.Frame(self.tab_url)
        row2.pack(fill=X, pady=8)
        ttk.Label(row2, text="Cookie文件").pack(side=LEFT)
        ttk.Entry(row2, textvariable=self.cookie_file).pack(side=LEFT, fill=X, expand=True, padx=8)
        ttk.Button(row2, text="选择", command=self.pick_cookie).pack(side=LEFT)

        ttk.Label(self.tab_url, text="或粘贴 Cookie 文本").pack(anchor="w")
        self.cookie_text = ScrolledText(self.tab_url, height=8)
        self.cookie_text.pack(fill=BOTH, expand=True, pady=6)
        ttk.Button(self.tab_url, text="开始链接转写", command=self.run_url).pack(anchor="w", pady=6)

    def pick_image(self) -> None:
        path = filedialog.askopenfilename(filetypes=[("Image", "*.png;*.jpg;*.jpeg;*.webp")])
        if path:
            self.image_path.set(path)

    def pick_video(self) -> None:
        path = filedialog.askopenfilename(filetypes=[("Video", "*.mp4;*.mov;*.mkv;*.avi")])
        if path:
            self.video_path.set(path)

    def pick_cookie(self) -> None:
        path = filedialog.askopenfilename(filetypes=[("Text", "*.txt")])
        if path:
            self.cookie_file.set(path)

    def cfg(self) -> AppConfig:
        lang = self.language.get().strip() or None
        return AppConfig(model_size=self.model_size.get().strip() or "base", language=lang)

    def set_text(self, text: str) -> None:
        self.output.delete("1.0", END)
        self.output.insert(END, text)

    def append_log(self, text: str) -> None:
        self.logs.configure(state="normal")
        self.logs.insert(END, text + "\n")
        self.logs.configure(state="disabled")
        self.logs.see(END)

    def run_worker(self, action_name: str, target) -> None:
        self.status.set(f"Running: {action_name}")
        self.append_log(f"[{timestamp()}] Start {action_name}")

        def job() -> None:
            try:
                result = target()
                self.root.after(0, lambda: self._on_success(action_name, result))
            except Exception as exc:
                detail = "\n".join(traceback.format_exception(exc))
                self.root.after(0, lambda: self._on_error(action_name, str(exc), detail))

        threading.Thread(target=job, daemon=True).start()

    def _on_success(self, action_name: str, text: str) -> None:
        self.set_text(text)
        self.status.set("Done")
        self.append_log(f"[{timestamp()}] Done {action_name}")

    def _on_error(self, action_name: str, err: str, detail: str) -> None:
        self.status.set("Failed")
        self.append_log(f"[{timestamp()}] Failed {action_name}: {err}")
        self.append_log(detail)
        messagebox.showerror("Error", err)

    def run_image(self) -> None:
        image = self.image_path.get().strip()
        if not image:
            messagebox.showwarning("提示", "请先选择图片")
            return
        self.run_worker("OCR", lambda: run_ocr(Path(image)))

    def run_video(self) -> None:
        video = self.video_path.get().strip()
        if not video:
            messagebox.showwarning("提示", "请先选择视频")
            return
        cfg = self.cfg()
        self.run_worker("Transcribe file", lambda: run_transcribe_file(Path(video), cfg))

    def run_url(self) -> None:
        url = self.url.get().strip()
        if not url:
            messagebox.showwarning("提示", "请填写视频链接")
            return

        cookie_text = self.cookie_text.get("1.0", END).strip()
        cookie_file = Path(self.cookie_file.get().strip()) if self.cookie_file.get().strip() else None
        data = UrlInput(url=url, cookie_text=cookie_text, cookie_file=cookie_file)
        cfg = self.cfg()
        self.run_worker("Transcribe URL", lambda: run_transcribe_url(data, cfg))

    def export_txt_action(self) -> None:
        text = self.output.get("1.0", END).strip()
        if not text:
            messagebox.showwarning("提示", "没有可导出的文本")
            return
        path = filedialog.asksaveasfilename(defaultextension=".txt", filetypes=[("Text", "*.txt")])
        if not path:
            return
        export_txt(text, Path(path))
        self.append_log(f"[{timestamp()}] Exported TXT: {path}")
        messagebox.showinfo("完成", "TXT 已导出")

    def export_docx_action(self) -> None:
        text = self.output.get("1.0", END).strip()
        if not text:
            messagebox.showwarning("提示", "没有可导出的文本")
            return
        path = filedialog.asksaveasfilename(defaultextension=".docx", filetypes=[("Word", "*.docx")])
        if not path:
            return
        export_docx(text, Path(path))
        self.append_log(f"[{timestamp()}] Exported DOCX: {path}")
        messagebox.showinfo("完成", "DOCX 已导出")


def run_app() -> None:
    root = Tk()
    app = VideoToTextApp(root)
    app.append_log("VideoToText started")
    root.mainloop()
