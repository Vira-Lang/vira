const std = @import("std");
const Allocator = std.mem.Allocator;

pub fn format(allocator: Allocator, source: []const u8) ![]u8 {
    // Prosty formatter: dodaj spacje, nowe linie itp.
    // Rozbudowano: np. indent blocks
    var formatted = try allocator.alloc(u8, source.len * 2);
    // Usunięto defer allocator.free(formatted), ponieważ zwracamy bufor i caller musi go zwolnić
    var i: usize = 0;
    var j: usize = 0;
    var indent: usize = 0;
    while (i < source.len) : (i += 1) {
        const c = source[i];
        if (c == '[') {
            formatted[j] = c;
            j += 1;
            indent += 4;
            formatted[j] = '\n';
            j += 1;
            for (0..indent) |_| {
                formatted[j] = ' ';
                j += 1;
            }
        } else if (c == ']') {
            indent -= 4;
            formatted[j] = '\n';
            j += 1;
            for (0..indent) |_| {
                formatted[j] = ' ';
                j += 1;
            }
            formatted[j] = c;
            j += 1;
        } else {
            formatted[j] = c;
            j += 1;
        }
    }
    return try allocator.realloc(formatted, j); // Przytnij
}

// Dodano więcej funkcji, np. formatExpr
pub fn formatExpr(expr: []const u8) []u8 {
    // Stub
    return expr;
}
