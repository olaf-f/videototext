from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class UrlInput:
    url: str
    cookie_text: str = ""
    cookie_file: Optional[Path] = None


@dataclass
class AppConfig:
    model_size: str = "base"
    compute_type: str = "int8"
    language: Optional[str] = None


@dataclass
class Segment:
    start: float
    end: float
    text: str


@dataclass
class TranscriptionResult:
    text: str
    segments: list[Segment]
