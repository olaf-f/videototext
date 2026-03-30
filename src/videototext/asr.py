from __future__ import annotations

from pathlib import Path
from threading import Lock
from typing import Callable, Optional

from faster_whisper import WhisperModel

from videototext.models import Segment, TranscriptionResult

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
    progress_callback: Optional[Callable[[str, int], None]] = None,
    is_cancelled: Optional[Callable[[], bool]] = None,
) -> TranscriptionResult:
    model = _get_model(model_size=model_size, compute_type=compute_type)
    segments, _ = model.transcribe(str(audio_path), language=language, vad_filter=True)
    chunk_texts: list[str] = []
    out_segments: list[Segment] = []

    if progress_callback:
        progress_callback("ASR running", 70)

    for idx, seg in enumerate(segments):
        if is_cancelled and is_cancelled():
            raise RuntimeError("Task cancelled")
        text = seg.text.strip()
        if text:
            chunk_texts.append(text)
            out_segments.append(Segment(start=float(seg.start), end=float(seg.end), text=text))
        if progress_callback and idx % 10 == 0:
            progress_callback("ASR running", 70 + min(25, idx))

    return TranscriptionResult(text="\n".join(chunk_texts), segments=out_segments)
