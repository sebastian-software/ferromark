import { nativeMatrix, nativePackages } from "ferramenta-native";

import config from "../native-npm.config.mjs";
const matrix = nativeMatrix();
const packages = nativePackages(config);
for (const target of matrix.include) {
  target.pgo = target.id !== "win32-arm64-msvc";
  target.artifact = `native-${target.id}`;
  target.binary = packages.find((entry) => entry.id === target.id).binaryName;
}
console.log(JSON.stringify(matrix));
