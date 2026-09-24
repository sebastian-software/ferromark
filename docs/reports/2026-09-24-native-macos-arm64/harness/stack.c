// Standalone equivalent of Bun's cached, per-thread lower stack bound.
// The parser's original reserve threshold and recursion checks remain intact.
// macOS reads the stack origin and size of the current pthread; Linux reads
// the same bounds through pthread_getattr_np, as WTF::StackBounds does there.
#if defined(__linux__)
#define _GNU_SOURCE
#endif
#include <pthread.h>
#include <stdint.h>
#include <stdlib.h>

static _Thread_local uintptr_t stack_end;

void Bun__StackCheck__initialize(void) {
    if (stack_end) return;
#if defined(__APPLE__)
    pthread_t thread = pthread_self();
    uintptr_t top = (uintptr_t)pthread_get_stackaddr_np(thread);
    size_t size = pthread_get_stacksize_np(thread);
    if (!top || !size || top <= size) abort();
    stack_end = top - size;
#elif defined(__linux__)
    pthread_attr_t attributes;
    void* lowest = NULL;
    size_t size = 0;
    if (pthread_getattr_np(pthread_self(), &attributes)) abort();
    int failed = pthread_attr_getstack(&attributes, &lowest, &size);
    pthread_attr_destroy(&attributes);
    // pthread_attr_getstack reports the lowest usable address directly.
    if (failed || !lowest || !size) abort();
    stack_end = (uintptr_t)lowest;
#else
#error "native comparison supports macOS and Linux only"
#endif
}

void* Bun__StackCheck__getMaxStack(void) {
    if (!stack_end) abort();
    return (void*)stack_end;
}
