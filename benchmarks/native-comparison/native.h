// Build-only replacement for Bun's WebKit umbrella header. Search code is unchanged.
// OS() selects the same per-platform branch Bun's own build takes.
#pragma once
#define BUN__ROOT__H
#if defined(__APPLE__)
#define OS_DARWIN 1
#define OS_LINUX 0
#elif defined(__linux__)
#define OS_DARWIN 0
#define OS_LINUX 1
#else
#error "native comparison supports macOS and Linux only"
#endif
#define OS(x) OS_##x
#define ASSERT(...) ((void)0)
#define ASSERT_NOT_REACHED_WITH_MESSAGE(...) __builtin_trap()
