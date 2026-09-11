import json
import platform
from pathlib import Path
from common import capture


def build(ctx):
    here = Path(__file__).resolve().parent
    version = capture(["dotnet", "--version"], cwd=here, env=ctx.env)
    if version != "10.0.401":
        raise ValueError(f"Expected .NET SDK 10.0.401, got {version}")
    ctx.details["dotnet"] = capture(["dotnet", "--info"], cwd=here, env=ctx.env)
    arch = {"arm64": "arm64", "aarch64": "arm64", "x86_64": "x64"}[platform.machine()]
    os = {"Darwin": "osx", "Linux": "linux"}[platform.system()]
    rid = f"{os}-{arch}"
    out = ctx.work / "markdig"
    # Publish the runtime with the worker; hash its complete runnable closure.
    ctx.run(["dotnet", "publish", here / "MarkdigDriver.csproj", "-c", "Release", "-r", rid,
        "--self-contained", "true", "-p:RestoreLockedMode=true", "-o", out,
        f"-p:BaseIntermediateOutputPath={ctx.work / 'obj'}/"], cwd=here)
    ctx.locks["engines/markdig/packages.lock.json"] = (here / "packages.lock.json").read_text()
    ctx.details["runtime_config"] = json.loads((out / "MarkdigDriver.runtimeconfig.json").read_text())
    return [str(out / "MarkdigDriver")]
