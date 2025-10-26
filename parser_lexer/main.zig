const std = @import("std");
const lexer = @import("lexer.zig");
const parser = @import("parser.zig");
const format = @import("formatter.zig").format;

pub fn main() !void {
    const allocator = std.heap.page_allocator;
    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 3) {
        std.debug.print("Usage: vira-parser_lexer <command> <file> [options]\nCommands: check, fmt\n", .{});
        return;
    }

    const command = args[1];
    const file_path = args[2];

    var file = try std.fs.cwd().openFile(file_path, .{});
    defer file.close();
    const source = try file.readToEndAlloc(allocator, 1024 * 1024);
    defer allocator.free(source);

    var lex = lexer.Lexer.init(allocator, source);
    var tokens = try lex.scanTokens();
    defer tokens.deinit();

    var pars = parser.Parser.init(allocator, tokens);
    defer pars.deinit();

    _ = try pars.parse(); // Build AST

    if (std.mem.eql(u8, command, "check")) {
        std.debug.print("Syntax check: OK\n", .{});
    } else if (std.mem.eql(u8, command, "fmt")) {
        const formatted = try format(allocator, source);
        defer allocator.free(formatted);
        try std.fs.cwd().writeFile(.{ .sub_path = file_path, .data = formatted });
        std.debug.print("Formatted {s}\n", .{file_path});
    } else {
        std.debug.print("Unknown command: {s}\n", .{command});
    }
}
