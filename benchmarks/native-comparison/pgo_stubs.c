/* Link stubs for the instrumented PGO training worker only.
 *
 * The standalone Bun integration compiles Bun's Rust support crates without
 * WebKit, simdutf, c-ares, or Bun's process bootstrap.  In every ordinary
 * build that is invisible: nothing the six-engine worker calls reaches those
 * declarations, so fat LTO deletes the referencing code before linking.
 *
 * `-Cprofile-generate` attaches a counter to each instrumented function and
 * keeps it alive, so the throwaway training binary suddenly needs the symbols
 * its dead code names.  These definitions satisfy the linker for that one
 * binary.  They are not linked into the measured worker: the final
 * `-Cprofile-use` build carries no instrumentation, so the same dead code is
 * removed again and the measured executable links exactly like the default
 * build.
 *
 * Every function stub aborts.  Reaching one would mean the worker executed
 * Bun code outside its Markdown engine, which must fail loudly rather than
 * quietly differ from the default build.
 */
#include <stdlib.h>

#define FERROMARK_PGO_STUB(name) \
    void name(void); \
    void name(void) { abort(); }

FERROMARK_PGO_STUB(BunString__createExternalGloballyAllocatedLatin1)
FERROMARK_PGO_STUB(BunString__createExternalGloballyAllocatedUTF16)
FERROMARK_PGO_STUB(BunString__fromBytes)
FERROMARK_PGO_STUB(BunString__tryCreateAtom)
FERROMARK_PGO_STUB(Bun__WTFStringImpl__destroy)
FERROMARK_PGO_STUB(Bun__ttySetMode)
FERROMARK_PGO_STUB(Bun__visibleWidthExcludeANSI_utf8)
FERROMARK_PGO_STUB(Bun__visibleWidthExcludeANSI_utf8IndexAtWidth)
FERROMARK_PGO_STUB(URL__deinit)
FERROMARK_PGO_STUB(URL__fromString)
FERROMARK_PGO_STUB(URL__getHref)
FERROMARK_PGO_STUB(URL__hostname)
FERROMARK_PGO_STUB(URL__pathname)
FERROMARK_PGO_STUB(URL__protocol)
FERROMARK_PGO_STUB(ares_inet_pton)
FERROMARK_PGO_STUB(bun_initialize_process)
FERROMARK_PGO_STUB(bun_restore_stdio)
FERROMARK_PGO_STUB(on_before_reload_process_posix)
FERROMARK_PGO_STUB(simdutf__base64_encode)
FERROMARK_PGO_STUB(simdutf__convert_utf16le_to_utf8_with_errors)
FERROMARK_PGO_STUB(simdutf__convert_utf8_to_utf16le_with_errors)
FERROMARK_PGO_STUB(simdutf__convert_valid_utf16le_to_utf8)
FERROMARK_PGO_STUB(simdutf__utf16_length_from_utf8)
FERROMARK_PGO_STUB(simdutf__utf8_length_from_latin1)
FERROMARK_PGO_STUB(simdutf__utf8_length_from_utf16le)
FERROMARK_PGO_STUB(simdutf__utf8_length_from_utf16le_with_replacement)
FERROMARK_PGO_STUB(simdutf__validate_ascii)
FERROMARK_PGO_STUB(simdutf__validate_ascii_with_errors)
FERROMARK_PGO_STUB(simdutf__validate_utf8)

#if defined(__linux__)
/* GNU ld resolves every symbol the instrumented objects reference, where
 * ld64 on macOS first strips the dead functions naming them, so the Linux
 * training binary needs these as well. */
FERROMARK_PGO_STUB(BunString__createAtom)
FERROMARK_PGO_STUB(BunString__createStaticExternal)
FERROMARK_PGO_STUB(BunString__fromLatin1)
FERROMARK_PGO_STUB(BunString__fromLatin1Unitialized)
FERROMARK_PGO_STUB(BunString__fromUTF16)
FERROMARK_PGO_STUB(BunString__fromUTF16ToLatin1)
FERROMARK_PGO_STUB(BunString__fromUTF16Unitialized)
FERROMARK_PGO_STUB(BunString__makeThreadShareable)
FERROMARK_PGO_STUB(BunString__threadIsolatedCopy)
FERROMARK_PGO_STUB(Bun__WTFStringImpl__ensureHash)
FERROMARK_PGO_STUB(Bun__ramSize)
FERROMARK_PGO_STUB(Bun__visibleWidthExcludeANSI_latin1)
FERROMARK_PGO_STUB(URL__fragmentIdentifier)
FERROMARK_PGO_STUB(URL__getFileURLString)
FERROMARK_PGO_STUB(URL__getHrefJoin)
FERROMARK_PGO_STUB(URL__host)
FERROMARK_PGO_STUB(URL__href)
FERROMARK_PGO_STUB(URL__password)
FERROMARK_PGO_STUB(URL__pathFromFileURL)
FERROMARK_PGO_STUB(URL__port)
FERROMARK_PGO_STUB(URL__username)
FERROMARK_PGO_STUB(getRSS)
FERROMARK_PGO_STUB(is_executable_file)
FERROMARK_PGO_STUB(simdutf__base64_decode_from_binary)
FERROMARK_PGO_STUB(simdutf__base64_decode_from_binary_lenient)
FERROMARK_PGO_STUB(simdutf__base64_length_from_binary)
FERROMARK_PGO_STUB(simdutf__validate_utf16le)
FERROMARK_PGO_STUB(simdutf__validate_utf8_with_errors)
FERROMARK_PGO_STUB(sys_preadv2)
FERROMARK_PGO_STUB(sys_pwritev2)
FERROMARK_PGO_STUB(BunString__createStaticExternalLatin1WithHash)
FERROMARK_PGO_STUB(BunString__createStaticExternalUTF16WithHash)
FERROMARK_PGO_STUB(Bun__ANSI__next)
FERROMARK_PGO_STUB(Bun__linux_trace_emit)
FERROMARK_PGO_STUB(Bun__linux_trace_init)
FERROMARK_PGO_STUB(Bun__visibleWidthExcludeANSI_utf16)
FERROMARK_PGO_STUB(WTF__dtoa)
FERROMARK_PGO_STUB(WTF__numberOfProcessorCores)
FERROMARK_PGO_STUB(WTF__parseDouble)
FERROMARK_PGO_STUB(WTF__parseES5Date)
FERROMARK_PGO_STUB(__bun_crash_handler_dump_stack_trace)
FERROMARK_PGO_STUB(__bun_crash_handler_out_of_memory)

/* The one data symbol: Bun's C startup code records here which standard
 * streams are closed.  Zero means open, as for a worker started with pipes. */
#include <stdint.h>
int32_t bun_is_stdio_null[3];
#endif
