// Persistent native workers; all protocol work stays outside the rendering timer.
package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"runtime"
	"time"

	"github.com/yuin/goldmark/v2/extension"
	"github.com/yuin/goldmark/v2/parser"
	"github.com/yuin/goldmark/v2/renderer/html"
)

type fixture struct {
	Case  string `json:"case"`
	Input string `json:"input"`
	Flags uint32 `json:"flags"`
}

// Each call allocates its own AST and output. Only immutable engine configuration
// is reused, as recommended by the native APIs. Go's normal GC remains enabled.
func renderer(engine string, flags uint32) func([]byte) []byte {
	if flags > 7 {
		panic("unknown flags")
	}
	if engine != "goldmark" {
		panic("unknown engine")
	}
	pe, he := []parser.Extension{}, []html.Extension{}
	if flags&1 != 0 {
		pe = append(pe, extension.TableParser)
		he = append(he, extension.TableHTMLRenderer)
	}
	if flags&2 != 0 {
		pe = append(pe, extension.StrikethroughParser)
		he = append(he, extension.StrikethroughHTMLRenderer)
	}
	if flags&4 != 0 {
		pe = append(pe, extension.TaskListItemParser)
		he = append(he, extension.TaskListItemHTMLRenderer)
	}
	p, r := parser.New(parser.WithExtensions(pe...)), html.New(html.WithUnsafe(), html.WithExtensions(he...))
	return func(source []byte) []byte {
		var out bytes.Buffer
		doc := p.Parse(source)
		if err := r.Render(&out, source, doc); err != nil {
			panic(err)
		}
		return out.Bytes()
	}
}

func main() {
	engine := os.Args[1]
	data, err := os.ReadFile(os.Args[2])
	if err != nil {
		panic(err)
	}
	var cases []fixture
	if err := json.Unmarshal(data, &cases); err != nil {
		panic(err)
	}
	renders := make([]func([]byte) []byte, len(cases))
	inputs := make([][]byte, len(cases))
	for i, c := range cases {
		renders[i] = renderer(engine, c.Flags)
		inputs[i] = []byte(c.Input)
	}
	scanner, output := bufio.NewScanner(os.Stdin), json.NewEncoder(os.Stdout)
	for scanner.Scan() {
		var request struct {
			Op    string `json:"op"`
			Index int    `json:"index"`
			MS    uint64 `json:"ms"`
		}
		if err := json.Unmarshal(scanner.Bytes(), &request); err != nil {
			panic(err)
		}
		c, input, render := cases[request.Index], inputs[request.Index], renders[request.Index]
		result := map[string]any{"case": c.Case, "engine": engine}
		switch request.Op {
		case "verify":
			result["html"], result["flags"], result["bytes"] = string(render(input)), c.Flags, len(input)
			result["options"] = fmt.Sprintf("flags=%d; trusted HTML and URLs; no linkify, tagfilter, typography or extra extensions", c.Flags)
		case "window":
			if request.MS == 0 {
				panic("empty timing window")
			}
			var before, after runtime.MemStats
			runtime.ReadMemStats(&before)
			start, count := time.Now(), uint64(0)
			for {
				for j := 0; j < 16; j++ {
					value := render(input)
					runtime.KeepAlive(value)
				}
				count += 16
				if time.Since(start) >= time.Duration(request.MS)*time.Millisecond {
					break
				}
			}
			elapsed := time.Since(start).Nanoseconds()
			runtime.ReadMemStats(&after)
			result["count"], result["elapsed_ns"], result["ns_per_render"] = count, elapsed, float64(elapsed)/float64(count)
			result["gc_cycles"], result["allocated_bytes"], result["heap_bytes_after"] = after.NumGC-before.NumGC, after.TotalAlloc-before.TotalAlloc, after.HeapAlloc
		default:
			panic("unknown operation")
		}
		if err := output.Encode(result); err != nil {
			panic(err)
		}
	}
	if err := scanner.Err(); err != nil {
		panic(err)
	}
}
