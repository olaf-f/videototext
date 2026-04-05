#!/usr/bin/env bash
set -euo pipefail

PYTHON_BIN="${1:-python3}"
"${PYTHON_BIN}" -m pip install --upgrade pip
"${PYTHON_BIN}" -m pip install -r requirements.txt
"${PYTHON_BIN}" -m pip install pyinstaller
"${PYTHON_BIN}" -m PyInstaller --noconfirm --windowed --name VideoToText --paths src main.py
