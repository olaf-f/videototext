from __future__ import annotations

from pathlib import Path

from rapidocr_onnxruntime import RapidOCR

_OCR = None


def _get_ocr() -> RapidOCR:
    global _OCR
    if _OCR is None:
        _OCR = RapidOCR()
    return _OCR


def ocr_image(image_path: Path) -> str:
    engine = _get_ocr()
    result, _ = engine(str(image_path))
    if not result:
        return ""
    return "\n".join([line[1] for line in result if len(line) > 1 and line[1]])
