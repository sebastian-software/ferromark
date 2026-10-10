/* Untimed cmark/cmark-gfm public specification configurations. */
#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef GFM
#include "cmark-gfm.h"
#include "cmark-gfm-extension_api.h"
#include "cmark-gfm-core-extensions.h"
#else
#include "cmark.h"
#endif

struct input { char *data; size_t length; };

static char *render(const struct input *input, int gfm) {
#ifdef GFM
    cmark_parser *parser = cmark_parser_new(CMARK_OPT_UNSAFE);
    assert(parser);
    if (gfm) {
        const char *names[] = {"table", "strikethrough", "tasklist", "autolink", "tagfilter"};
        for (size_t i = 0; i < 5; i++) {
            cmark_syntax_extension *extension = cmark_find_syntax_extension(names[i]);
            assert(extension);
            cmark_parser_attach_syntax_extension(parser, extension);
        }
    }
    cmark_parser_feed(parser, input->data, input->length);
    cmark_node *document = cmark_parser_finish(parser);
    assert(document);
    char *html = cmark_render_html(document, CMARK_OPT_UNSAFE,
                                  cmark_parser_get_syntax_extensions(parser));
    cmark_node_free(document);
    cmark_parser_free(parser);
    return html;
#else
    assert(!gfm);
    return cmark_markdown_to_html(input->data, input->length, CMARK_OPT_UNSAFE);
#endif
}
int main(int argc, char **argv) {
    assert(argc >= 5);
    int gfm = strcmp(argv[2], "commonmark") != 0;
    assert(!strcmp(argv[2], "commonmark") || !strcmp(argv[2], "gfm") || !strcmp(argv[2], "gfm-shared"));
    assert(!strcmp(argv[3], "fresh") || !strcmp(argv[3], "reuse"));
#ifdef GFM
    assert(!strcmp(argv[1], "cmark-gfm"));
    cmark_gfm_core_extensions_ensure_registered();
#else
    assert(!strcmp(argv[1], "cmark") && !gfm);
#endif
    size_t count = (size_t)argc - 4;
    struct input *inputs = calloc(count, sizeof(*inputs));
    assert(inputs);
    for (size_t i = 0; i < count; i++) {
        FILE *file = fopen(argv[i + 4], "rb");
        assert(file && fseek(file, 0, SEEK_END) == 0);
        long length = ftell(file);
        assert(length >= 0 && fseek(file, 0, SEEK_SET) == 0);
        inputs[i].length = (size_t)length;
        inputs[i].data = malloc((size_t)length + 1);
        assert(inputs[i].data && fread(inputs[i].data, 1, (size_t)length, file) == (size_t)length);
        inputs[i].data[length] = '\0';
        fclose(file);
    }
    char *line = NULL;
    size_t capacity = 0;
    while (getline(&line, &capacity, stdin) > 0) {
        if (!strcmp(line, "verify\n")) {
            for (size_t i = 0; i < count; i++) {
                char *html = render(&inputs[i], gfm);
                assert(html);
                printf("html %zu ", i);
                for (const unsigned char *p = (unsigned char *)html; *p; p++) printf("%02x", *p);
                putchar('\n');
                free(html);
            }
            puts("done");
        } else if (!strcmp(line, "quit\n")) break;
        else abort();
        fflush(stdout);
    }
    free(line);
    for (size_t i = 0; i < count; i++) free(inputs[i].data);
    free(inputs);
    return 0;
}
