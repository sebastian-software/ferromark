/* Native cmark API adapter. File I/O and command handling stay outside timing. */
#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
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
        const char *names[] = {"table", "strikethrough", "tasklist"};
        for (size_t i = 0; i < 3; i++) {
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
static uint64_t now_ns(void) {
    struct timespec ts;
    assert(clock_gettime(CLOCK_MONOTONIC, &ts) == 0);
    return (uint64_t)ts.tv_sec * UINT64_C(1000000000) + (uint64_t)ts.tv_nsec;
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
        } else if (!strncmp(line, "bench ", 6)) {
            uint64_t budget = strtoull(line + 6, NULL, 10);
            assert(budget > 0);
            uint64_t start = now_ns(), elapsed, iterations = 0;
            size_t checksum = 0;
            do {
                for (size_t cycle = 0; cycle < 32; cycle++) {
                    for (size_t i = 0; i < count; i++) {
                        char *html = render(&inputs[i], gfm);
                        assert(html);
                        /* Observe the complete returned string before its owner frees it. */
                        checksum += strlen(html);
                        free(html);
                    }
                }
                iterations += 32;
                elapsed = now_ns() - start;
            } while (elapsed < budget);
            printf("timing %" PRIu64 " %" PRIu64 " %zu\n", iterations, elapsed, checksum);
        } else if (!strcmp(line, "quit\n")) break;
        else abort();
        fflush(stdout);
    }
    free(line);
    for (size_t i = 0; i < count; i++) free(inputs[i].data);
    free(inputs);
    return 0;
}
