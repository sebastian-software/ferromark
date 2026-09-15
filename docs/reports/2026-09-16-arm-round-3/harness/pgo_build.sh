#!/bin/zsh
# usage: pgo_build.sh <source-checkout> <name>
# Builds an instrumented worker, trains it on the broad corpus (all stages), and builds a PGO-optimized worker.
# Produces $S/build-<name>/build.json compatible with run.py (baseline reused from build-aa).
set -e
source $(ls -d /private/tmp/claude-501/-Users-sebastian-Workspace-ferromark--claude-worktrees-code-audit-review-bb9235/*/scratchpad)/env.sh
SRC=$1; NAME=$2
ROOT=$S/build-$NAME; rm -rf $ROOT; mkdir -p $ROOT/src $ROOT/profraw
cp $S/worker.rs $ROOT/worker.rs; cp $S/worker.rs $ROOT/src/main.rs
cp $SRC/Cargo.lock $ROOT/Cargo.lock
cat > $ROOT/Cargo.toml <<TOML
[package]
name = "optimization-round-worker"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
ferromark = { path = "$SRC/crates/ferromark" }

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
TOML
export CARGO_TARGET_DIR=$ROOT/target-gen
unset CARGO_ENCODED_RUSTFLAGS
RUSTFLAGS="-C target-cpu=generic -Cprofile-generate=$ROOT/profraw" cargo +1.95 build --release --offline --manifest-path $ROOT/Cargo.toml > $ROOT/build-gen.log 2>&1
GEN=$ROOT/target-gen/release/optimization-round-worker
echo "instrumented: $GEN"
# training: every broad case in all four stages, short windows
python3 - "$GEN" <<'PY'
import json, os, subprocess, sys
S=os.environ["S"]; gen=sys.argv[1]
import re as _re
_flt=os.environ.get("TRAIN_FILTER")
cases=[c for c in json.load(open(f"{S}/corpus.json"))["cases"] if (_re.search(_flt,c["name"]) if _flt else (c.get("suite")=="broad" or c["name"].startswith(("scan-","table-","autolink-"))))]
for c in cases:
    for stage in ("fresh","reuse","parse","render"):
        p=subprocess.Popen([gen,c["profile"],stage,f"{S}/inputs/{c['name']}.md"],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
        out,_=p.communicate("verify\nbench 60000000\nquit\n")
        assert p.returncode==0,(c["name"],stage,p.returncode)
print("trained on",len(cases),"cases x 4 stages")
PY
PROFDATA=$(ls ~/.rustup/toolchains/1.95-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata)
$PROFDATA merge -o $ROOT/merged.profdata $ROOT/profraw/*.profraw
ls -la $ROOT/merged.profdata
export CARGO_TARGET_DIR=$ROOT/target
RUSTFLAGS="-C target-cpu=generic -Cprofile-use=$ROOT/merged.profdata -Cllvm-args=-pgo-warn-missing-function" cargo +1.95 build --release --offline --manifest-path $ROOT/Cargo.toml > $ROOT/build-use.log 2>&1
BIN=$ROOT/target/release/optimization-round-worker
python3 - "$ROOT" "$BIN" <<'PY'
import json, hashlib, sys, os
S=os.environ["S"]; root, binary = sys.argv[1], sys.argv[2]
base=json.load(open(f"{S}/{os.environ.get('BASE_BUILD','build-aa')}/build.json"))
sha=lambda p: hashlib.sha256(open(p,'rb').read()).hexdigest()
base["engines"]["candidate"]={"binary":binary,"binary_sha256":sha(binary),"lto":"fat","pgo":True,"cache":"built","source":root}
base["lto"]="fat+pgo(candidate)"
json.dump(base,open(f"{root}/build.json","w"),indent=2)
print("wrote",f"{root}/build.json")
PY
echo "pgo build done: $BIN"
