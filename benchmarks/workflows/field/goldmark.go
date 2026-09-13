// Persistent native workers; all protocol work stays outside the rendering timer.
package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"os"
	"runtime"
	"time"

	"github.com/yuin/goldmark/v2/extension"
	"github.com/yuin/goldmark/v2/parser"
	"github.com/yuin/goldmark/v2/renderer/html"
)

type fixture struct {
	Case  string `json:"id"`
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
	var corpus map[string]json.RawMessage
	if err := json.Unmarshal(data, &corpus); err != nil {
		panic(err)
	}
	var cases []fixture
	if err := json.Unmarshal(corpus[os.Args[3]], &cases); err != nil {
		panic(err)
	}
	render := renderer(engine, 7)
	inputs := make([][]byte, len(cases))
	for i, c := range cases {
		inputs[i] = []byte(c.Input)
	}
	retain := os.Args[4] == "retain"
	run := func() uint64 {
		var kept [][]byte
		var units uint64
		for _, input := range inputs {
			html := render(input)
			units += uint64(len(html))
			if retain {
				kept = append(kept, html)
			} else {
				runtime.KeepAlive(html)
			}
		}
		runtime.KeepAlive(kept)
		return units
	}
	scanner, output := bufio.NewScanner(os.Stdin), json.NewEncoder(os.Stdout)
	for scanner.Scan() {
		var request struct {
			Action string `json:"action"`
			MS     uint64 `json:"milliseconds"`
		}
		if err := json.Unmarshal(scanner.Bytes(), &request); err != nil {
			panic(err)
		}
		var result any
		switch request.Action {
		case "verify":
			rows := make([]map[string]any, len(cases))
			for i, c := range cases {
				rows[i] = map[string]any{"id": c.Case, "html": string(render(inputs[i])), "metadata": nil}
			}
			result = map[string]any{"engine": engine, "group": os.Args[3], "retain": retain, "outputs": rows,
				"options": "tables, strikethrough, tasks; trusted HTML/URLs; fresh AST and bytes.Buffer; normal GC; immutable parser configuration"}
		case "time":
			var before, after runtime.MemStats
			runtime.ReadMemStats(&before)
			start, count, units := time.Now(), uint64(0), uint64(0)
			for {
				for j := 0; j < 4; j++ {
					units += run()
					count++
				}
				if time.Since(start) >= time.Duration(request.MS)*time.Millisecond {
					break
				}
			}
			elapsed := time.Since(start).Nanoseconds()
			runtime.ReadMemStats(&after)
			result = map[string]any{"iterations": count, "elapsed_ns": elapsed, "output_units": units,
				"ns_per_workload": float64(elapsed) / float64(count), "gc_cycles": after.NumGC - before.NumGC}
		default:
			panic("unknown action")
		}
		if err := output.Encode(result); err != nil {
			panic(err)
		}
	}
	if err := scanner.Err(); err != nil {
		panic(err)
	}
}
