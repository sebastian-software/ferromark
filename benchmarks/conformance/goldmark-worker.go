// Untimed specification profile. Goldmark v2 has no public GFM tagfilter option.
package main

import (
	"bufio"
	"bytes"
	"encoding/hex"
	"fmt"
	"os"
	"runtime"
	"runtime/debug"

	"github.com/yuin/goldmark/v2/extension"
	"github.com/yuin/goldmark/v2/parser"
	"github.com/yuin/goldmark/v2/renderer/html"
)

func main() {
	args := os.Args[1:]
	if len(args) < 4 || args[0] != "goldmark" || (args[1] != "commonmark" && args[1] != "gfm" && args[1] != "gfm-shared") || (args[2] != "fresh" && args[2] != "reuse") {
		panic("worker goldmark PROFILE MODE INPUT...")
	}
	runtime.GOMAXPROCS(1)
	debug.SetGCPercent(100)
	debug.SetMemoryLimit(1<<63 - 1)
	parserOptions := []parser.Option{}
	rendererOptions := []html.Option{html.WithUnsafe()}
	if args[1] != "commonmark" {
		parserOptions = append(parserOptions, parser.WithExtensions(extension.NewTableParser(), extension.NewStrikethroughParser(), extension.NewTaskListItemParser(), extension.NewLinkifyParser()))
		rendererOptions = append(rendererOptions, html.WithExtensions(extension.NewTableHTMLRenderer(), extension.NewStrikethroughHTMLRenderer(), extension.NewTaskListItemHTMLRenderer()))
	}
	p := parser.New(parserOptions...)
	r := html.New(rendererOptions...)
	inputs := make([][]byte, len(args)-3)
	for i, path := range args[3:] {
		data, err := os.ReadFile(path)
		if err != nil {
			panic(err)
		}
		inputs[i] = data
	}
	render := func(source []byte) []byte {
		var output bytes.Buffer
		if err := r.Render(&output, source, p.Parse(source)); err != nil {
			panic(err)
		}
		return output.Bytes()
	}
	out := bufio.NewWriter(os.Stdout)
	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		line := scanner.Text()
		switch {
		case line == "quit":
			return
		case line == "verify":
			for i, source := range inputs {
				fmt.Fprintf(out, "html %d %s\n", i, hex.EncodeToString(render(source)))
			}
			fmt.Fprintln(out, "done")
		default:
			panic("unknown command")
		}
		if err := out.Flush(); err != nil {
			panic(err)
		}
	}
	if err := scanner.Err(); err != nil {
		panic(err)
	}
}
