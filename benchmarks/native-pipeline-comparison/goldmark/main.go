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

	v1 "github.com/yuin/goldmark"
	e1 "github.com/yuin/goldmark/extension"
	h1 "github.com/yuin/goldmark/renderer/html"
	e2 "github.com/yuin/goldmark/v2/extension"
	p2 "github.com/yuin/goldmark/v2/parser"
	h2 "github.com/yuin/goldmark/v2/renderer/html"
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
	if engine == "goldmark-v1" {
		ext := []v1.Extender{}
		if flags&1 != 0 {
			ext = append(ext, e1.Table)
		}
		if flags&2 != 0 {
			ext = append(ext, e1.Strikethrough)
		}
		if flags&4 != 0 {
			ext = append(ext, e1.TaskList)
		}
		md := v1.New(v1.WithExtensions(ext...), v1.WithRendererOptions(h1.WithUnsafe()))
		return func(source []byte) []byte {
			var out bytes.Buffer
			if err := md.Convert(source, &out); err != nil {
				panic(err)
			}
			return out.Bytes()
		}
	}
	if engine != "goldmark-v2" {
		panic("unknown engine")
	}
	pe, he := []p2.Extension{}, []h2.Extension{}
	if flags&1 != 0 {
		pe = append(pe, e2.TableParser)
		he = append(he, e2.TableHTMLRenderer)
	}
	if flags&2 != 0 {
		pe = append(pe, e2.StrikethroughParser)
		he = append(he, e2.StrikethroughHTMLRenderer)
	}
	if flags&4 != 0 {
		pe = append(pe, e2.TaskListItemParser)
		he = append(he, e2.TaskListItemHTMLRenderer)
	}
	p, r := p2.New(p2.WithExtensions(pe...)), h2.New(h2.WithUnsafe(), h2.WithExtensions(he...))
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
