//! Native MD4X output diagnostics. This release cannot match independent syntax flags.
const std = @import("std");
const md4x = @import("md4x");
const c = @cImport({
    @cInclude("stdio.h");
    @cInclude("stdlib.h");
});
const allocator = std.heap.c_allocator;
const Case = struct { case: []const u8, input: []const u8, flags: u32 };
const Request = struct { op: []const u8, index: usize };
const Output = struct {
    data: std.ArrayList(u8) = .empty,
    failed: bool = false,
};

fn append(bytes: [*c]const u8, len: c_uint, userdata: ?*anyopaque) void {
    const output: *Output = @ptrCast(@alignCast(userdata.?));
    output.data.appendSlice(allocator, bytes[0..len]) catch {
        output.failed = true;
    };
}

fn render(input: []const u8) ![]u8 {
    var output: Output = .{};
    errdefer output.data.deinit(allocator);
    const status = md4x.md_html(input.ptr, @intCast(input.len), append, &output, 0);
    if (status != 0 or output.failed) return error.RenderFailed;
    return output.data.toOwnedSlice(allocator);
}

pub fn main(init: std.process.Init) !void {
    const args = try init.minimal.args.toSlice(allocator);
    defer allocator.free(args);
    if (args.len != 3 or !std.mem.eql(u8, args[1], "md4x")) return error.BadArguments;
    const file = c.fopen(args[2], "rb") orelse return error.OpenFailed;
    defer _ = c.fclose(file);
    if (c.fseek(file, 0, c.SEEK_END) != 0) return error.SeekFailed;
    const size = c.ftell(file);
    if (size < 0 or c.fseek(file, 0, c.SEEK_SET) != 0) return error.SeekFailed;
    const bytes = try allocator.alloc(u8, @intCast(size));
    defer allocator.free(bytes);
    if (c.fread(bytes.ptr, 1, bytes.len, file) != bytes.len) return error.ReadFailed;
    const cases = try std.json.parseFromSlice([]Case, allocator, bytes, .{});
    defer cases.deinit();
    const input_stream = c.fdopen(0, "r") orelse return error.OpenFailed;
    defer _ = c.fclose(input_stream);
    const output_stream = c.fdopen(1, "w") orelse return error.OpenFailed;
    defer _ = c.fclose(output_stream);
    var line: [*c]u8 = null;
    var capacity: usize = 0;
    defer c.free(line);
    while (true) {
        const length = c.getline(&line, &capacity, input_stream);
        if (length < 0) break;
        const request = try std.json.parseFromSlice(Request, allocator, line[0..@intCast(length)], .{ .ignore_unknown_fields = true });
        defer request.deinit();
        if (!std.mem.eql(u8, request.value.op, "verify")) return error.IncomparableDialectCannotBeTimed;
        if (request.value.index >= cases.value.len) return error.BadIndex;
        const case = cases.value[request.value.index];
        const html = try render(case.input);
        defer allocator.free(html);
        const json = try std.json.Stringify.valueAlloc(allocator, .{
            .case = case.case,
            .engine = "md4x",
            .flags = case.flags,
            .bytes = case.input.len,
            .html = html,
            .options = "md_html(renderer_flags=0); fixed extended syntax, requested feature switches unavailable; emoji=false (upstream default); no healing, heading IDs, highlighting or document wrapper",
        }, .{});
        defer allocator.free(json);
        if (c.fwrite(json.ptr, 1, json.len, output_stream) != json.len) return error.WriteFailed;
        if (c.fputc('\n', output_stream) < 0 or c.fflush(output_stream) != 0) return error.WriteFailed;
    }
}
