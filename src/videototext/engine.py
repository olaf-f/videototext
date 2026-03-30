from __future__ import annotations

from pathlib import Path
from typing import Callable, Optional

from videototext.asr import transcribe_audio
from videototext.downloader import download_video
from videototext.media import extract_audio
from videototext.models import AppConfig, TranscriptionResult, UrlInput
from videototext.ocr import ocr_image
from videototext.utils import make_workdir


def _check_cancel(is_cancelled: Optional[Callable[[], bool]]) -> None:
    if is_cancelled and is_cancelled():
        raise RuntimeError("Task cancelled")


def run_ocr(
    image_path: Path,
    progress_callback: Optional[Callable[[str, int], None]] = None,
    is_cancelled: Optional[Callable[[], bool]] = None,
) -> str:
    _check_cancel(is_cancelled)
    if progress_callback:
        progress_callback("OCR running", 30)
    text = ocr_image(image_path)
    _check_cancel(is_cancelled)
    if progress_callback:
        progress_callback("OCR done", 100)
    return text


def run_transcribe_file(
    video_path: Path,
    cfg: AppConfig,
    progress_callback: Optional[Callable[[str, int], None]] = None,
    is_cancelled: Optional[Callable[[], bool]] = None,
) -> TranscriptionResult:
    _check_cancel(is_cancelled)
    if progress_callback:
        progress_callback("Preparing workspace", 5)
    workdir = make_workdir()
    audio_path = workdir / "audio.wav"
    if progress_callback:
        progress_callback("Extracting audio", 20)
    extract_audio(video_path, audio_path)
    _check_cancel(is_cancelled)
    if progress_callback:
        progress_callback("Starting ASR", 45)
    return transcribe_audio(
        audio_path,
        model_size=cfg.model_size,
        compute_type=cfg.compute_type,
        language=cfg.language,
        progress_callback=progress_callback,
        is_cancelled=is_cancelled,
    )


def run_transcribe_url(
    data: UrlInput,
    cfg: AppConfig,
    progress_callback: Optional[Callable[[str, int], None]] = None,
    is_cancelled: Optional[Callable[[], bool]] = None,
) -> TranscriptionResult:
    _check_cancel(is_cancelled)
    if progress_callback:
        progress_callback("Preparing workspace", 5)
    workdir = make_workdir(prefix="vtword_url_")
    cookie_path = None
    if data.cookie_text.strip():
        cookie_path = workdir / "cookies.txt"
        cookie_path.write_text(data.cookie_text, encoding="utf-8")
    elif data.cookie_file:
        cookie_path = data.cookie_file

    if progress_callback:
        progress_callback("Downloading video", 20)
    video_path = download_video(data.url, workdir, cookie_file=cookie_path)
    _check_cancel(is_cancelled)
    audio_path = workdir / "audio.wav"
    if progress_callback:
        progress_callback("Extracting audio", 45)
    extract_audio(video_path, audio_path)
    _check_cancel(is_cancelled)
    if progress_callback:
        progress_callback("Starting ASR", 60)
    return transcribe_audio(
        audio_path,
        model_size=cfg.model_size,
        compute_type=cfg.compute_type,
        language=cfg.language,
        progress_callback=progress_callback,
        is_cancelled=is_cancelled,
    )
