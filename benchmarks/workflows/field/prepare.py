#!/usr/bin/env python3
"""Build the published engine field against the frozen practical corpus."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

FIELD = Path(__file__).resolve().parent
sys.path.insert(0, str(FIELD.parent))
from support import ROOT, command, observation, sha, source_hashes, write_json

NATIVE = ROOT / "benchmarks/native-pipeline-comparison"
CMARK = ROOT / "benchmarks/cmark-comparison"
BUN = ROOT / "benchmarks/bun-comparison"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build", type=Path)
    parser.add_argument("--bun", type=Path, required=True, help="Dedicated workspace prepared by bun-comparison/prepare.py")
    parser.add_argument("--md4c", type=Path, required=True)
    parser.add_argument("--cmark", type=Path, required=True)
    parser.add_argument("--cmark-gfm", type=Path, required=True)
    parser.add_argument("--dotnet", type=Path, required=True)
    parser.add_argument("--resume", action="store_true", help="Resume a failed build in the same directory; all steps and hashes are rechecked")
    args = parser.parse_args()
    work = args.build.resolve()
    work.mkdir(parents=True, exist_ok=args.resume)
    env = os.environ.copy()
    for key in list(env):
        if key.startswith(("CARGO_PROFILE_", "DOTNET_", "COMPlus_")) or key in (
            "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "LD_PRELOAD", "DYLD_INSERT_LIBRARIES"):
            env.pop(key)
    env.update(RUSTFLAGS="-C target-cpu=generic", LC_ALL="C", GOENV="off", GOTOOLCHAIN="local",
               GOWORK="off", GOFLAGS="", CGO_ENABLED="0", GOEXPERIMENT="", GOARM64="v8.0", GOAMD64="v1")
    commands, workers, locks, upstream, referenced = [], {}, {}, {}, {}
    before = source_hashes()

    def run(argv, cwd=ROOT, extra=None):
        commands.append({"argv": list(map(str, argv)), "cwd": str(cwd), "extra_environment": extra or {}})
        with (work / "build.log").open("a") as log:
            subprocess.run(list(map(str, argv)), cwd=cwd, env={**env, **(extra or {})}, stdout=log, stderr=subprocess.STDOUT, check=True)

    def read(path):
        referenced[str(path.relative_to(ROOT))] = sha(path)
        return path.read_text()

    def source(path, revision):
        path = path.resolve()
        if command(["git", "-C", str(path), "rev-parse", "HEAD"]) != revision:
            raise ValueError(f"Wrong pinned source: {path}")
        upstream[str(path)] = {"revision": revision, "diff": command(["git", "-C", str(path), "diff", "--binary", "HEAD"])}

    def rust(name, manifest, code, features=(), build_script=None, extra=None):
        dest = work / "sources" / name
        dest.mkdir(parents=True, exist_ok=True)
        text = read(manifest)
        # Preserve package/dependency versions and release profile; resolve only local paths.
        def absolute(match):
            return 'path = ' + json.dumps(str((manifest.parent / match[1]).resolve()))
        text = re.sub(r'path = "(\.[^\"]*)"', absolute, text)
        (dest / "Cargo.toml").write_text(text)
        lock = read(manifest.with_name("Cargo.lock"))
        (dest / "Cargo.lock").write_text(lock)
        locks[name] = lock
        config = tomllib.loads(text)
        target = config.get("bin", [{"path": "src/main.rs", "name": config["package"]["name"]}])[0]
        file = dest / target["path"]
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(code + "\n" + read(FIELD / "worker.rs"))
        if build_script is not None:
            (dest / "build.rs").write_text(build_script)
        argv = ["cargo", "build", "--release", "--locked", "--manifest-path", dest / "Cargo.toml", "--target-dir", work / "target"]
        if features:
            argv += ["--features", ",".join(features)]
        run(argv, extra=extra)
        output = work / name
        shutil.copy2(work / "target/release" / target["name"], output)
        return [str(output)]

    basic = rust("basic", FIELD.parent / "Cargo.toml", read(FIELD / "basic.rs"))
    for engine in ("ferromark", "pulldown", "comrak"):
        workers[engine] = {"command": basic, "engine": engine, "units": "utf8", "environment": {}}
    for engine in ("rushdown", "markdown-rs", "ox-content"):
        directory = NATIVE / "engines" / engine
        code = read(directory / "main.rs").split('include!("../worker.rs");')[0]
        binary = rust(engine, directory / "Cargo.toml", code)
        workers[engine] = {"command": binary, "engine": engine, "units": "utf8", "environment": {},
                           "adapter": json.loads(read(directory / "engine.json"))}
    workers["satteri"] = {"command": rust("satteri", NATIVE / "Cargo.toml", read(FIELD / "satteri.rs")),
                          "engine": "satteri", "units": "utf8", "environment": {}}

    source(args.md4c, "65c6c9d72cebd9a731aaa5597414ce04d9ea5de3")
    mdsource = args.md4c.resolve() / "src"
    mdscript = 'fn main() { cc::Build::new().file(' + json.dumps(str(mdsource / "md4c.c")) + ').file(' + json.dumps(str(mdsource / "md4c-html.c")) + ').file(' + json.dumps(str(mdsource / "entity.c")) + ').include(' + json.dumps(str(mdsource)) + ').opt_level(3).compile("md4c"); }\n'
    workers["md4c"] = {"command": rust("md4c", CMARK / "Cargo.toml", read(FIELD / "md4c.rs"), build_script=mdscript),
                        "engine": "md4c", "units": "utf8", "environment": {}}
    for engine, path, revision in (("cmark", args.cmark, "bb3678d7a73cb02d35c8876ecd097072636200a8"),
                                   ("cmark-gfm", args.cmark_gfm, "587a12bb54d95ac37241377e6ddc93ea0e45439b")):
        source(path, revision)
        native = work / (engine + "-native")
        run(["cmake", "-S", path.resolve(), "-B", native, "-DCMAKE_BUILD_TYPE=Release", "-DBUILD_SHARED_LIBS=OFF",
             "-DCMARK_TESTS=OFF", "-DCMARK_GFM_TESTS=OFF", "-DCMARK_STATIC=ON", "-DCMARK_SHARED=OFF"])
        run(["cmake", "--build", native, "--config", "Release", "-j", "4"])
        bridge = read(CMARK / "bridge.c")
        bridge_path = work / (engine + "-bridge.c")
        bridge_path.write_text(bridge)
        build_script = read(CMARK / "build.rs").replace('.file("bridge.c")', '.file(' + json.dumps(str(bridge_path)) + ')')
        binary = rust(engine, CMARK / "Cargo.toml", read(FIELD / "cmark.rs"),
                      features=("cmark-gfm",) if engine == "cmark-gfm" else (), build_script=build_script,
                      extra={"CMARK_SOURCE": str(path.resolve()), "CMARK_BUILD": str(native)})
        workers[engine] = {"command": binary, "engine": engine, "units": "utf8", "environment": {}}

    if command(["go", "version"]).split()[2] != "go1.27.1":
        raise ValueError("Expected Go 1.27.1")
    godir = work / "sources/goldmark"
    godir.mkdir(exist_ok=True)
    for name in ("go.mod", "go.sum"):
        (godir / name).write_text(read(NATIVE / "goldmark" / name))
        locks[name] = (godir / name).read_text()
    (godir / "main.go").write_text(read(FIELD / "goldmark.go"))
    run(["go", "mod", "verify"], cwd=godir)
    run(["go", "build", "-mod=readonly", "-pgo=off", "-trimpath", "-o", work / "goldmark", "."], cwd=godir)
    workers["goldmark"] = {"command": [str(work / "goldmark")], "engine": "goldmark", "units": "utf8", "environment": {}}

    dotnet = args.dotnet.resolve()
    csdir = work / "sources/markdig"
    csdir.mkdir(exist_ok=True)
    for name in ("MarkdigDriver.csproj", "packages.lock.json", "global.json"):
        (csdir / name).write_text(read(NATIVE / "engines/markdig" / name))
    locks["packages.lock.json"] = (csdir / "packages.lock.json").read_text()
    (csdir / "Program.cs").write_text(read(FIELD / "Markdig.cs"))
    if subprocess.check_output([str(dotnet), "--version"], cwd=csdir, text=True).strip() != "10.0.401":
        raise ValueError("Expected .NET SDK 10.0.401")
    run([dotnet, "publish", "MarkdigDriver.csproj", "-c", "Release", "-r", "osx-arm64", "--self-contained", "true",
         "-p:RestoreLockedMode=true", "-o", work / "markdig-runtime"], cwd=csdir)
    workers["markdig"] = {"command": [str(work / "markdig-runtime/MarkdigDriver")], "engine": "markdig", "units": "utf16",
                          "environment": {"DOTNET_TieredCompilation": "1", "DOTNET_TieredPGO": "1", "DOTNET_gcServer": "0", "DOTNET_gcConcurrent": "1"}}

    # Keep Bun's native support and shared mimalloc as its own environment.
    bun = args.bun.resolve()
    source(bun, "76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1")
    driver = bun / "ferromark-comparison"
    manifest = (driver / "Cargo.toml").read_text()
    manifest = re.sub(r'ferromark = \{ path = "[^"]+" \}', 'ferromark = { path = ' + json.dumps(str(ROOT)) + ' }', manifest)
    if 'name = "ferromark-workflow-field"' not in manifest:
        manifest += '\n[[bin]]\nname = "ferromark-workflow-field"\npath = "field.rs"\n'
    (driver / "Cargo.toml").write_text(manifest)
    prefix = read(BUN / "driver.rs").split("fn corpora()", 1)[0]
    adapter = '''mod adapter {
        pub struct Renderer(super::Renderers, usize);
        impl Renderer {
            pub fn new(flags: u32) -> Self { Self(super::Renderers::new(flags), if std::env::args().nth(1).unwrap() == "bun" { 1 } else { 0 }) }
            pub fn render(&self, input: &str) -> Vec<u8> { self.0.render(self.1, input) }
            pub fn options(&self) -> String { format!("Bun-native shared mimalloc environment; trusted tables/strikethrough/tasks; all other BOOL_FIELD_SETTERS false; Ferromark: {:?}", self.0.ferro) }
        }
    }'''
    (driver / "field.rs").write_text(prefix + adapter + read(FIELD / "worker.rs"))
    # The existing preparation records the native support archive and codegen directory.
    build_rs = (driver / "build.rs").read_text()
    native_match = re.search(r'link-search=native=.*?", "([^"]+)"', build_rs)
    if not native_match:
        raise ValueError("Unrecognized prepared Bun native support path")
    native = Path(native_match[1])
    codegen = native.parent / "codegen"
    run(["cargo", "+nightly-2026-07-20", "build", "--release", "--locked", "-p", "ferromark-bun-comparison", "--bin", "ferromark-workflow-field"],
        cwd=bun, extra={"BUN_CODEGEN_DIR": str(codegen)})
    shutil.copy2(bun / "target/release/ferromark-workflow-field", work / "bun")
    locks["bun-Cargo.lock"] = (bun / "Cargo.lock").read_text()
    upstream["bun_native"] = {"build_rs": build_rs, "preparation": json.loads((driver / "build-info.json").read_text()),
                               "archives": {p.name: sha(p) for p in native.glob("*.a")}}
    for key, engine in (("bun", "bun"), ("ferromark-bun", "ferromark")):
        workers[key] = {"command": [str(work / "bun")], "engine": engine, "units": "utf8", "environment": {}}
    if before != source_hashes():
        raise ValueError("Source changed while building the field")
    binaries = {str(p.relative_to(work)): sha(p) for p in work.iterdir() if p.is_file() and p.name != "build.log"}
    binaries.update({str(p.relative_to(work)): sha(p) for p in (work / "markdig-runtime").iterdir() if p.is_file()})
    write_json(work / "build.json", {"source_revision": command(["git", "rev-parse", "HEAD"]), "source_status": command(["git", "status", "--porcelain"]),
        "source_sha256": before, "referenced_sha256": referenced, "workers": workers, "commands": commands, "locks": locks,
        "upstream": upstream, "binaries": binaries, "environment": {key: env[key] for key in ("RUSTFLAGS", "GOENV", "GOTOOLCHAIN", "GOWORK", "GOFLAGS", "CGO_ENABLED", "GOEXPERIMENT", "GOARM64", "GOAMD64")}, "rustc": command(["rustc", "-Vv"]),
        "cpu": command(["sysctl", "-n", "machdep.cpu.brand_string"]), "os": command(["sw_vers"]), "host_observation": observation()})
    print(f"Built {len(workers)} field workers in {work}")


if __name__ == "__main__":
    main()
