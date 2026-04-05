from __future__ import annotations

from pathlib import Path

from docx import Document

from videototext.models import Segment


def export_txt(text: str, target: Path) -> Path:
    target.write_text(text, encoding="utf-8")
    return target


def export_docx(text: str, target: Path) -> Path:
    doc = Document()
    for line in text.splitlines() or [""]:
        doc.add_paragraph(line)
    doc.save(str(target))
    return target


def _srt_ts(seconds: float) -> str:
    millis = int(max(0.0, seconds) * 1000)
    hour = millis // 3600000
    minute = (millis % 3600000) // 60000
    sec = (millis % 60000) // 1000
    ms = millis % 1000
    return f"{hour:02d}:{minute:02d}:{sec:02d},{ms:03d}"


def export_srt(segments: list[Segment], target: Path) -> Path:
    lines: list[str] = []
    for idx, seg in enumerate(segments, start=1):
        lines.append(str(idx))
        lines.append(f"{_srt_ts(seg.start)} --> {_srt_ts(seg.end)}")
        lines.append(seg.text.strip())
        lines.append("")
    target.write_text("\n".join(lines), encoding="utf-8")
    return target
