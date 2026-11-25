import os
import shutil
from pathlib import Path

root = Path(__file__).resolve().parent.parent
turbo_src = root / "src-turbo-bar"
turbo_dest = root / "src" / "turbo"

if turbo_src.is_dir():
    print(f"[inject_turbo] Injecting {turbo_src} -> {turbo_dest}")
    shutil.copytree(turbo_src, turbo_dest, dirs_exist_ok=True)
    print("[inject_turbo] Done.")
else:
    print(f"[inject_turbo] Warning: {turbo_src} not found.", flush=True)