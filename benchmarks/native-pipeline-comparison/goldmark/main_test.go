package main

import (
	"strings"
	"testing"
)

func TestIndependentFeaturesAndTrustedHTML(t *testing.T) {
	input := []byte("| A |\n| --- |\n| B |\n\n~~old~~\n\n- [x] done\n- [ ] pending\n\nwww.example.com\n\n<script>x</script>\n")
	const engine = "goldmark"
	for flags := uint32(0); flags <= 7; flags++ {
		out := string(renderer(engine, flags)(input))
		if strings.Contains(out, "<table>") != (flags&1 != 0) || strings.Contains(out, "<del>old</del>") != (flags&2 != 0) {
			t.Fatal(engine, flags, out)
		}
		checks := 0
		if flags&4 != 0 {
			checks = 2
		}
		if strings.Count(out, "type=\"checkbox\"") != checks {
			t.Fatal(engine, flags, out)
		}
		checked := 0
		if flags&4 != 0 {
			checked = 1
		}
		if strings.Count(out, "checked") != checked {
			t.Fatal(engine, flags, out)
		}
		if strings.Contains(out, "href=") || !strings.Contains(out, "<script>x</script>") {
			t.Fatal(engine, flags, out)
		}
	}
}

func TestSingleTildeDialectIsRecordedRatherThanPatched(t *testing.T) {
	const engine = "goldmark"
	if out := string(renderer(engine, 2)([]byte("~single~\n"))); !strings.Contains(out, "<del>single</del>") {
		t.Fatal(engine, out)
	}
}
