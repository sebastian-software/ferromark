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

using var corpus = JsonDocument.Parse(File.ReadAllText(args[1]));
var cases = corpus.RootElement.GetProperty(args[2]).EnumerateArray().Select(row => new Input(
    row.GetProperty("id").GetString()!, row.GetProperty("input").GetString()!)).ToArray();
var pipeline = Pipeline(7);
var retain = args[3] == "retain";
long Run()
{
    List<string>? kept = retain ? new() : null;
    long units = 0;
    foreach (var input in cases)
    {
        var html = Markdown.ToHtml(input.Text, pipeline);
        units += html.Length;
        if (kept is not null) kept.Add(html); else GC.KeepAlive(html);
    }
    GC.KeepAlive(kept);
    return units;
}
while (Console.ReadLine() is { } line)
{
    using var request = JsonDocument.Parse(line);
    object result;
    switch (request.RootElement.GetProperty("action").GetString())
    {
        case "verify":
            result = new { engine = "markdig", group = args[2], retain,
                outputs = cases.Select(input => new { id = input.Name, html = Markdown.ToHtml(input.Text, pipeline), metadata = (string?)null }),
                options = "tables, strikethrough, tasks; trusted HTML/URLs; immutable pipeline; fresh owned UTF-16 output; normal automatic GC" };
            break;
        case "time":
            var ms = request.RootElement.GetProperty("milliseconds").GetInt32();
            var collections = Enumerable.Range(0, 3).Select(GC.CollectionCount).ToArray();
            var start = Stopwatch.GetTimestamp();
            long count = 0, units = 0, elapsed;
            do
            {
                for (var n = 0; n < 4; n++) { units += Run(); count++; }
                elapsed = Stopwatch.GetTimestamp() - start;
            } while (elapsed * 1000.0 / Stopwatch.Frequency < ms);
            var elapsedNs = checked((long)(elapsed * (1_000_000_000.0 / Stopwatch.Frequency)));
            result = new { iterations = count, elapsed_ns = elapsedNs, output_units = units,
                ns_per_workload = (double)elapsedNs / count,
                gc_cycles = Enumerable.Range(0, 3).Select(i => GC.CollectionCount(i) - collections[i]).ToArray() };
            break;
        default: throw new ArgumentException("Unknown action");
    }
    Console.WriteLine(JsonSerializer.Serialize(result));
}
record Input(string Name, string Text);
