"""Create a source-only ZIP with a SHA-256 inventory. Never package live state."""
from pathlib import Path, PurePosixPath
import argparse
import hashlib
import os
import stat
import zipfile

root = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output", type=Path, required=True)
args = parser.parse_args()
output = args.output.resolve()
excluded_dirs = {".git", ".aws", ".codex", ".agents", "target", "node_modules", "dist", "__pycache__", ".venv", "data", "backups", ".pytest_cache"}
excluded_suffixes = {".zip", ".pyc", ".tsbuildinfo", ".log", ".db", ".sqlite", ".sqlite3", ".pem", ".key", ".p12"}
files = []
for directory, dirs, names in os.walk(root):
    dirs[:] = sorted(d for d in dirs if d not in excluded_dirs and not (Path(directory)/d).is_symlink())
    for name in sorted(names):
        p = Path(directory)/name
        if p.is_symlink():
            raise RuntimeError(f"Refuse source symlink: {p.relative_to(root)}")
        if name in {"MANIFEST.sha256", ".DS_Store"} or p.resolve() == output:
            continue
        if (name.startswith(".env") and name != ".env.example") or p.suffix in excluded_suffixes or name.endswith((".db-shm", ".db-wal")):
            continue
        files.append(p)
manifest = "".join(f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.relative_to(root).as_posix()}\n" for p in sorted(files))
manifest_file = root/"MANIFEST.sha256"
manifest_file.write_text(manifest)
output.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    for p in sorted(files+[manifest_file]):
        name = "sami/"+p.relative_to(root).as_posix()
        info = zipfile.ZipInfo(name, (2026, 10, 2, 0, 0, 0))
        mode = 0o755 if p.suffix == ".sh" else 0o644
        info.external_attr = (stat.S_IFREG|mode)<<16
        info.compress_type = zipfile.ZIP_DEFLATED
        archive.writestr(info, p.read_bytes())
with zipfile.ZipFile(output) as archive:
    assert archive.testzip() is None, "Corrupt ZIP"
    for name in archive.namelist():
        path = PurePosixPath(name)
        assert not path.is_absolute() and ".." not in path.parts and path.parts[0] == "sami"
    for line in manifest.splitlines():
        digest, name = line.split("  ",1)
        assert hashlib.sha256(archive.read("sami/"+name)).hexdigest() == digest
print(f"ZIP verified: {len(files)+1} files, {output.stat().st_size} bytes, SHA-256 {hashlib.sha256(output.read_bytes()).hexdigest()}")
