from __future__ import annotations

from pathlib import Path
from typing import Optional

from yt_dlp import YoutubeDL


def download_video(url: str, output_dir: Path, cookie_file: Optional[Path] = None) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    outtmpl = str(output_dir / "%(title).120s.%(ext)s")
    opts = {
        "outtmpl": outtmpl,
        "noplaylist": True,
        "quiet": True,
    }
    if cookie_file:
        opts["cookiefile"] = str(cookie_file)

    with YoutubeDL(opts) as ydl:
        info = ydl.extract_info(url, download=True)
        path = ydl.prepare_filename(info)

    return Path(path)
