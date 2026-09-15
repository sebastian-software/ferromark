// Build-only replacement for Bun's WebKit umbrella header. Search code is unchanged.
#pragma once
#define BUN__ROOT__H
#define OS_DARWIN 1
#define OS_LINUX 0
#define OS(x) OS_##x
#define ASSERT(...) ((void)0)
#define ASSERT_NOT_REACHED_WITH_MESSAGE(...) __builtin_trap()
