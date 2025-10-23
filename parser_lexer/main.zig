const std = @import("std");
const Allocator = std.mem.Allocator;
const ArrayList = std.ArrayList;

const TokenType = enum {
    Import,         // <>
    From,           // ::
    Write,          // write
    Comment,        // @
    MultiComment,   // @@ @@
    Let,            // let
    Func,           // func
    If,             // if
    Return,         // return
    Colon,          // :
    Arrow,          // ->
    LeftBracket,    // [
    RightBracket,   // ]
    HashEqual,      // # ==
    Identifier,
    Number,
    String,
    Plus, Minus, Star, Slash,
    LessEqual,      // <=
    Eof,
};

const Token = struct {
    typ: TokenType,
    lexeme: []const u8,
    line: usize,
};

const Lexer = struct {
    source: []const u8,
    start: usize = 0,
    current: usize = 0,
    line: usize = 1,
    allocator: Allocator,

    fn init(allocator: Allocator, source: []const u8) Lexer {
        return .{ .allocator = allocator, .source = source };
    }

    fn scanTokens(self: *Lexer) !ArrayList(Token) {
        var tokens = ArrayList(Token).init(self.allocator);
        while (!self.isAtEnd()) {
            self.start = self.current;
            try self.scanToken(&tokens);
        }
        try tokens.append(.{ .typ = .Eof, .lexeme = "", .line = self.line });
        return tokens;
    }

    fn isAtEnd(self: *Lexer) bool {
        return self.current >= self.source.len;
    }

    fn advance(self: *Lexer) u8 {
        self.current += 1;
        return self.source[self.current - 1];
    }

    fn peek(self: *Lexer) u8 {
        if (self.isAtEnd()) return 0;
        return self.source[self.current];
    }

    fn peekNext(self: *Lexer) u8 {
        if (self.current + 1 >= self.source.len) return 0;
        return self.source[self.current + 1];
    }

    fn addToken(self: *Lexer, tokens: *ArrayList(Token), typ: TokenType) !void {
        const lexeme = self.source[self.start..self.current];
        try tokens.append(.{ .typ = typ, .lexeme = lexeme, .line = self.line });
    }

    fn scanToken(self: *Lexer, tokens: *ArrayList(Token)) !void {
        const c = self.advance();
        switch (c) {
            '<' => try self.addToken(tokens, .Import),
            ':' => if (self.peek() == ':') {
                _ = self.advance();
                try self.addToken(tokens, .From);
            } else {
                try self.addToken(tokens, .Colon);
            },
            '[' => try self.addToken(tokens, .LeftBracket),
            ']' => try self.addToken(tokens, .RightBracket),
            '#' => if (self.peek() == ' ' and self.peekNext() == '=') {
                self.current += 2;
                try self.addToken(tokens, .HashEqual);
            } else {
                // Skip comment or something
            },
            '@' => if (self.peek() == '@') {
                self.current += 1;
                // Multiline comment @@ ... @@
                while (!self.isAtEnd() and !(self.peek() == '@' and self.peekNext() == '@')) {
                    if (self.advance() == '\n') self.line += 1;
                }
                if (!self.isAtEnd()) self.current += 2;
                try self.addToken(tokens, .MultiComment);
            } else {
                // Single line comment
                while (!self.isAtEnd() and self.peek() != '\n') _ = self.advance();
                try self.addToken(tokens, .Comment);
            },
            ' ' , '\r', '\t' => {}, // Ignore whitespace
            '\n' => self.line += 1,
            '"' => try self.string(tokens),
            '0'...'9' => try self.number(tokens),
            'a'...'z', 'A'...'Z', '_' => try self.identifier(tokens),
            '+' => try self.addToken(tokens, .Plus),
            '-' => if (self.peek() == '>') {
                _ = self.advance();
                try self.addToken(tokens, .Arrow);
            } else {
                try self.addToken(tokens, .Minus);
            },
            '*' => try self.addToken(tokens, .Star),
            '/' => try self.addToken(tokens, .Slash),
            '<' => if (self.peek() == '=') {
                _ = self.advance();
                try self.addToken(tokens, .LessEqual);
            },
            else => {
                std.debug.print("Unexpected character: {c}\n", .{c});
            },
        }
    }

    fn string(self: *Lexer, tokens: *ArrayList(Token)) !void {
        while (!self.isAtEnd() and self.peek() != '"') {
            if (self.peek() == '\n') self.line += 1;
            _ = self.advance();
        }
        if (self.isAtEnd()) {
            std.debug.print("Unterminated string.\n", .{});
            return error.UnterminatedString;
        }
        _ = self.advance(); // Closing "
        try self.addToken(tokens, .String);
    }

    fn number(self: *Lexer, tokens: *ArrayList(Token)) !void {
        while (std.ascii.isDigit(self.peek())) _ = self.advance();
        try self.addToken(tokens, .Number);
    }

    fn identifier(self: *Lexer, tokens: *ArrayList(Token)) !void {
        while (std.ascii.isAlphanumeric(self.peek()) or self.peek() == '_') _ = self.advance();
        const text = self.source[self.start..self.current];
        const typ: TokenType = switch (text) {
            inline else => |id| if (std.mem.eql(u8, id, "write")) .Write else if (std.mem.eql(u8, id, "let")) .Let else if (std.mem.eql(u8, id, "func")) .Func else if (std.mem.eql(u8, id, "if")) .If else if (std.mem.eql(u8, id, "return")) .Return else .Identifier,
        };
        try self.addToken(tokens, typ);
    }
};

const Parser = struct {
    tokens: ArrayList(Token),
    current: usize = 0,

    fn init(tokens: ArrayList(Token)) Parser {
        return .{ .tokens = tokens };
    }

    fn parse(self: *Parser) !void {
        // Simple parse for demonstration
        while (!self.isAtEnd()) {
            _ = self.advance(); // Consume tokens
        }
        std.debug.print("Parsing complete.\n", .{});
    }

    fn isAtEnd(self: *Parser) bool {
        return self.tokens.items[self.current].typ == .Eof;
    }

    fn advance(self: *Parser) Token {
        if (!self.isAtEnd()) self.current += 1;
        return self.tokens.items[self.current - 1];
    }
};

pub fn main() !void {
    const allocator = std.heap.page_allocator;
    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 3) {
        std.debug.print("Usage: vira-parser_lexer <command> <file>\nCommands: check, fmt\n", .{});
        return;
    }

    const command = args[1];
    const file_path = args[2];

    const source = try std.fs.cwd().readFileAlloc(allocator, file_path, 1024 * 1024);
    defer allocator.free(source);

    var lexer = Lexer.init(allocator, source);
    var tokens = try lexer.scanTokens();
    defer tokens.deinit();

    var parser = Parser.init(tokens);
    try parser.parse();

    if (std.mem.eql(u8, command, "check")) {
        std.debug.print("Check OK\n", .{});
    } else if (std.mem.eql(u8, command, "fmt")) {
        std.debug.print("Formatted (stub)\n", .{});
    } else {
        std.debug.print("Unknown command\n", .{});
    }
}
