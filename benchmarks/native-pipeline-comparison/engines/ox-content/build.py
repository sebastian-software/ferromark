from pathlib import Path


def build(ctx):
    return ctx.rust(Path(__file__).with_name("Cargo.toml"), "ox-content-driver")
