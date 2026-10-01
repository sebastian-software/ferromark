// Private local-build target selection; this does not add published sidecars.
import { readFileSync } from "node:fs";
import { linuxLibc, nativeTarget } from "../../node/ferromark/native-target.mjs";

export function benchmarkTarget(platform, arch, libc) {
  if (platform === "darwin" && arch === "x64") return "darwin-x64";
  return nativeTarget(platform, arch, libc);
}

export function localTarget() {
  const libc =
    process.platform === "linux"
      ? linuxLibc(process.report?.getReport?.(), () => {
          try {
            return readFileSync("/usr/bin/ldd", "utf8");
          } catch {
            return "";
          }
        })
      : undefined;
  return benchmarkTarget(process.platform, process.arch, libc);
}
