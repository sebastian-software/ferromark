export type DiagnosticReport = {
  header?: { glibcVersionRuntime?: string };
  sharedObjects?: readonly string[];
};
export declare const NATIVE_TARGETS: ReadonlyArray<{
  id: string;
  triple: string;
  os: string;
  cpu: string;
  libc?: string;
}>;
export declare function linuxLibc(
  report: DiagnosticReport | undefined,
  helper: () => string,
): "gnu" | "musl";
export declare function diagnosticReport(): DiagnosticReport | undefined;
export declare function detectLinuxLibc(platform?: string): "gnu" | "musl" | undefined;
export declare function nativeTarget(platform: string, arch: string, libc?: string): string;
export declare function createNativeLoader(
  parentUrl: string,
  config: { name: string; packagePattern: string; binaryPattern: string; localPatterns?: string[] },
): () => unknown;
