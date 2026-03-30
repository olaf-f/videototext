from __future__ import annotations

from pathlib import Path

from videototext.asr import transcribe_audio
from videototext.downloader import download_video
from videototext.media import extract_audio
from videototext.models import AppConfig, UrlInput
from videototext.ocr import ocr_image
from videototext.utils import make_workdir


def run_ocr(image_path: Path) -> str:
    return ocr_image(image_path)


def run_transcribe_file(video_path: Path, cfg: AppConfig) -> str:
    workdir = make_workdir()
    audio_path = workdir / "audio.wav"
    extract_audio(video_path, audio_path)
    return transcribe_audio(
        audio_path,
        model_size=cfg.model_size,
        compute_type=cfg.compute_type,
        language=cfg.language,
    )


def run_transcribe_url(data: UrlInput, cfg: AppConfig) -> str:
    workdir = make_workdir(prefix="vtword_url_")
    cookie_path = None
    if data.cookie_text.strip():
        cookie_path = workdir / "cookies.txt"
        cookie_path.write_text(data.cookie_text, encoding="utf-8")
    elif data.cookie_file:
        cookie_path = data.cookie_file

    video_path = download_video(data.url, workdir, cookie_file=cookie_path)
    audio_path = workdir / "audio.wav"
    extract_audio(video_path, audio_path)
    return transcribe_audio(
        audio_path,
        model_size=cfg.model_size,
        compute_type=cfg.compute_type,
        language=cfg.language,
    )
