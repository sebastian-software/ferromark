"""Reconstruct frozen sources and optionally replay the exact-output checks."""
from pathlib import Path
import argparse
import gzip
import hashlib
import json
import shutil
import subprocess
import sys

archive = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--work", type=Path, required=True, help="New scratch directory")
parser.add_argument("--repo", type=Path, default=archive.parents[2])
parser.add_argument("--verify", action="store_true")
args = parser.parse_args()

for line in (archive / "SHA256SUMS").read_text().splitlines():
    expected, name = line.split("  ", 1)
    actual = hashlib.sha256((archive / name).read_bytes()).hexdigest()
    if actual != expected:
        raise SystemExit(f"Archive checksum mismatch: {name}")

shutil.copytree(archive, args.work)
for compressed in args.work.rglob("*.json.gz"):
    compressed.with_suffix("").write_bytes(gzip.decompress(compressed.read_bytes()))
metadata = json.loads((args.work / "metadata.json").read_text())
source_hashes = json.loads((args.work / "baseline/source.json").read_text())
for name, expected in source_hashes.items():
    source = subprocess.check_output(
        ["git", "show", f"{metadata['baseline']}:{name}"], cwd=args.repo
    )
    if hashlib.sha256(source).hexdigest() != expected:
        raise SystemExit(f"Frozen source mismatch: {name}")
    for tree in ["baseline", "source"]:
        target = args.work / tree / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(source)
for tree in ["baseline", "source"]:
    shutil.copyfile(args.work / "source-Cargo.toml", args.work / tree / "Cargo.toml")
production_hash = hashlib.sha256((args.work / "production-inline.rs").read_bytes()).hexdigest()
if production_hash != metadata["production_source_hashes"]["src/inline/mod.rs"]:
    raise SystemExit("Production source identity mismatch")
print(f"Prepared frozen baseline {metadata['baseline']} in {args.work}", flush=True)

if args.verify:
    # Cargo build uses --offline: dependency download is an explicit preparatory step.
    subprocess.run(
        ["cargo", "fetch", "--locked", "--manifest-path", str(args.work / "driver/Cargo.toml")],
        cwd=args.work,
        check=True,
    )
    sys.path.insert(0, str(args.work))
    from experiment import build, verify

    original = json.loads((args.work / "baseline-verification.json").read_text())
    if not build("baseline") or not verify("baseline"):
        raise SystemExit("Baseline verification failed")
    if json.loads((args.work / "baseline-verification.json").read_text()) != original:
        raise SystemExit("Rebuilt baseline differs from the archived output oracle")
    if not build("production") or not verify("production"):
        raise SystemExit("Production verification failed")
    for script, record in [
        ("verify-extra.py", "extra-verification.json"),
        ("verify-extended.py", "extended-verification.json"),
    ]:
        subprocess.run([sys.executable, str(args.work / script), "production"], check=True)
        result = json.loads((args.work / "production" / record).read_text())
        if result["failures"]:
            raise SystemExit(f"Production event verification failed: {record}")
    print("All frozen production output checks passed", flush=True)
