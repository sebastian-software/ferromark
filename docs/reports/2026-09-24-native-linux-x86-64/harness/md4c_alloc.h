// Benchmark-only allocator binding; retain md4c's parser and renderer sources.
// Include libc declarations before mapping calls to the shared Bun mimalloc.
#pragma once
#include <stdlib.h>
#include <mimalloc.h>
#define malloc mi_malloc
#define calloc mi_calloc
#define realloc mi_realloc
#define free mi_free
