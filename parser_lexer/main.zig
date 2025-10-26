const std = @import("std");
const Allocator = std.mem.Allocator;
const ArrayList = std.ArrayList;
const print = std.debug.print;

const TokenType = enum {
    Import,         // <>
    From,           // ::
    Write,          // write
    Comment,        // @
    MultiComment,   // @@ @@
    Let,            // let
    Func,           // func
    If,             // if
    Else,           // else
    While,          // while added
    For,            // for added
    Return,         // return
    Colon,          // :
    Arrow,          // ->
    LeftBracket,    // [
    RightBracket,   // ]
    LeftParen,      // (
    RightParen,     // )
    Comma,          // ,
    HashEqual,      // # ==
    Equals,         // =
    Plus, Minus, Star, Slash, Mod, // Mod added
    Less, Greater, LessEqual, GreaterEqual, EqualEqual, BangEqual,
    Bang,           // !
    And, Or,        // and or
    Identifier,
    Number, Float,  // Float added
    String,
    IntType, FloatType, StringType, BoolType, ArrayType, // added FloatType, ArrayType
    True, False,
    Eof,
};

const Token = struct {
    typ: TokenType,
    lexeme: []const u8,
    line: usize,
    column: usize,
};

const Lexer = struct {
    source: []const u8,
    start: usize = 0,
    current: usize = 0,
    line: usize = 1,
    column: usize = 1,
    allocator: Allocator,

    fn init(allocator: Allocator, source: []const u8) Lexer {
        return .{ .allocator = allocator, .source = source };
    }

    fn scanTokens(self: *Lexer) !ArrayList(Token) {
        var tokens = ArrayList(Token).init(self.allocator);
        errdefer tokens.deinit();

        while (!self.isAtEnd()) {
            self.start = self.current;
            try self.scanToken(&tokens);
        }
        try tokens.append(.{ .typ = .Eof, .lexeme = "", .line = self.line, .column = self.column });
        return tokens;
    }

    fn isAtEnd(self: *Lexer) bool {
        return self.current >= self.source.len;
    }

    fn advance(self: *Lexer) u8 {
        self.current += 1;
        self.column += 1;
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
        try tokens.append(.{ .typ = typ, .lexeme = lexeme, .line = self.line, .column = self.start - self.lineStart() + 1 });
    }

    fn lineStart(self: *Lexer) usize {
        var i = self.start;
        while (i > 0 and self.source[i - 1] != '\n') i -= 1;
        return i;
    }

    fn skipWhitespace(self: *Lexer) {
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

    fn scanToken(self: *Lexer, tokens: *ArrayList(Token)) !void {
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

    fn comment(self: *Lexer, tokens: *ArrayList(Token)) !void {
        if (self.peek() == '@') {
            // Multiline @@ .. @@
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
            // Single line @
            while (!self.isAtEnd() and self.peek() != '\n') _ = self.advance();
            try self.addToken(tokens, .Comment);
        }
    }

    fn string(self: *Lexer, tokens: *ArrayList(Token)) !void {
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

    fn number(self: *Lexer, tokens: *ArrayList(Token)) !void {
        while (std.ascii.isDigit(self.peek())) _ = self.advance();
        if (self.peek() == '.' and std.ascii.isDigit(self.peekNext())) {
            _ = self.advance();
            while (std.ascii.isDigit(self.peek())) _ = self.advance();
            try self.addToken(tokens, .Float);
        } else {
            try self.addToken(tokens, .Number);
        }
    }

    fn identifier(self: *Lexer, tokens: *ArrayList(Token)) !void {
        while (std.ascii.isAlphanumeric(self.peek()) or self.peek() == '_') _ = self.advance();
        const text = self.source[self.start..self.current];
        const typ = keywords.get(text) orelse .Identifier;
        try self.addToken(tokens, typ);
    }

    fn errorAtCurrent(self: *Lexer, msg: []const u8) !void {
        print("Error at line {d}, column {d}: {s}\n", .{self.line, self.column, msg});
        return error.ParseError;
    }
};

const keywords = std.ComptimeStringMap(TokenType, .{
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

// AST Nodes
const AstNode = union(enum) {
    Literal: struct { value: []const u8, typ: TokenType },
    Binary: struct { left: *AstNode, op: TokenType, right: *AstNode },
    Unary: struct { op: TokenType, right: *AstNode },
    VarDecl: struct { name: []const u8, typ: ?[]const u8, init: *AstNode },
    FuncDecl: struct { name: []const u8, params: ArrayList(Param), return_type: []const u8, body: *AstNode },
    IfStmt: struct { cond: *AstNode, then: *AstNode, else_: ?*AstNode },
    WhileStmt: struct { cond: *AstNode, body: *AstNode }, // added
    ForStmt: struct { init: *AstNode, cond: *AstNode, incr: *AstNode, body: *AstNode }, // added
    ReturnStmt: struct { expr: ?*AstNode },
    Block: ArrayList(*AstNode),
    Call: struct { callee: []const u8, args: ArrayList(*AstNode) },
    ArrayLiteral: ArrayList(*AstNode), // added
    Index: struct { arr: *AstNode, idx: *AstNode }, // added
};

const Param = struct { name: []const u8, typ: []const u8 };

const Parser = struct {
    tokens: ArrayList(Token),
    current: usize = 0,
    allocator: Allocator,

    fn init(allocator: Allocator, tokens: ArrayList(Token)) Parser {
        return .{ .allocator = allocator, .tokens = tokens };
    }

    fn deinit(self: *Parser) void {
        // Deinit AST if needed
    }

    fn parse(self: *Parser) !ArrayList(*AstNode) {
        var statements = ArrayList(*AstNode).init(self.allocator);
        errdefer statements.deinit();

        while (!self.isAtEnd()) {
            const stmt = try self.statement();
            try statements.append(stmt);
        }
        return statements;
    }

    fn statement(self: *Parser) !*AstNode {
        if (self.match(.Func)) return try self.funcDecl();
        if (self.match(.Let)) return try self.varDecl();
        if (self.match(.If)) return try self.ifStmt();
        if (self.match(.While)) return try self.whileStmt();
        if (self.match(.For)) return try self.forStmt();
        if (self.match(.Return)) return try self.returnStmt();
        if (self.match(.LeftBracket)) return try self.block();
        return try self.expressionStmt();
    }

    fn funcDecl(self: *Parser) !*AstNode {
        const name = try self.consume(.Identifier, "Expect function name.");
        _ = try self.consume(.LeftParen, "Expect '(' after name.");
        var params = ArrayList(Param).init(self.allocator);
        if (!self.check(.RightParen)) {
            while (true) {
                const param_name = try self.consume(.Identifier, "Expect param name.");
                _ = try self.consume(.Colon, "Expect ':' after param name.");
                const param_type = try self.consume(.Identifier, "Expect param type.");
                try params.append(.{ .name = param_name.lexeme, .typ = param_type.lexeme });
                if (!self.match(.Comma)) break;
            }
        }
        _ = try self.consume(.RightParen, "Expect ')' after params.");
        if (self.match(.Arrow)) {
            const return_type = try self.consume(.Identifier, "Expect return type.");
            const body = try self.statement();
            const node = try self.allocator.create(AstNode);
            node.* = .{ .FuncDecl = .{ .name = name.lexeme, .params = params, .return_type = return_type.lexeme, .body = body } };
            return node;
        } else {
            return error.MissingArrow;
        }
    }

    fn varDecl(self: *Parser) !*AstNode {
        const name = try self.consume(.Identifier, "Expect variable name.");
        var typ: ?[]const u8 = null;
        if (self.match(.Colon)) {
            typ = (try self.consume(.Identifier, "Expect type.")).lexeme;
        }
        _ = try self.consume(.Equals, "Expect '=' after variable.");
        const init = try self.expression();
        const node = try self.allocator.create(AstNode);
        node.* = .{ .VarDecl = .{ .name = name.lexeme, .typ = typ, .init = init } };
        return node;
    }

    fn ifStmt(self: *Parser) !*AstNode {
        const cond = try self.expression();
        const then = try self.statement();
        var else_: ?*AstNode = null;
        if (self.match(.Else)) {
            else_ = try self.statement();
        }
        const node = try self.allocator.create(AstNode);
        node.* = .{ .IfStmt = .{ .cond = cond, .then = then, .else_ = else_ } };
        return node;
    }

    fn whileStmt(self: *Parser) !*AstNode {
        const cond = try self.expression();
        const body = try self.statement();
        const node = try self.allocator.create(AstNode);
        node.* = .{ .WhileStmt = .{ .cond = cond, .body = body } };
        return node;
    }

    fn forStmt(self: *Parser) !*AstNode {
        const init = try self.statement();
        const cond = try self.expression();
        const incr = try self.expression();
        const body = try self.statement();
        const node = try self.allocator.create(AstNode);
        node.* = .{ .ForStmt = .{ .init = init, .cond = cond, .incr = incr, .body = body } };
        return node;
    }

    fn returnStmt(self: *Parser) !*AstNode {
        var expr: ?*AstNode = null;
        if (!self.check(.RightBracket)) {
            expr = try self.expression();
        }
        const node = try self.allocator.create(AstNode);
        node.* = .{ .ReturnStmt = .{ .expr = expr } };
        return node;
    }

    fn block(self: *Parser) !*AstNode {
        var statements = ArrayList(*AstNode).init(self.allocator);
        while (!self.check(.RightBracket) and !self.isAtEnd()) {
            try statements.append(try self.statement());
        }
        _ = try self.consume(.RightBracket, "Expect ']' after block.");
        const node = try self.allocator.create(AstNode);
        node.* = .{ .Block = statements };
        return node;
    }

    fn expressionStmt(self: *Parser) !*AstNode {
        return try self.expression();
    }

    fn expression(self: *Parser) !*AstNode {
        return try self.logicalOr();
    }

    fn logicalOr(self: *Parser) !*AstNode {
        var expr = try self.logicalAnd();
        while (self.match(.Or)) {
            const op = self.previous().typ;
            const right = try self.logicalAnd();
            const new_expr = try self.allocator.create(AstNode);
            new_expr.* = .{ .Binary = .{ .left = expr, .op = op, .right = right } };
            expr = new_expr;
        }
        return expr;
    }

    fn logicalAnd(self: *Parser) !*AstNode {
        var expr = try self.equality();
        while (self.match(.And)) {
            const op = self.previous().typ;
            const right = try self.equality();
            const new_expr = try self.allocator.create(AstNode);
            new_expr.* = .{ .Binary = .{ .left = expr, .op = op, .right = right } };
            expr = new_expr;
        }
        return expr;
    }

    fn equality(self: *Parser) !*AstNode {
        var expr = try self.comparison();
        while (self.match(.EqualEqual) or self.match(.BangEqual)) {
            const op = self.previous().typ;
            const right = try self.comparison();
            const new_expr = try self.allocator.create(AstNode);
            new_expr.* = .{ .Binary = .{ .left = expr, .op = op, .right = right } };
            expr = new_expr;
        }
        return expr;
    }

    fn comparison(self: *Parser) !*AstNode {
        var expr = try self.term();
        while (self.match(.Greater) or self.match(.GreaterEqual) or self.match(.Less) or self.match(.LessEqual)) {
            const op = self.previous().typ;
            const right = try self.term();
            const new_expr = try self.allocator.create(AstNode);
            new_expr.* = .{ .Binary = .{ .left = expr, .op = op, .right = right } };
            expr = new_expr;
        }
        return expr;
    }

    fn term(self: *Parser) !*AstNode {
        var expr = try self.factor();
        while (self.match(.Minus) or self.match(.Plus)) {
            const op = self.previous().typ;
            const right = try self.factor();
            const new_expr = try self.allocator.create(AstNode);
            new_expr.* = .{ .Binary = .{ .left = expr, .op = op, .right = right } };
            expr = new_expr;
        }
        return expr;
    }

    fn factor(self: *Parser) !*AstNode {
        var expr = try self.unary();
        while (self.match(.Slash) or self.match(.Star) or self.match(.Mod)) {
            const op = self.previous().typ;
            const right = try self.unary();
            const new_expr = try self.allocator.create(AstNode);
            new_expr.* = .{ .Binary = .{ .left = expr, .op = op, .right = right } };
            expr = new_expr;
        }
        return expr;
    }

    fn unary(self: *Parser) !*AstNode {
        if (self.match(.Bang) or self.match(.Minus)) {
            const op = self.previous().typ;
            const right = try self.unary();
            const node = try self.allocator.create(AstNode);
            node.* = .{ .Unary = .{ .op = op, .right = right } };
            return node;
        }
        return try self.call();
    }

    fn call(self: *Parser) !*AstNode {
        var expr = try self.primary();
        if (self.match(.LeftParen)) {
            var args = ArrayList(*AstNode).init(self.allocator);
            if (!self.check(.RightParen)) {
                while (true) {
                    try args.append(try self.expression());
                    if (!self.match(.Comma)) break;
                }
            }
            _ = try self.consume(.RightParen, "Expect ')' after arguments.");
            if (expr.* != .Literal or expr.Literal.typ != .Identifier) return error.InvalidCallee;
            const callee = expr.Literal.value;
            const node = try self.allocator.create(AstNode);
            node.* = .{ .Call = .{ .callee = callee, .args = args } };
            return node;
        }
        return expr;
    }

    fn primary(self: *Parser) !*AstNode {
        if (self.match(.False)) return try self.literal(.False);
        if (self.match(.True)) return try self.literal(.True);
        if (self.match(.Number)) return try self.literal(.Number);
        if (self.match(.Float)) return try self.literal(.Float);
        if (self.match(.String)) return try self.literal(.String);
        if (self.match(.Identifier)) return try self.literal(.Identifier);
        if (self.match(.LeftBracket)) {
            var elems = ArrayList(*AstNode).init(self.allocator);
            if (!self.check(.RightBracket)) {
                while (true) {
                    try elems.append(try self.expression());
                    if (!self.match(.Comma)) break;
                }
            }
            _ = try self.consume(.RightBracket, "Expect ']' after array.");
            const node = try self.allocator.create(AstNode);
            node.* = .{ .ArrayLiteral = elems };
            return node;
        }
        if (self.match(.LeftParen)) {
            const expr = try self.expression();
            _ = try self.consume(.RightParen, "Expect ')' after expression.");
            return expr;
        }
        return error.InvalidPrimary;
    }

    fn literal(self: *Parser, typ: TokenType) !*AstNode {
        const token = self.previous();
        const node = try self.allocator.create(AstNode);
        node.* = .{ .Literal = .{ .value = token.lexeme, .typ = typ } };
        return node;
    }

    fn match(self: *Parser, typ: TokenType) bool {
        if (self.check(typ)) {
            _ = self.advance();
            return true;
        }
        return false;
    }

    fn check(self: *Parser, typ: TokenType) bool {
        if (self.isAtEnd()) return false;
        return self.tokens.items[self.current].typ == typ;
    }

    fn advance(self: *Parser) Token {
        if (!self.isAtEnd()) self.current += 1;
        return self.previous();
    }

    fn previous(self: *Parser) Token {
        return self.tokens.items[self.current - 1];
    }

    fn isAtEnd(self: *Parser) bool {
        return self.tokens.items[self.current].typ == .Eof;
    }

    fn consume(self: *Parser, typ: TokenType, msg: []const u8) !Token {
        if (self.check(typ)) return self.advance();
        try self.errorAt(self.previous(), msg);
        return error.ParseError;
    }

    fn errorAt(self: *Parser, token: Token, msg: []const u8) !void {
        print("Error at line {d}, column {d}: {s}\n", .{token.line, token.column, msg});
        return error.ParseError;
    }
};

// Formatter stub
fn format(source: []const u8, allocator: Allocator) ![]u8 {
    _ = allocator;
    return try allocator.dupe(u8, source); // Stub, can be expanded
}

pub fn main() !void {
    const allocator = std.heap.page_allocator;
    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 3) {
        print("Usage: vira-parser_lexer <command> <file> [options]\nCommands: check, fmt\n", .{});
        return;
    }

    const command = args[1];
    const file_path = args[2];

    var file = try std.fs.cwd().openFile(file_path, .{});
    defer file.close();
    const source = try file.readToEndAlloc(allocator, 1024 * 1024);
    defer allocator.free(source);

    var lexer = Lexer.init(allocator, source);
    var tokens = try lexer.scanTokens();
    defer tokens.deinit();

    var parser = Parser.init(allocator, tokens);
    defer parser.deinit();

    _ = try parser.parse(); // Build AST

    if (std.mem.eql(u8, command, "check")) {
        print("Syntax check: OK\n", .{});
    } else if (std.mem.eql(u8, command, "fmt")) {
        const formatted = try format(source, allocator);
        defer allocator.free(formatted);
        try std.fs.cwd().writeFile(file_path, formatted);
        print("Formatted {s}\n", .{file_path});
    } else {
        print("Unknown command: {s}\n", .{command});
    }
}
