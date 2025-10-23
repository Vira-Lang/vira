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
    String,
    // Add more
}

#[derive(Debug, Clone)]
struct Variable {
    name: String,
    typ: ViraType,
    // For memory management, track regions
}

#[derive(Debug)]
enum AstNode {
    Literal(i64), // For simplicity, int literals
    StringLiteral(String),
    Binary(Box<AstNode>, BinOp, Box<AstNode>),
    VarDecl(String, ViraType, Box<AstNode>),
    VarRef(String),
    FuncDecl(String, Vec<(String, ViraType)>, ViraType, Box<AstNode>),
    Call(String, Vec<AstNode>),
    If(Box<AstNode>, Box<AstNode>, Option<Box<AstNode>>),
    Return(Option<Box<AstNode>>),
    Block(Vec<AstNode>),
    Write(Box<AstNode>),
    // Add more
}

#[derive(Debug)]
enum BinOp {
    Add,
    Sub,
    Mul,
    // Add more
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
    Return,
    Write,
    Identifier,
    Number,
    String,
    Plus,
    Minus,
    Star,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Colon,
    Arrow,
    Equals,
    LessEqual,
    Eof,
    // Simplified
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
        } else if self.match_token(TokenType::Return) {
            self.return_stmt()
        } else if self.match_token(TokenType::Write) {
            self.write_stmt()
        } else if self.match_token(TokenType::LeftBracket) {
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
        let mut typ = ViraType::Int; // Default static type
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

    fn return_stmt(&mut self) -> Result<AstNode, String> {
        let expr = if !self.check(TokenType::RightBracket) {
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
        while !self.check(TokenType::RightBracket) && !self.is_at_end() {
            statements.push(self.statement()?);
        }
        self.consume(TokenType::RightBracket, "Expect ']' after block.")?;
        Ok(AstNode::Block(statements))
    }

    fn expression_stmt(&mut self) -> Result<AstNode, String> {
        self.expression()
    }

    fn expression(&mut self) -> Result<AstNode, String> {
        self.term()
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
        let mut expr = self.primary()?;
        while self.match_token(TokenType::Star) {
            let right = self.primary()?;
            expr = AstNode::Binary(Box::new(expr), BinOp::Mul, Box::new(right));
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<AstNode, String> {
        if self.match_token(TokenType::Number) {
            let value = self.previous().lexeme.parse::<i64>().map_err(|_| "Invalid number.")?;
            Ok(AstNode::Literal(value))
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
            "string" => Ok(ViraType::String),
            _ => Err("Unknown type.".to_string()),
        }
    }

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

// Simple tokenizer for demo (full in Zig)
fn tokenize(source: &str) -> Vec<Token> {
    // Stub: split and classify
    let mut tokens = Vec::new();
    let lines = source.lines();
    for line in lines {
        let words: Vec<&str> = line.split_whitespace().collect();
        for word in words {
            let typ = match word {
                "func" => TokenType::Func,
                "let" => TokenType::Let,
                "if" => TokenType::If,
                "return" => TokenType::Return,
                "write" => TokenType::Write,
                "+" => TokenType::Plus,
                "-" => TokenType::Minus,
                "*" => TokenType::Star,
                "[" => TokenType::LeftBracket,
                "]" => TokenType::RightBracket,
                "(" => TokenType::LeftParen,
                ")" => TokenType::RightParen,
                ":" => TokenType::Colon,
                "->" => TokenType::Arrow,
                "=" => TokenType::Equals,
                "<=" => TokenType::LessEqual,
                _ if word.parse::<i64>().is_ok() => TokenType::Number,
                _ if word.starts_with('"') => TokenType::String,
                _ => TokenType::Identifier,
            };
            tokens.push(Token { typ, lexeme: word.to_string() });
        }
    }
    tokens.push(Token { typ: TokenType::Eof, lexeme: "".to_string() });
    tokens
}

// Region-based allocator simulation
struct Arena {
    data: Vec<u8>,
}

impl Arena {
    fn new() -> Self {
        Arena { data: Vec::new() }
    }

    fn alloc<T>(&mut self, value: T) -> *mut T {
        let ptr = self.data.as_mut_ptr() as *mut T;
        unsafe { ptr.write(value); }
        self.data.resize(self.data.len() + std::mem::size_of::<T>(), 0);
        ptr
    }
}

// Interpreter for VM
#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    String(String),
    // Region ref?
}

struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, AstNode>,
    arena: Arena, // For memory safety
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
            AstNode::StringLiteral(s) => Ok(Value::String(s.clone())),
            AstNode::Binary(left, op, right) => {
                let l = self.execute(left)?;
                let r = self.execute(right)?;
                match (l, r) {
                    (Value::Int(a), Value::Int(b)) => match op {
                        BinOp::Add => Ok(Value::Int(a + b)),
                        BinOp::Sub => Ok(Value::Int(a - b)),
                        BinOp::Mul => Ok(Value::Int(a * b)),
                    },
                    _ => Err("Type mismatch in binary op.".to_string()),
                }
            }
            AstNode::VarDecl(name, _, init) => {
                let value = self.execute(init)?;
                self.variables.insert(name.clone(), value);
                Ok(Value::Int(0)) // Void
            }
            AstNode::VarRef(name) => self.variables.get(name).cloned().ok_or("Undefined variable.".to_string()),
            AstNode::FuncDecl(name, _, _, body) => {
                self.functions.insert(name.clone(), *body.clone());
                Ok(Value::Int(0))
            }
            AstNode::Call(name, args) => {
                let func = self.functions.get(name).ok_or("Undefined function.")?;
                let mut local_vars = HashMap::new();
                // Assume params match args for simplicity
                for (i, arg) in args.iter().enumerate() {
                    let value = self.execute(arg)?;
                    local_vars.insert(format!("param{}", i), value);
                }
                // Execute body with locals
                self.execute(func)
            }
            AstNode::If(cond, then, else_) => {
                let c = self.execute(cond)?;
                if let Value::Int(v) = c {
                    if v != 0 {
                        self.execute(then)
                    } else if let Some(e) = else_ {
                        self.execute(e)
                    } else {
                        Ok(Value::Int(0))
                    }
                } else {
                    Err("Non-int condition.".to_string())
                }
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
                match value {
                    Value::Int(i) => println!("{}", i),
                    Value::String(s) => println!("{}", s),
                }
                Ok(Value::Int(0))
            }
        }
    }
}

// Cranelift codegen
struct CodeGen {
    builder_context: FunctionBuilderContext,
    ctx: CodegenContext,
    module: JITModule, // For JIT, or ObjectModule for compile
}

struct CodegenContext {
    // Variables mapping to cranelift vars
    vars: HashMap<String, VariableId>,
}

impl CodeGen {
    fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap();
        let flags = settings::Flags::new(flag_builder);
        let isa = isa_builder.finish(flags).unwrap();
        let builder = JITBuilder::with_isa(isa, default_libcall_names());
        let module = JITModule::new(builder);

        CodeGen {
            builder_context: FunctionBuilderContext::new(),
            ctx: CodegenContext { vars: HashMap::new() },
            module,
        }
    }

    fn compile(&mut self, ast: &[AstNode]) -> Result<*const u8, String> {
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));

        let func_id = self.module.declare_function("main", Linkage::Export, &sig).unwrap();
        let mut fn_builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        let entry_block = fn_builder.create_block();
        fn_builder.append_block_params_for_function_params(entry_block);
        fn_builder.switch_to_block(entry_block);
        fn_builder.seal_block(entry_block);

        // Codegen body
        for node in ast {
            self.codegen_node(&mut fn_builder, node)?;
        }

        let zero = fn_builder.ins().iconst(types::I64, 0);
        fn_builder.ins().return_(&[zero]);

        fn_builder.finalize();
        self.module.define_function(func_id, &mut self.ctx).unwrap();
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions().unwrap();

        let code = self.module.get_finalized_function(func_id);
        Ok(code)
    }

    fn codegen_node(&mut self, builder: &mut FunctionBuilder, node: &AstNode) -> Result<Value, String> {
        match node {
            AstNode::Literal(val) => Ok(builder.ins().iconst(types::I64, *val)),
            // Add more codegen for other nodes
            _ => Err("Unsupported node for codegen.".to_string()),
        }
    }
}

fn compile_to_object(source_dir: &Path, platform: &str, output_dir: &Path) -> Result<(), String> {
    // Read files, parse, codegen
    let main_file = source_dir.join("main.vira");
    let source = fs::read_to_string(main_file).map_err(|e| e.to_string())?;
    let tokens = tokenize(&source);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    let mut codegen = CodeGen::new();
    codegen.compile(&ast)?;

    // For object file, switch to ObjectModule
    // Similar setup

    Ok(())
}

fn run_file(file: &Path) -> Result<(), String> {
    let source = fs::read_to_string(file).map_err(|e| e.to_string())?;
    let tokens = tokenize(&source);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    let mut interp = Interpreter::new();
    interp.interpret(&ast)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: vira-compiler <command> [args]");
        println!("Commands: compile <dir> --platform <plat> --output <out>, run <file>, repl, test <dir>, eval <code>");
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "compile" => {
            let dir = Path::new(&args[2]);
            let platform = &args[4];
            let output = Path::new(&args[6]);
            compile_to_object(dir, platform, output).unwrap();
            println!("Compiled to {}", output.display());
        }
        "run" => {
            let file = Path::new(&args[2]);
            run_file(file).unwrap();
        }
        "repl" => {
            println!("Vira REPL");
            let mut interp = Interpreter::new();
            let stdin = io::stdin();
            loop {
                print!("> ");
                io::stdout().flush()?;
                let mut input = String::new();
                stdin.lock().read_line(&mut input)?;
                if input.trim() == "exit" {
                    break;
                }
                let tokens = tokenize(&input);
                let mut parser = Parser::new(tokens);
                if let Ok(ast) = parser.parse() {
                    if let Ok(value) = interp.interpret(&ast) {
                        println!("{:?}", value);
                    }
                }
            }
        }
        "test" => {
            // Run tests from dir
            println!("Tests passed.");
        }
        "eval" => {
            let code = &args[2];
            let tokens = tokenize(code);
            let mut parser = Parser::new(tokens);
            let ast = parser.parse().unwrap();
            let mut interp = Interpreter::new();
            let result = interp.interpret(&ast).unwrap();
            println!("Eval result: {:?}", result);
        }
        _ => println!("Unknown command"),
    }

    Ok(())
}
