use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

#[derive(Debug, Clone)]
enum ViraType {
    Int,
    Float,
    Bool,
    String,
    Array(Box<ViraType>),
    // Dodano: Float, Bool, Array dla nowoczesności
}

#[derive(Debug, Clone)]
struct Variable {
    name: String,
    typ: ViraType,
    // For memory management, track regions
}

#[derive(Debug)]
enum AstNode {
    Literal(i64),
    FloatLiteral(f64), // Dodano
    BoolLiteral(bool), // Dodano
    StringLiteral(String),
    Binary(Box<AstNode>, BinOp, Box<AstNode>),
    Unary(UnaryOp, Box<AstNode>), // Dodano unary
    VarDecl(String, ViraType, Box<AstNode>),
    VarRef(String),
    FuncDecl(String, Vec<(String, ViraType)>, ViraType, Box<AstNode>),
    Call(String, Vec<AstNode>),
    If(Box<AstNode>, Box<AstNode>, Option<Box<AstNode>>),
    While(Box<AstNode>, Box<AstNode>), // Dodano loop while
    For(String, Box<AstNode>, Box<AstNode>, Box<AstNode>, Box<AstNode>), // Dodano for (init, cond, incr, body)
    Return(Option<Box<AstNode>>),
    Block(Vec<AstNode>),
    Write(Box<AstNode>),
    ArrayLiteral(Vec<AstNode>), // Dodano arrays
    Index(Box<AstNode>, Box<AstNode>), // Dodano indexing
    // Dodano więcej dla nowoczesności
}

#[derive(Debug)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div, // Dodano
    Mod, // Dodano
    Eq,  // Dodano comparisons
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And, // Logical
    Or,
}

#[derive(Debug)]
enum UnaryOp {
    Neg,
    Not,
    // Dodano
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, Clone)]
struct Token {
    typ: TokenType,
    lexeme: String,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    Func,
    Let,
    If,
    Else, // Dodano
    While, // Dodano
    For,   // Dodano
    Return,
    Write,
    Identifier,
    Number,
    Float, // Dodano
    String,
    True,  // Dodano
    False, // Dodano
    Plus,
    Minus,
    Star,
    Slash, // Dodano
    Mod,   // Dodano
    Bang,  // Dodano !
    And,   // Dodano &&
    Or,    // Dodano ||
    EqualEqual, // Dodano ==
    BangEqual,  // Dodano !=
    Less,       // Dodano <
    Greater,    // Dodano >
    LessEqual,
    GreaterEqual,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Colon,
    Arrow,
    Equals,
    Comma, // Dodano ,
    LeftBrace, // Dodano { dla alternatywnych bloków
    RightBrace, // }
    Eof,
    // Rozbudowano o więcej tokenów dla nowoczesnego języka
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn parse(&mut self) -> Result<Vec<AstNode>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<AstNode, String> {
        if self.match_token(TokenType::Func) {
            self.func_decl()
        } else if self.match_token(TokenType::Let) {
            self.var_decl()
        } else if self.match_token(TokenType::If) {
            self.if_stmt()
        } else if self.match_token(TokenType::While) {
            self.while_stmt() // Dodano
        } else if self.match_token(TokenType::For) {
            self.for_stmt() // Dodano
        } else if self.match_token(TokenType::Return) {
            self.return_stmt()
        } else if self.match_token(TokenType::Write) {
            self.write_stmt()
        } else if self.match_token(TokenType::LeftBracket) || self.match_token(TokenType::LeftBrace) {
            self.block()
        } else {
            self.expression_stmt()
        }
    }

    fn func_decl(&mut self) -> Result<AstNode, String> {
        let name = self.consume(TokenType::Identifier, "Expect function name.")?.lexeme;
        self.consume(TokenType::LeftParen, "Expect '(' after name.")?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = self.consume(TokenType::Identifier, "Expect param name.")?.lexeme;
                self.consume(TokenType::Colon, "Expect ':' after param name.")?;
                let param_type = self.parse_type()?;
                params.push((param_name, param_type));
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after params.")?;
        if self.match_token(TokenType::Arrow) {
            let return_type = self.parse_type()?;
            let body = self.statement()?;
            Ok(AstNode::FuncDecl(name, params, return_type, Box::new(body)))
        } else {
            Err("Missing '->' in function declaration.".to_string())
        }
    }

    fn var_decl(&mut self) -> Result<AstNode, String> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?.lexeme;
        let mut typ = ViraType::Int; // Default
        if self.match_token(TokenType::Colon) {
            typ = self.parse_type()?;
        }
        self.consume(TokenType::Equals, "Expect '=' after variable.")?;
        let init = self.expression()?;
        Ok(AstNode::VarDecl(name, typ, Box::new(init)))
    }

    fn if_stmt(&mut self) -> Result<AstNode, String> {
        let cond = self.expression()?;
        let then = self.statement()?;
        let else_branch = if self.match_token(TokenType::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(AstNode::If(Box::new(cond), Box::new(then), else_branch))
    }

    fn while_stmt(&mut self) -> Result<AstNode, String> {
        let cond = self.expression()?;
        let body = self.statement()?;
        Ok(AstNode::While(Box::new(cond), Box::new(body)))
    }

    fn for_stmt(&mut self) -> Result<AstNode, String> {
        let init = self.statement()?;
        let cond = self.expression()?;
        let incr = self.expression()?;
        let body = self.statement()?;
        Ok(AstNode::For("".to_string(), Box::new(init), Box::new(cond), Box::new(incr), Box::new(body))) // Uproszczono, dostosować
    }

    fn return_stmt(&mut self) -> Result<AstNode, String> {
        let expr = if !self.check(TokenType::RightBracket) && !self.check(TokenType::RightBrace) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        Ok(AstNode::Return(expr))
    }

    fn write_stmt(&mut self) -> Result<AstNode, String> {
        let expr = self.expression()?;
        Ok(AstNode::Write(Box::new(expr)))
    }

    fn block(&mut self) -> Result<AstNode, String> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBracket) && !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }
        if self.check(TokenType::RightBracket) {
            self.consume(TokenType::RightBracket, "Expect ']' after block.")?;
        } else {
            self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        }
        Ok(AstNode::Block(statements))
    }

    fn expression_stmt(&mut self) -> Result<AstNode, String> {
        self.expression()
    }

    fn expression(&mut self) -> Result<AstNode, String> {
        self.logical_or()
    }

    fn logical_or(&mut self) -> Result<AstNode, String> {
        let mut expr = self.logical_and()?;
        while self.match_token(TokenType::Or) {
            let right = self.logical_and()?;
            expr = AstNode::Binary(Box::new(expr), BinOp::Or, Box::new(right));
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<AstNode, String> {
        let mut expr = self.equality()?;
        while self.match_token(TokenType::And) {
            let right = self.equality()?;
            expr = AstNode::Binary(Box::new(expr), BinOp::And, Box::new(right));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<AstNode, String> {
        let mut expr = self.comparison()?;
        while self.match_token(TokenType::EqualEqual) || self.match_token(TokenType::BangEqual) {
            let op = if self.previous().typ == TokenType::EqualEqual { BinOp::Eq } else { BinOp::Neq };
            let right = self.comparison()?;
            expr = AstNode::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<AstNode, String> {
        let mut expr = self.term()?;
        while self.match_token(TokenType::Less) || self.match_token(TokenType::Greater) || self.match_token(TokenType::LessEqual) || self.match_token(TokenType::GreaterEqual) {
            let op = match self.previous().typ {
                TokenType::Less => BinOp::Lt,
                TokenType::Greater => BinOp::Gt,
                TokenType::LessEqual => BinOp::Le,
                TokenType::GreaterEqual => BinOp::Ge,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = AstNode::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<AstNode, String> {
        let mut expr = self.factor()?;
        while self.match_token(TokenType::Minus) || self.match_token(TokenType::Plus) {
            let op = if self.previous().typ == TokenType::Plus { BinOp::Add } else { BinOp::Sub };
            let right = self.factor()?;
            expr = AstNode::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<AstNode, String> {
        let mut expr = self.unary()?;
        while self.match_token(TokenType::Star) || self.match_token(TokenType::Slash) || self.match_token(TokenType::Mod) {
            let op = match self.previous().typ {
                TokenType::Star => BinOp::Mul,
                TokenType::Slash => BinOp::Div,
                TokenType::Mod => BinOp::Mod,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = AstNode::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<AstNode, String> {
        if self.match_token(TokenType::Minus) || self.match_token(TokenType::Bang) {
            let op = if self.previous().typ == TokenType::Minus { UnaryOp::Neg } else { UnaryOp::Not };
            let right = self.unary()?;
            Ok(AstNode::Unary(op, Box::new(right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<AstNode, String> {
        if self.match_token(TokenType::Number) {
            let value = self.previous().lexeme.parse::<i64>().map_err(|_| "Invalid number.")?;
            Ok(AstNode::Literal(value))
        } else if self.match_token(TokenType::Float) {
            let value = self.previous().lexeme.parse::<f64>().map_err(|_| "Invalid float.")?;
            Ok(AstNode::FloatLiteral(value))
        } else if self.match_token(TokenType::True) {
            Ok(AstNode::BoolLiteral(true))
        } else if self.match_token(TokenType::False) {
            Ok(AstNode::BoolLiteral(false))
        } else if self.match_token(TokenType::String) {
            Ok(AstNode::StringLiteral(self.previous().lexeme))
        } else if self.match_token(TokenType::Identifier) {
            let name = self.previous().lexeme;
            if self.match_token(TokenType::LeftParen) {
                let mut args = Vec::new();
                if !self.check(TokenType::RightParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
                Ok(AstNode::Call(name, args))
            } else {
                Ok(AstNode::VarRef(name))
            }
        } else if self.match_token(TokenType::LeftBracket) {
            let mut elements = Vec::new();
            if !self.check(TokenType::RightBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                }
            }
            self.consume(TokenType::RightBracket, "Expect ']' after array.")?;
            Ok(AstNode::ArrayLiteral(elements))
        } else if self.match_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            Ok(expr)
        } else {
            Err(format!("Unexpected token: {:?}", self.peek()))
        }
    }

    fn parse_type(&mut self) -> Result<ViraType, String> {
        let typ = self.consume(TokenType::Identifier, "Expect type.")?.lexeme;
        match typ.as_str() {
            "int" => Ok(ViraType::Int),
            "float" => Ok(ViraType::Float),
            "bool" => Ok(ViraType::Bool),
            "string" => Ok(ViraType::String),
            "array" => {
                self.consume(TokenType::Less, "Expect '<' for array type.")?;
                let inner = self.parse_type()?;
                self.consume(TokenType::Greater, "Expect '>' for array type.")?;
                Ok(ViraType::Array(Box::new(inner)))
            }
            _ => Err("Unknown type.".to_string()),
        }
    }

    // Reszta metod jak consume, match_token, etc. bez zmian, ale dodano obsługę nowych tokenów
    fn consume(&mut self, typ: TokenType, msg: &str) -> Result<Token, String> {
        if self.check(typ) {
            Ok(self.advance())
        } else {
            Err(msg.to_string())
        }
    }

    fn match_token(&mut self, typ: TokenType) -> bool {
        if self.check(typ) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, typ: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().typ == typ
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().typ == TokenType::Eof
    }
}

// Rozbudowany tokenizer
fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut iter = source.chars().peekable();
    let mut line = 1;
    while let Some(&c) = iter.peek() {
        match c {
            'f' if source.starts_with("func") => { tokens.push(Token { typ: TokenType::Func, lexeme: "func".to_string() }); iter.take(4); },
            'l' if source.starts_with("let") => { tokens.push(Token { typ: TokenType::Let, lexeme: "let".to_string() }); iter.take(3); },
            // Dodaj więcej keywordów
            'w' if source.starts_with("while") => { tokens.push(Token { typ: TokenType::While, lexeme: "while".to_string() }); iter.take(5); },
            'f' if source.starts_with("for") => { tokens.push(Token { typ: TokenType::For, lexeme: "for".to_string() }); iter.take(3); },
            'e' if source.starts_with("else") => { tokens.push(Token { typ: TokenType::Else, lexeme: "else".to_string() }); iter.take(4); },
            't' if source.starts_with("true") => { tokens.push(Token { typ: TokenType::True, lexeme: "true".to_string() }); iter.take(4); },
            'f' if source.starts_with("false") => { tokens.push(Token { typ: TokenType::False, lexeme: "false".to_string() }); iter.take(5); },
            // ... inne
            '+' => tokens.push(Token { typ: TokenType::Plus, lexeme: "+".to_string() }),
            '-' => {
                iter.next();
                if iter.peek() == Some(&'>') {
                    iter.next();
                    tokens.push(Token { typ: TokenType::Arrow, lexeme: "->".to_string() });
                } else {
                    tokens.push(Token { typ: TokenType::Minus, lexeme: "-".to_string() });
                }
            },
            // Dodaj więcej: / % == != < > <= >= ! && ||
            '/' => tokens.push(Token { typ: TokenType::Slash, lexeme: "/".to_string() }),
            '%' => tokens.push(Token { typ: TokenType::Mod, lexeme: "%".to_string() }),
            '=' if iter.peek() == Some(&'=') => { iter.next(); tokens.push(Token { typ: TokenType::EqualEqual, lexeme: "==".to_string() }); },
            '!' if iter.peek() == Some(&'=') => { iter.next(); tokens.push(Token { typ: TokenType::BangEqual, lexeme: "!=".to_string() }); },
            '<' if iter.peek() == Some(&'=') => { iter.next(); tokens.push(Token { typ: TokenType::LessEqual, lexeme: "<=".to_string() }); },
            '>' if iter.peek() == Some(&'=') => { iter.next(); tokens.push(Token { typ: TokenType::GreaterEqual, lexeme: ">=".to_string() }); },
            '<' => tokens.push(Token { typ: TokenType::Less, lexeme: "<".to_string() }),
            '>' => tokens.push(Token { typ: TokenType::Greater, lexeme: ">".to_string() }),
            '!' => tokens.push(Token { typ: TokenType::Bang, lexeme: "!".to_string() }),
            '&' if iter.peek() == Some(&'&') => { iter.next(); tokens.push(Token { typ: TokenType::And, lexeme: "&&".to_string() }); },
            '|' if iter.peek() == Some(&'|') => { iter.next(); tokens.push(Token { typ: TokenType::Or, lexeme: "||".to_string() }); },
            '{' => tokens.push(Token { typ: TokenType::LeftBrace, lexeme: "{".to_string() }),
            '}' => tokens.push(Token { typ: TokenType::RightBrace, lexeme: "}".to_string() }),
            // ... reszta jak w oryginale, ale rozbudowana o skip whitespace, comments itp.
            '\n' => line += 1,
            _ => {}, // Pomijaj lub error
        }
        iter.next();
    }
    tokens.push(Token { typ: TokenType::Eof, lexeme: "".to_string() });
    tokens
}

// Arena bez zmian

// Interpreter rozbudowany
#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
}

struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, AstNode>,
    arena: Arena,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
            functions: HashMap::new(),
            arena: Arena::new(),
        }
    }

    fn interpret(&mut self, ast: &[AstNode]) -> Result<(), String> {
        for node in ast {
            self.execute(node)?;
        }
        Ok(())
    }

    fn execute(&mut self, node: &AstNode) -> Result<Value, String> {
        match node {
            AstNode::Literal(val) => Ok(Value::Int(*val)),
            AstNode::FloatLiteral(val) => Ok(Value::Float(*val)),
            AstNode::BoolLiteral(val) => Ok(Value::Bool(*val)),
            AstNode::StringLiteral(s) => Ok(Value::String(s.clone())),
            AstNode::Binary(left, op, right) => {
                let l = self.execute(left)?;
                let r = self.execute(right)?;
                match (l, r, op) {
                    (Value::Int(a), Value::Int(b), BinOp::Add) => Ok(Value::Int(a + b)),
                    (Value::Int(a), Value::Int(b), BinOp::Sub) => Ok(Value::Int(a - b)),
                    (Value::Int(a), Value::Int(b), BinOp::Mul) => Ok(Value::Int(a * b)),
                    (Value::Int(a), Value::Int(b), BinOp::Div) => Ok(Value::Int(a / b)),
                    (Value::Int(a), Value::Int(b), BinOp::Mod) => Ok(Value::Int(a % b)),
                    (Value::Bool(a), Value::Bool(b), BinOp::And) => Ok(Value::Bool(a && b)),
                    (Value::Bool(a), Value::Bool(b), BinOp::Or) => Ok(Value::Bool(a || b)),
                    // Dodaj więcej kombinacji, np. dla float, comparisons
                    _ => Err("Type mismatch".to_string()),
                }
            }
            AstNode::Unary(op, right) => {
                let r = self.execute(right)?;
                match (op, r) {
                    (UnaryOp::Neg, Value::Int(v)) => Ok(Value::Int(-v)),
                    (UnaryOp::Neg, Value::Float(v)) => Ok(Value::Float(-v)),
                    (UnaryOp::Not, Value::Bool(v)) => Ok(Value::Bool(!v)),
                    _ => Err("Invalid unary".to_string()),
                }
            }
            AstNode::VarDecl(name, _, init) => {
                let value = self.execute(init)?;
                self.variables.insert(name.clone(), value);
                Ok(Value::Int(0))
            }
            AstNode::VarRef(name) => self.variables.get(name).cloned().ok_or("Undefined var".to_string()),
            AstNode::FuncDecl(name, _, _, body) => {
                self.functions.insert(name.clone(), *body.clone());
                Ok(Value::Int(0))
            }
            AstNode::Call(name, args) => {
                let func = self.functions.get(name).ok_or("Undefined func")?;
                // Locals, params - rozbuduj
                self.execute(func)
            }
            AstNode::If(cond, then, else_) => {
                if let Value::Bool(true) = self.execute(cond)? {
                    self.execute(then)
                } else if let Some(e) = else_ {
                    self.execute(e)
                } else {
                    Ok(Value::Int(0))
                }
            }
            AstNode::While(cond, body) => {
                while let Value::Bool(true) = self.execute(cond)? {
                    self.execute(body)?;
                }
                Ok(Value::Int(0))
            }
            AstNode::For(_, init, cond, incr, body) => {
                self.execute(init)?;
                while let Value::Bool(true) = self.execute(cond)? {
                    self.execute(body)?;
                    self.execute(incr)?;
                }
                Ok(Value::Int(0))
            }
            AstNode::Return(expr) => {
                if let Some(e) = expr {
                    self.execute(e)
                } else {
                    Ok(Value::Int(0))
                }
            }
            AstNode::Block(stmts) => {
                let mut result = Value::Int(0);
                for stmt in stmts {
                    result = self.execute(stmt)?;
                }
                Ok(result)
            }
            AstNode::Write(expr) => {
                let value = self.execute(expr)?;
                println!("{:?}", value);
                Ok(Value::Int(0))
            }
            AstNode::ArrayLiteral(elems) => {
                let mut arr = Vec::new();
                for elem in elems {
                    arr.push(self.execute(elem)?);
                }
                Ok(Value::Array(arr))
            }
            AstNode::Index(arr, idx) => {
                if let Value::Array(a) = self.execute(arr)? {
                    if let Value::Int(i) = self.execute(idx)? {
                        a.get(i as usize).cloned().ok_or("Index out of bounds".to_string())
                    } else {
                        Err("Invalid index".to_string())
                    }
                } else {
                    Err("Not an array".to_string())
                }
            }
        }
    }
}

// CodeGen rozbudowany - dodaj obsługę nowych node'ów
struct CodeGen {
    builder_context: FunctionBuilderContext,
    ctx: CodegenContext,
    module: JITModule,
}

struct CodegenContext {
    vars: HashMap<String, VariableId>,
}

impl CodeGen {
    fn new() -> Self {
        // Jak w oryginale
        // ...
    }

    fn compile(&mut self, ast: &[AstNode]) -> Result<*const u8, String> {
        // Jak w oryginale, ale dodaj codegen dla nowych
    }

    fn codegen_node(&mut self, builder: &mut FunctionBuilder, node: &AstNode) -> Result<Value, String> {
        match node {
            AstNode::Literal(val) => Ok(builder.ins().iconst(types::I64, *val)),
            AstNode::FloatLiteral(val) => Ok(builder.ins().fconst(types::F64, *val)),
            AstNode::BoolLiteral(val) => Ok(builder.ins().iconst(types::I8, if *val {1} else {0})),
            // Dodaj binary, unary, loops itd. - to jest placeholder dla rozbudowy
            _ => Err("Unsupported".to_string()),
        }
    }
}

// Reszta funkcji jak compile_to_object, run_file, main - bez dużych zmian, ale dodaj obsługę nowych komend jeśli potrzeba

fn main() -> io::Result<()> {
    // Jak w oryginale, ale dodaj więcej komend np. "format", "check"
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: vira-compiler <command> [args]");
        println!("Commands: compile <dir> --platform <plat> --output <out>, run <file>, repl, test <dir>, eval <code>, check <file>, fmt <file>");
        return Ok(());
    }

    // ... obsługa nowych
}
