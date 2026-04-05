from __future__ import annotations

import tempfile
from datetime import datetime
from pathlib import Path


def make_workdir(prefix: str = "vtword_") -> Path:
    return Path(tempfile.mkdtemp(prefix=prefix))


def timestamp() -> str:
    return datetime.now().strftime("%Y%m%d_%H%M%S")
