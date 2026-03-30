from __future__ import annotations

from pathlib import Path
from threading import Lock
from typing import Optional

from faster_whisper import WhisperModel

_MODEL = None
_MODEL_LOCK = Lock()


def _get_model(model_size: str, compute_type: str) -> WhisperModel:
    global _MODEL
    with _MODEL_LOCK:
        if _MODEL is None:
            _MODEL = WhisperModel(model_size, compute_type=compute_type)
    return _MODEL


def transcribe_audio(
    audio_path: Path,
    model_size: str = "base",
    compute_type: str = "int8",
    language: Optional[str] = None,
) -> str:
    model = _get_model(model_size=model_size, compute_type=compute_type)
    segments, _ = model.transcribe(str(audio_path), language=language, vad_filter=True)
    chunks = [seg.text.strip() for seg in segments if seg.text.strip()]
    return "\n".join(chunks)
