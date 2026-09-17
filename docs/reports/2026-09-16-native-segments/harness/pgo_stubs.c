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
 * Every stub aborts.  Reaching one would mean the worker executed Bun code
 * outside its Markdown engine, which must fail loudly rather than quietly
 * differ from the default build.
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
