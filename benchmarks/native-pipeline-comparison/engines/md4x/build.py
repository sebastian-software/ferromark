from pathlib import Path
from common import capture


def build(ctx):
    here = Path(__file__).resolve().parent
    version = capture(["zig", "version"])
    if version != "0.16.0":
        raise ValueError(f"Expected Zig 0.16.0, got {version}")
    ctx.details["zig"] = version
    output = ctx.work / "md4x-driver"
    ctx.run(["zig", "fmt", "--check", here / "main.zig", here / "features.zig"])
    ctx.run(["zig", "build-exe", "-O", "ReleaseFast", "-mcpu=baseline", "-lc",
        "--dep", "md4x", f"-Mroot={here / 'main.zig'}", "--dep", "abi", "--dep", "build_config",
        f"-Mmd4x={ctx.source / 'src/lib.zig'}", f"-Mabi={ctx.source / 'src/abi.zig'}",
        f"-Mbuild_config={here / 'features.zig'}", f"-femit-bin={output}",
        "--cache-dir", ctx.work / ".zig-cache", "--global-cache-dir", ctx.work / ".zig-cache/global"])
    return [str(output)]
