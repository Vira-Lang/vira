const std = @import("std");
const ArrayList = std.ArrayList;
const Allocator = std.mem.Allocator;
const TokenType = @import("lexer.zig").TokenType;
const Token = @import("lexer.zig").Token;

// AST Nodes
pub const AstNode = union(enum) {
    Literal: struct { value: []const u8, typ: TokenType },
    Binary: struct { left: *AstNode, op: TokenType, right: *AstNode },
    Unary: struct { op: TokenType, right: *AstNode },
    VarDecl: struct { name: []const u8, typ: ?[]const u8, init: *AstNode },
    FuncDecl: struct { name: []const u8, params: ArrayList(Param), return_type: []const u8, body: *AstNode },
    IfStmt: struct { cond: *AstNode, then: *AstNode, else_: ?*AstNode },
    WhileStmt: struct { cond: *AstNode, body: *AstNode },
    ForStmt: struct { init: *AstNode, cond: *AstNode, incr: *AstNode, body: *AstNode },
    ReturnStmt: struct { expr: ?*AstNode },
    Block: ArrayList(*AstNode),
    Call: struct { callee: []const u8, args: ArrayList(*AstNode) },
    ArrayLiteral: ArrayList(*AstNode),
    Index: struct { arr: *AstNode, idx: *AstNode },
    // Dodano więcej: np. Assign, Loop etc. jeśli potrzeba
};

pub const Param = struct { name: []const u8, typ: []const u8 };

pub const Error = error {
    OutOfMemory,
    ParseError,
    MissingArrow,
    InvalidCallee,
    InvalidPrimary,
};

pub const Parser = struct {
    tokens: ArrayList(Token),
    current: usize = 0,
    allocator: Allocator,

    pub fn init(allocator: Allocator, tokens: ArrayList(Token)) Parser {
        return .{ .allocator = allocator, .tokens = tokens };
    }

    pub fn deinit(self: *Parser) void {
        // Deinit AST recursively if needed, stub
        _ = self;
    }

    pub fn parse(self: *Parser) Error!ArrayList(*AstNode) {
        var statements = ArrayList(*AstNode).init(self.allocator);
        errdefer statements.deinit();

        while (!self.isAtEnd()) {
            const stmt = try self.statement();
            try statements.append(stmt);
        }
        return statements;
    }

    pub fn statement(self: *Parser) Error!*AstNode {
        if (self.match(.Func)) return try self.funcDecl();
        if (self.match(.Let)) return try self.varDecl();
        if (self.match(.If)) return try self.ifStmt();
        if (self.match(.While)) return try self.whileStmt();
        if (self.match(.For)) return try self.forStmt();
        if (self.match(.Return)) return try self.returnStmt();
        if (self.match(.LeftBracket)) return try self.block();
        return try self.expressionStmt();
    }

    pub fn funcDecl(self: *Parser) Error!*AstNode {
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
            return Error.MissingArrow;
        }
    }

    pub fn varDecl(self: *Parser) Error!*AstNode {
        const name = try self.consume(.Identifier, "Expect variable name.");
        var typ: ?[]const u8 = null;
        if (self.match(.Colon)) {
            typ = (try self.consume(.Identifier, "Expect type.")).lexeme;
        }
        _ = try self.consume(.Equals, "Expect '=' after variable.");
        const initializer = try self.expression();
        const node = try self.allocator.create(AstNode);
        node.* = .{ .VarDecl = .{ .name = name.lexeme, .typ = typ, .init = initializer } };
        return node;
    }

    pub fn ifStmt(self: *Parser) Error!*AstNode {
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

    pub fn whileStmt(self: *Parser) Error!*AstNode {
        const cond = try self.expression();
        const body = try self.statement();
        const node = try self.allocator.create(AstNode);
        node.* = .{ .WhileStmt = .{ .cond = cond, .body = body } };
        return node;
    }

    pub fn forStmt(self: *Parser) Error!*AstNode {
        const for_init = try self.statement();
        const cond = try self.expression();
        const incr = try self.expression();
        const body = try self.statement();
        const node = try self.allocator.create(AstNode);
        node.* = .{ .ForStmt = .{ .init = for_init, .cond = cond, .incr = incr, .body = body } };
        return node;
    }

    pub fn returnStmt(self: *Parser) Error!*AstNode {
        var expr: ?*AstNode = null;
        if (!self.check(.RightBracket)) {
            expr = try self.expression();
        }
        const node = try self.allocator.create(AstNode);
        node.* = .{ .ReturnStmt = .{ .expr = expr } };
        return node;
    }

    pub fn block(self: *Parser) Error!*AstNode {
        var statements = ArrayList(*AstNode).init(self.allocator);
        while (!self.check(.RightBracket) and !self.isAtEnd()) {
            try statements.append(try self.statement());
        }
        _ = try self.consume(.RightBracket, "Expect ']' after block.");
        const node = try self.allocator.create(AstNode);
        node.* = .{ .Block = statements };
        return node;
    }

    pub fn expressionStmt(self: *Parser) Error!*AstNode {
        return try self.expression();
    }

    pub fn expression(self: *Parser) Error!*AstNode {
        return try self.logicalOr();
    }

    pub fn logicalOr(self: *Parser) Error!*AstNode {
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

    pub fn logicalAnd(self: *Parser) Error!*AstNode {
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

    pub fn equality(self: *Parser) Error!*AstNode {
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

    pub fn comparison(self: *Parser) Error!*AstNode {
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

    pub fn term(self: *Parser) Error!*AstNode {
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

    pub fn factor(self: *Parser) Error!*AstNode {
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

    pub fn unary(self: *Parser) Error!*AstNode {
        if (self.match(.Bang) or self.match(.Minus)) {
            const op = self.previous().typ;
            const right = try self.unary();
            const node = try self.allocator.create(AstNode);
            node.* = .{ .Unary = .{ .op = op, .right = right } };
            return node;
        }
        return try self.call();
    }

    pub fn call(self: *Parser) Error!*AstNode {
        const expr = try self.primary();
        if (self.match(.LeftParen)) {
            var args = ArrayList(*AstNode).init(self.allocator);
            if (!self.check(.RightParen)) {
                while (true) {
                    try args.append(try self.expression());
                    if (!self.match(.Comma)) break;
                }
            }
            _ = try self.consume(.RightParen, "Expect ')' after arguments.");
            if (expr.* != .Literal or expr.Literal.typ != .Identifier) return Error.InvalidCallee;
            const callee = expr.Literal.value;
            const node = try self.allocator.create(AstNode);
            node.* = .{ .Call = .{ .callee = callee, .args = args } };
            return node;
        }
        return expr;
    }

    pub fn primary(self: *Parser) Error!*AstNode {
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
        return Error.InvalidPrimary;
    }

    pub fn literal(self: *Parser, typ: TokenType) Error!*AstNode {
        const token = self.previous();
        const node = try self.allocator.create(AstNode);
        node.* = .{ .Literal = .{ .value = token.lexeme, .typ = typ } };
        return node;
    }

    pub fn match(self: *Parser, typ: TokenType) bool {
        if (self.check(typ)) {
            _ = self.advance();
            return true;
        }
        return false;
    }

    pub fn check(self: *Parser, typ: TokenType) bool {
        if (self.isAtEnd()) return false;
        return self.tokens.items[self.current].typ == typ;
    }

    pub fn advance(self: *Parser) Token {
        if (!self.isAtEnd()) self.current += 1;
        return self.previous();
    }

    pub fn previous(self: *Parser) Token {
        return self.tokens.items[self.current - 1];
    }

    pub fn isAtEnd(self: *Parser) bool {
        return self.tokens.items[self.current].typ == .Eof;
    }

    pub fn consume(self: *Parser, typ: TokenType, msg: []const u8) Error!Token {
        if (self.check(typ)) return self.advance();
        try self.errorAt(self.previous(), msg);
        return Error.ParseError;
    }

    pub fn errorAt(self: *Parser, token: Token, msg: []const u8) Error!void {
        std.debug.print("Error at line {d}, column {d}: {s}\n", .{token.line, token.column, msg});
        _ = self;
        return Error.ParseError;
    }

    // Dodano więcej funkcji, np. synchronize po error
    pub fn synchronize(self: *Parser) void {
        while (!self.isAtEnd()) {
            switch (self.peek().typ) {
                .Func, .Let, .If, .While, .For, .Return => return,
                else => _ = self.advance(),
            }
        }
    }

    pub fn peek(self: *Parser) Token {
        return self.tokens.items[self.current];
    }
};
