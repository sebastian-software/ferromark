"""Extract archived trees next to this script: extract.py <name>=<archive.tar>..."""
import sys
import tarfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
for spec in sys.argv[1:]:
    name, archive = spec.split("=", 1)
    target = HERE / name
    target.mkdir(exist_ok=True)
    with tarfile.open(HERE / archive) as tar:
        tar.extractall(target, filter="tar")
    print("extracted", name)
