#!/usr/bin/env python3
import argparse
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CLIENT_DIR = ROOT / "clients" / "firefox"
EXCLUDE_NAMES = {"LICENSE", ".source-baseline", "release-state.json"}
EXCLUDE_SUFFIXES = {".md", ".py", ".pyc"}
EXCLUDE_PARTS = {"scripts", "tests", "__pycache__", ".git"}

def should_include(path: Path) -> bool:
    rel = path.relative_to(CLIENT_DIR)
    if any(part in EXCLUDE_PARTS for part in rel.parts):
        return False
    if path.name in EXCLUDE_NAMES:
        return False
    if path.suffix.lower() in EXCLUDE_SUFFIXES:
        return False
    return path.is_file()

def main() -> None:
    parser = argparse.ArgumentParser(description="Package the GoreeCloud Download Manager Firefox client.")
    parser.add_argument("--output-dir", default="dist")
    args = parser.parse_args()
    manifest = json.loads((CLIENT_DIR / "manifest.json").read_text(encoding="utf-8"))
    version = manifest["version"]
    output_dir = ROOT / args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)
    output = output_dir / f"goreecloud-download-manager-{version}.xpi"
    files = sorted(path for path in CLIENT_DIR.rglob("*") if should_include(path))
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in files:
            rel = path.relative_to(CLIENT_DIR).as_posix()
            info = zipfile.ZipInfo(rel, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            archive.writestr(info, path.read_bytes())
    with zipfile.ZipFile(output) as archive:
        bad = archive.testzip()
        if bad:
            raise SystemExit(f"Package integrity failure: {bad}")
        names = archive.namelist()
        if "manifest.json" not in names:
            raise SystemExit("Package does not contain manifest.json at archive root")
        source_only = [
            name for name in names
            if any(part in {"scripts", "tests", "__pycache__", ".git"} for part in Path(name).parts)
            or Path(name).suffix.lower() in EXCLUDE_SUFFIXES
        ]
        if source_only:
            raise SystemExit(f"Package contains source-only files: {', '.join(source_only)}")
    print(output.relative_to(ROOT))

if __name__ == "__main__":
    main()
