from __future__ import annotations

from pathlib import Path

from docx import Document


def export_txt(text: str, target: Path) -> Path:
    target.write_text(text, encoding="utf-8")
    return target


def export_docx(text: str, target: Path) -> Path:
    doc = Document()
    for line in text.splitlines() or [""]:
        doc.add_paragraph(line)
    doc.save(str(target))
    return target
