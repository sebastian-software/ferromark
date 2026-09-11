/* Native public API adapter; upstream parser and renderer sources are unchanged. */
#include <stdlib.h>
#ifdef BENCH_GFM
#include <cmark-gfm.h>
#include <cmark-gfm-extension_api.h>
#include <cmark-gfm-core-extensions.h>
static cmark_syntax_extension *extensions[3];
#else
#include <cmark.h>
#endif

int bench_cmark_init(void) {
#ifdef BENCH_GFM
  const char *names[] = {"table", "strikethrough", "tasklist"};
  cmark_gfm_core_extensions_ensure_registered();
  for (unsigned i = 0; i < 3; i++) {
    extensions[i] = cmark_find_syntax_extension(names[i]);
    if (!extensions[i]) return 0;
  }
#endif
  return 1;
}

int bench_cmark_options(unsigned flags) {
  int options = CMARK_OPT_UNSAFE;
#ifdef BENCH_GFM
  if (flags & 2u) options |= CMARK_OPT_STRIKETHROUGH_DOUBLE_TILDE;
#else
  (void)flags;
#endif
  return options;
}

char *bench_cmark_render(const char *input, size_t len, unsigned flags, int options) {
  if (flags & ~7u) return NULL;
#ifndef BENCH_GFM
  if (flags) return NULL;
#endif
  cmark_parser *parser = cmark_parser_new(options);
  if (!parser) return NULL;
#ifdef BENCH_GFM
  for (unsigned i = 0; i < 3; i++) {
    if ((flags & (1u << i)) &&
        !cmark_parser_attach_syntax_extension(parser, extensions[i])) {
      cmark_parser_free(parser);
      return NULL;
    }
  }
#endif
  cmark_parser_feed(parser, input, len);
  cmark_node *doc = cmark_parser_finish(parser);
  if (!doc) {
    cmark_parser_free(parser);
    return NULL;
  }
#ifdef BENCH_GFM
  char *html = cmark_render_html(doc, options,
                               cmark_parser_get_syntax_extensions(parser));
#else
  char *html = cmark_render_html(doc, options);
#endif
  cmark_node_free(doc);
  cmark_parser_free(parser);
  return html;
}

void bench_cmark_free(char *html) {
  cmark_get_default_mem_allocator()->free(html);
}
