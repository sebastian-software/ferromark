// macOS standalone equivalent of Bun's cached, per-thread lower stack bound.
// The parser's original reserve threshold and recursion checks remain intact.
#include <pthread.h>
#include <stdint.h>
#include <stdlib.h>
static _Thread_local uintptr_t stack_end;
void Bun__StackCheck__initialize(void) {
    if (stack_end) return;
    pthread_t thread = pthread_self();
    uintptr_t top = (uintptr_t)pthread_get_stackaddr_np(thread);
    size_t size = pthread_get_stacksize_np(thread);
    if (!top || !size || top <= size) abort();
    stack_end = top - size;
}
void* Bun__StackCheck__getMaxStack(void) {
    if (!stack_end) abort();
    return (void*)stack_end;
}
