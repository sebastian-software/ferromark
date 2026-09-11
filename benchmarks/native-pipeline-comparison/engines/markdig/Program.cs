using System.Diagnostics;
using System.Text;
using System.Text.Json;
using Markdig;
using Markdig.Extensions.EmphasisExtras;

static MarkdownPipeline Pipeline(uint flags)
{
    if (flags > 7) throw new ArgumentOutOfRangeException(nameof(flags));
    var builder = new MarkdownPipelineBuilder();
    if ((flags & 1) != 0) builder.UsePipeTables();
    if ((flags & 2) != 0) builder.UseEmphasisExtras(EmphasisExtraOptions.Strikethrough);
    if ((flags & 4) != 0) builder.UseTaskLists();
    return builder.Build();
}

if (args.Length != 2 || args[0] != "markdig") throw new ArgumentException("Expected markdig and a case file");
using var corpus = JsonDocument.Parse(File.ReadAllText(args[1]));
var cases = corpus.RootElement.EnumerateArray().Select(row => new Input(
    row.GetProperty("case").GetString()!, row.GetProperty("input").GetString()!, row.GetProperty("flags").GetUInt32())).ToArray();
var pipelines = cases.Select(row => Pipeline(row.Flags)).ToArray();
while (Console.ReadLine() is { } line)
{
    using var request = JsonDocument.Parse(line);
    var index = request.RootElement.GetProperty("index").GetInt32();
    var input = cases[index];
    var pipeline = pipelines[index];
    object result;
    switch (request.RootElement.GetProperty("op").GetString())
    {
        case "verify":
            result = new { @case = input.Name, engine = "markdig", flags = input.Flags,
                bytes = Encoding.UTF8.GetByteCount(input.Text), html = Markdown.ToHtml(input.Text, pipeline),
                options = $"CommonMark defaults; tables={(input.Flags & 1) != 0}; strikethrough={(input.Flags & 2) != 0}; tasks={(input.Flags & 4) != 0}; HTML/URLs preserved; no advanced extension bundle; normal automatic GC" };
            break;
        case "window":
            var ms = request.RootElement.GetProperty("ms").GetInt32();
            if (ms <= 0) throw new ArgumentOutOfRangeException(nameof(ms));
            // Counter reads are outside the timed render window. No forced GC.
            var collections = Enumerable.Range(0, 3).Select(GC.CollectionCount).ToArray();
            var allocated = GC.GetTotalAllocatedBytes();
            var start = Stopwatch.GetTimestamp();
            long count = 0;
            long elapsed;
            do
            {
                for (var n = 0; n < 16; n++) GC.KeepAlive(Markdown.ToHtml(input.Text, pipeline));
                count += 16;
                elapsed = Stopwatch.GetTimestamp() - start;
            } while (elapsed * 1000.0 / Stopwatch.Frequency < ms);
            var elapsedNs = checked((long)(elapsed * (1_000_000_000.0 / Stopwatch.Frequency)));
            result = new { @case = input.Name, engine = "markdig", count, elapsed_ns = elapsedNs,
                ns_per_render = (double)elapsedNs / count,
                gc_cycles = Enumerable.Range(0, 3).Select(i => GC.CollectionCount(i) - collections[i]).ToArray(),
                allocated_bytes = GC.GetTotalAllocatedBytes() - allocated, heap_bytes = GC.GetTotalMemory(false) };
            break;
        default: throw new ArgumentException("Unknown operation");
    }
    Console.WriteLine(JsonSerializer.Serialize(result));
}
record Input(string Name, string Text, uint Flags);
