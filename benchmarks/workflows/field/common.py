"""Complete-document admission and cross-runtime process measurements."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

FIELD = Path(__file__).resolve().parent
sys.path.append(str(FIELD.parent))
from support import ROOT, Worker, read_json, sha, write_json
from workload import CanonicalHTML, workload_review, _review_tasks, _review_alignment

ENGINES = ["ferromark", "pulldown", "comrak", "md4c", "cmark", "cmark-gfm", "goldmark", "satteri",
           "rushdown", "markdig", "markdown-rs", "ox-content", "ferromark-bun", "bun"]
PROTOCOL = {"schema": 1, "rounds": 3, "windows": 80, "window_ms": 63, "warmup_ms": 3000,
            "memory": "macOS wait4 ru_maxrss; whole warmed worker lifetime, bytes"}
LABELS = {"ferromark": "Ferromark", "pulldown": "pulldown-cmark", "comrak": "Comrak", "md4c": "md4c",
          "cmark": "cmark", "cmark-gfm": "cmark-gfm", "goldmark": "Goldmark", "satteri": "Sätteri",
          "rushdown": "Rushdown", "markdig": "Markdig", "markdown-rs": "markdown-rs", "ox-content": "Ox Content",
          "ferromark-bun": "Ferromark (Bun support)", "bun": "Bun (native)"}


class FieldWorker(Worker):
    def __init__(self, config, corpus, group="documentation", lifetime="stream"):
        self.engine = config["engine"]
        env = os.environ.copy()
        for key in list(env):
            if key.startswith(("DOTNET_", "COMPlus_")) or key in ("GOGC", "GOMEMLIMIT", "GOMAXPROCS", "LD_PRELOAD", "DYLD_INSERT_LIBRARIES"):
                env.pop(key)
        env.update(config["environment"])
        env["LC_ALL"] = "C"
        self.process = subprocess.Popen([*config["command"], config["engine"], str(corpus), group, lifetime],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, env=env)

    def ask(self, action, **fields):
        self.process.stdin.write(json.dumps({"action": action, **fields}) + "\n")
        self.process.stdin.flush()
        line = self.process.stdout.readline()
        if not line:
            # Do not poll/reap here: close() needs wait4's per-child accounting.
            raise RuntimeError(f"No response from {self.engine} field worker")
        return json.loads(line)

    def close(self):
        self.process.stdin.close()
        _, status, usage = os.wait4(self.process.pid, 0)
        self.process.returncode = os.waitstatus_to_exitcode(status)
        self.process.stdout.close()
        if self.process.returncode:
            raise RuntimeError(f"{self.engine} field worker failed: {self.process.returncode}")
        return {"peak_rss_bytes": usage.ru_maxrss, "user_cpu_seconds": usage.ru_utime, "system_cpu_seconds": usage.ru_stime}


def output_units(outputs, unit):
    if unit not in ("utf8", "utf16"):
        raise ValueError("Unknown native output unit")
    return sum(len(d["html"].encode("utf-8" if unit == "utf8" else "utf-16-le")) // (1 if unit == "utf8" else 2) for d in outputs)


def without_generated_heading_ids(html):
    tokens = CanonicalHTML(html).tokens
    ids = set()
    result = []
    for token in tokens:
        if token[0] == "start" and token[1] in ("h1", "h2", "h3", "h4", "h5", "h6"):
            attrs = dict(token[2])
            if "id" in attrs:
                value = attrs.pop("id")
                if not value or value in ids:
                    raise ValueError("Empty or duplicate generated heading ID")
                ids.add(value)
            token = ("start", token[1], sorted(attrs.items()))
        result.append(token)
    return _review_tasks(_review_alignment(result))


def admission(outputs, corpus):
    if set(outputs) != set(ENGINES):
        raise ValueError("Missing engine in the declared field")
    expected = [d["id"] for d in corpus["documentation"]]
    result = {}
    for engine in ENGINES:
        stream, retain = outputs[engine]["stream"], outputs[engine]["retain"]
        identity = "ferromark" if engine == "ferromark-bun" else engine
        if (stream["engine"] != identity or retain["engine"] != identity or
                stream["group"] != "documentation" or retain["group"] != "documentation" or
                stream["retain"] is not False or retain["retain"] is not True or
                stream["options"] != retain["options"]):
            raise ValueError("Engine identity, options or lifetime changed")
        if stream["outputs"] != retain["outputs"] or [d["id"] for d in stream["outputs"]] != expected:
            raise ValueError("Incomplete collection or output lifetime changed content")
        reference = "ferromark-bun" if engine in ("bun", "ferromark-bun") else "ferromark"
        rows = []
        for a, b in zip(outputs[reference]["stream"]["outputs"], stream["outputs"]):
            pair = {"ferromark": a["html"], "candidate": b["html"]}
            review = workload_review("gfm_overlap/" + a["id"], pair, parsers=set(pair))
            if not review["comparable"] and engine == "ox-content":
                if without_generated_heading_ids(a["html"]) == without_generated_heading_ids(b["html"]):
                    review = {"comparable": True, "html_equivalent": False,
                              "accepted_differences": ["additional-generated-heading-ids", "existing-task-and-table-presentation"]}
            rows.append({"id": a["id"], **review,
                         "html_sha256": hashlib.sha256(b["html"].encode()).hexdigest()})
        result[engine] = {"admitted": all(r["comparable"] for r in rows), "documents": rows,
                          "complete_documents": sum(r["comparable"] for r in rows), "total_documents": len(expected)}
    return result
