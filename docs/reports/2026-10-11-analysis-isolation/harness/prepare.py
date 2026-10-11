"""Prepare the retained paired harness without modifying either source checkout."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import tomllib

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--repository", type=Path, required=True)
parser.add_argument("--before", default="2df7d368")
parser.add_argument("--destination", type=Path, required=True)
args = parser.parse_args()
repository = args.repository.resolve()
destination = args.destination.resolve()
if destination.exists():
    parser.error("--destination must not exist")
destination.mkdir(parents=True)
baseline = destination / "baseline"
baseline.mkdir()
with tempfile.TemporaryFile() as archive:
    subprocess.run(["git", "-C", str(repository), "archive", args.before], stdout=archive, check=True)
    archive.seek(0)
    with tarfile.open(fileobj=archive) as files:
        files.extractall(baseline, filter="data")
manifest = baseline / "Cargo.toml"
contents = manifest.read_text()
version = tomllib.loads(contents)["package"]["version"]
contents = contents.replace(f'version = "{version}"', f'version = "{version}-performance.baseline"', 1)
manifest.write_text(contents)
retained = Path(__file__).resolve().parent
shutil.copytree(retained / "benches", destination / "benches")
shutil.copytree(retained / "fixtures", destination / "fixtures")
shutil.copyfile(retained / "Cargo.lock", destination / "Cargo.lock")
(destination / "Cargo.toml").write_text(
    '[package]\nname = "ferromark-analysis-performance"\n'
    'version = "0.0.0"\nedition = "2024"\npublish = false\n\n'
    '[dependencies]\n'
    f'before = {{ package = "ferromark", path = {json.dumps(str(baseline))}, features = ["jsx"] }}\n'
    f'after = {{ package = "ferromark", path = {json.dumps(str(repository))}, features = ["jsx"] }}\n'
    'criterion = "=0.8.2"\n\n[[bench]]\nname = "ordinary_paths"\nharness = false\n\n'
    '[profile.bench]\nlto = "fat"\ncodegen-units = 1\n'
)
print(destination)

