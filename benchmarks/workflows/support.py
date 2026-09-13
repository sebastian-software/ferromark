"""Frozen workflow protocol and independently reproducible validation helpers."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.append(str(ROOT / "benchmarks/bun-comparison"))
from workload import workload_review

LEGACY_VARIANTS = ["preview-fresh", "preview-reuse", "guide-metadata",
            "ferromark-stream", "pulldown-stream", "comrak-stream",
            "ferromark-retain", "pulldown-retain", "comrak-retain"]
VARIANTS = ["preview-fresh", "preview-reuse", "preview-pulldown", "preview-comrak",
            "guide-metadata", "guide-pulldown", "guide-comrak", *LEGACY_VARIANTS[3:]]
PROTOCOL = {"rounds": 3, "windows": 80, "window_ms": 63, "warmup_ms": 3000,
            "memory_observations": 10, "schema": 2}


def variants_for(protocol):
    if protocol == PROTOCOL:
        return VARIANTS
    if protocol == {**PROTOCOL, "schema": 1}:
        return LEGACY_VARIANTS
    raise ValueError("Publication protocol changed")


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_json(path):
    path = Path(path)
    return json.loads(gzip.decompress(path.read_bytes()) if path.suffix == ".gz" else path.read_bytes())


def write_json(path, data):
    path = Path(path)
    raw = (json.dumps(data, ensure_ascii=False, indent=2) + "\n").encode()
    path.write_bytes(gzip.compress(raw, mtime=0) if path.suffix == ".gz" else raw)


def source_hashes():
    paths = [ROOT / name for name in ("Cargo.toml", "Cargo.lock", ".cargo/config.toml", "rust-toolchain.toml")]
    paths += list((ROOT / "src").rglob("*.rs"))
    paths += list((ROOT / "crates/ferro-byte-search").rglob("*.rs"))
    paths += [ROOT / "crates/ferro-byte-search/Cargo.toml", ROOT / "benchmarks/bun-comparison/workload.py"]
    paths += [p for p in HERE.rglob("*") if p.is_file() and p.suffix in (".rs", ".py", ".toml", ".lock", ".json", ".go", ".cs", ".c", ".h", ".sum", ".csproj") and "target" not in p.parts]
    return {str(p.relative_to(ROOT)): sha(p) for p in sorted(set(paths))}


def command(args):
    env = os.environ.copy()
    env["LC_ALL"] = "C"
    return subprocess.check_output(args, cwd=ROOT, env=env, text=True, stderr=subprocess.STDOUT).strip()


def observation():
    result = {}
    for key, args in {
        "power": ["pmset", "-g", "batt"],
        "load": ["sysctl", "-n", "vm.loadavg"],
        "process_cpu": ["ps", "-Ao", "pcpu,comm"],
        "thermal": ["pmset", "-g", "therm"],
    }.items():
        try:
            value = command(args)
            if key == "process_cpu":
                value = "\n".join(sorted(value.splitlines()[1:], key=lambda s: float(s.split()[0]), reverse=True)[:12])
            result[key] = value
        except (OSError, subprocess.CalledProcessError) as error:
            result[key] = str(error)
    return result


class Worker:
    def __init__(self, binary, corpus, variant):
        self.process = subprocess.Popen([str(binary), str(corpus), variant], stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, text=True, cwd=ROOT)

    def ask(self, action, **fields):
        self.process.stdin.write(json.dumps({"action": action, **fields}) + "\n")
        self.process.stdin.flush()
        line = self.process.stdout.readline()
        if not line:
            raise RuntimeError(f"worker exited with {self.process.poll()}")
        return json.loads(line)

    def close(self):
        self.process.stdin.close()
        code = self.process.wait(timeout=15)
        self.process.stdout.close()
        if code:
            raise RuntimeError(f"worker failed: {code}")


def group(variant):
    return "previews" if variant.startswith("preview-") else "guides" if variant.startswith("guide-") else "documentation"


def review(verification, corpus, protocol=PROTOCOL):
    variants = variants_for(protocol)
    if set(verification) != set(variants):
        raise ValueError("Missing workflow variants")
    for variant, row in verification.items():
        if row["variant"] != variant or [d["id"] for d in row["outputs"]] != [d["id"] for d in corpus[group(variant)]]:
            raise ValueError("Verification does not cover the complete corpus")
    if verification["preview-fresh"]["outputs"] != verification["preview-reuse"]["outputs"]:
        raise ValueError("Reused preview renderer changed output")
    previews = {d["id"]: d["html"] for d in verification["preview-fresh"]["outputs"]}
    if '<div onclick=' in previews["raw-html"] or 'href="javascript:' in previews["unsafe-link"]:
        raise ValueError("Untrusted preview safety boundary was bypassed")
    if '&lt;div' not in previews["raw-html"] or 'href="https://example.org/docs"' not in previews["unsafe-link"]:
        raise ValueError("Preview lost expected escaped text or safe links")
    for row in verification["guide-metadata"]["outputs"]:
        metadata = row["metadata"]
        if not metadata["front_matter"] or not metadata["headings"] or not all(h["id"] for h in metadata["headings"]):
            raise ValueError("Guide pipeline did not return metadata")
    results = []
    for index, document in enumerate(corpus["documentation"]):
        outputs = {}
        for parser in ("ferromark", "pulldown", "comrak"):
            stream = verification[f"{parser}-stream"]["outputs"][index]
            if stream != verification[f"{parser}-retain"]["outputs"][index]:
                raise ValueError("Output lifetime changed rendered content")
            outputs[parser] = stream["html"]
        result = workload_review("gfm_overlap/" + document["id"], outputs, parsers=set(outputs))
        results.append({"id": document["id"], **result,
                        "html_sha256": {p: hashlib.sha256(h.encode()).hexdigest() for p, h in outputs.items()}})
    if protocol["schema"] == 2:
        for name, reference, others in (
            ("previews", "preview-fresh", ("preview-pulldown", "preview-comrak")),
            ("guides", "guide-metadata", ("guide-pulldown", "guide-comrak")),
        ):
            for index, document in enumerate(corpus[name]):
                ref = verification[reference]["outputs"][index]
                outputs = {"ferromark": ref["html"]}
                for variant in others:
                    row = verification[variant]["outputs"][index]
                    if row["metadata"] != ref["metadata"]:
                        raise ValueError(f"Complete metadata differs: {variant}/{document['id']}")
                    outputs[variant.split("-")[1]] = row["html"]
                result = workload_review("gfm_overlap/" + document["id"], outputs, parsers=set(outputs))
                results.append({"id": name + "/" + document["id"], **result,
                                "metadata_equivalent": True,
                                "html_sha256": {p: hashlib.sha256(h.encode()).hexdigest() for p, h in outputs.items()}})
    return results
