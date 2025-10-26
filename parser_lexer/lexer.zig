const std = @import("std");
const ArrayList = std.ArrayList;
const Allocator = std.mem.Allocator;

pub const TokenType = enum {
    Import, From, Write, Comment, MultiComment, Let, Func, If, Else, While, For, Return, Colon, Arrow, LeftBracket, RightBracket, LeftParen, RightParen, Comma, HashEqual, Equals, Plus, Minus, Star, Slash, Mod, Less, Greater, LessEqual, GreaterEqual, EqualEqual, BangEqual, Bang, And, Or, Identifier, Number, Float, String, IntType, FloatType, StringType, BoolType, ArrayType, True, False, Eof,
};

pub const Token = struct {
    typ: TokenType,
    lexeme: []const u8,
    line: usize,
    column: usize,
};

pub const Lexer = struct {
    source: []const u8,
    start: usize = 0,
    current: usize = 0,
    line: usize = 1,
    column: usize = 1,
    allocator: Allocator,

    pub fn init(allocator: Allocator, source: []const u8) Lexer {
        return .{ .allocator = allocator, .source = source };
    }

    pub fn scanTokens(self: *Lexer) !ArrayList(Token) {
        var tokens = ArrayList(Token).init(self.allocator);
        errdefer tokens.deinit();

        while (!self.isAtEnd()) {
            self.start = self.current;
            try self.scanToken(&tokens);
        }
        try tokens.append(.{ .typ = .Eof, .lexeme = "", .line = self.line, .column = self.column });
        return tokens;
    }

    pub fn isAtEnd(self: *Lexer) bool {
        return self.current >= self.source.len;
    }

    pub fn advance(self: *Lexer) u8 {
        self.current += 1;
        self.column += 1;
        return self.source[self.current - 1];
    }

    pub fn peek(self: *Lexer) u8 {
        if (self.isAtEnd()) return 0;
        return self.source[self.current];
    }

    pub fn peekNext(self: *Lexer) u8 {
        if (self.current + 1 >= self.source.len) return 0;
        return self.source[self.current + 1];
    }

    pub fn addToken(self: *Lexer, tokens: *ArrayList(Token), typ: TokenType) !void {
        const lexeme = self.source[self.start..self.current];
        try tokens.append(.{ .typ = typ, .lexeme = lexeme, .line = self.line, .column = self.start - self.lineStart() + 1 });
    }

    pub fn lineStart(self: *Lexer) usize {
        var i = self.start;
        while (i > 0 and self.source[i - 1] != '\n') i -= 1;
        return i;
    }

    pub fn skipWhitespace(self: *Lexer) void {
        while (!self.isAtEnd()) {
            const c = self.peek();
            switch (c) {
                ' ', '\r', '\t' => _ = self.advance(),
                '\n' => {
                    _ = self.advance();
                    self.line += 1;
                    self.column = 1;
                },
                else => break,
            }
        }
    }

    pub fn scanToken(self: *Lexer, tokens: *ArrayList(Token)) !void {
        self.skipWhitespace();
        if (self.isAtEnd()) return;

        self.start = self.current;
        const c = self.advance();

        switch (c) {
            '<' => if (self.peek() == '>') {
                _ = self.advance();
                try self.addToken(tokens, .Import);
            } else if (self.peek() == '=') {
                _ = self.advance();
                try self.addToken(tokens, .LessEqual);
            } else {
                try self.addToken(tokens, .Less);
            },
            '>' => if (self.peek() == '=') {
                _ = self.advance();
                try self.addToken(tokens, .GreaterEqual);
            } else {
                try self.addToken(tokens, .Greater);
            },
            ':' => if (self.peek() == ':') {
                _ = self.advance();
                try self.addToken(tokens, .From);
            } else {
                try self.addToken(tokens, .Colon);
            },
            '[' => try self.addToken(tokens, .LeftBracket),
            ']' => try self.addToken(tokens, .RightBracket),
            '(' => try self.addToken(tokens, .LeftParen),
            ')' => try self.addToken(tokens, .RightParen),
            ',' => try self.addToken(tokens, .Comma),
            '=' => if (self.peek() == '=') {
                _ = self.advance();
                try self.addToken(tokens, .EqualEqual);
            } else {
                try self.addToken(tokens, .Equals);
            },
            '+' => try self.addToken(tokens, .Plus),
            '-' => if (self.peek() == '>') {
                _ = self.advance();
                try self.addToken(tokens, .Arrow);
            } else {
                try self.addToken(tokens, .Minus);
            },
            '*' => try self.addToken(tokens, .Star),
            '/' => try self.addToken(tokens, .Slash),
            '%' => try self.addToken(tokens, .Mod),
            '!' => if (self.peek() == '=') {
                _ = self.advance();
                try self.addToken(tokens, .BangEqual);
            } else {
                try self.addToken(tokens, .Bang);
            },
            '&' => if (self.peek() == '&') {
                _ = self.advance();
                try self.addToken(tokens, .And);
            },
            '|' => if (self.peek() == '|') {
                _ = self.advance();
                try self.addToken(tokens, .Or);
            },
            '#' => if (self.peek() == ' ' and self.peekNext() == '=') {
                self.current += 2;
                try self.addToken(tokens, .HashEqual);
            } else {
                try self.comment(tokens);
            },
            '@' => try self.comment(tokens),
            '"' => try self.string(tokens),
            '0'...'9' => try self.number(tokens),
            'a'...'z', 'A'...'Z', '_' => try self.identifier(tokens),
            else => try self.errorAtCurrent("Unexpected character."),
        }
    }

    pub fn comment(self: *Lexer, tokens: *ArrayList(Token)) !void {
        if (self.peek() == '@') {
            self.current += 1;
            while (!self.isAtEnd() and !(self.peek() == '@' and self.peekNext() == '@')) {
                const ch = self.advance();
                if (ch == '\n') {
                    self.line += 1;
                    self.column = 1;
                }
            }
            if (self.isAtEnd()) return error.UnterminatedMultiComment;
            self.current += 2;
            try self.addToken(tokens, .MultiComment);
        } else {
            while (!self.isAtEnd() and self.peek() != '\n') _ = self.advance();
            try self.addToken(tokens, .Comment);
        }
    }

    pub fn string(self: *Lexer, tokens: *ArrayList(Token)) !void {
        while (!self.isAtEnd() and self.peek() != '"') {
            const ch = self.advance();
            if (ch == '\n') {
                self.line += 1;
                self.column = 1;
            }
        }
        if (self.isAtEnd()) return error.UnterminatedString;
        _ = self.advance(); // "
        try self.addToken(tokens, .String);
    }

    pub fn number(self: *Lexer, tokens: *ArrayList(Token)) !void {
        while (std.ascii.isDigit(self.peek())) _ = self.advance();
        if (self.peek() == '.' and std.ascii.isDigit(self.peekNext())) {
            _ = self.advance();
            while (std.ascii.isDigit(self.peek())) _ = self.advance();
            try self.addToken(tokens, .Float);
        } else {
            try self.addToken(tokens, .Number);
        }
    }

    pub fn identifier(self: *Lexer, tokens: *ArrayList(Token)) !void {
        while (std.ascii.isAlphanumeric(self.peek()) or self.peek() == '_') _ = self.advance();
        const text = self.source[self.start..self.current];
        const typ = keywords.get(text) orelse .Identifier;
        try self.addToken(tokens, typ);
    }

    pub fn errorAtCurrent(self: *Lexer, msg: []const u8) !void {
        std.debug.print("Error at line {d}, column {d}: {s}\n", .{self.line, self.column, msg});
        return error.ParseError;
    }

    // Dodano więcej funkcji, np. dla obsługi escape w stringach
    pub fn handleEscape(self: *Lexer) !void {
        // Stub dla \n itp., rozbuduj jeśli potrzeba
        _ = self;
    }
};

const keywords = std.StaticStringMap(TokenType).initComptime(.{
    .{ "write", .Write },
    .{ "let", .Let },
    .{ "func", .Func },
    .{ "if", .If },
    .{ "else", .Else },
    .{ "while", .While },
    .{ "for", .For },
    .{ "return", .Return },
    .{ "and", .And },
    .{ "or", .Or },
    .{ "true", .True },
    .{ "false", .False },
    .{ "int", .IntType },
    .{ "float", .FloatType },
    .{ "string", .StringType },
    .{ "bool", .BoolType },
    .{ "array", .ArrayType },
});
