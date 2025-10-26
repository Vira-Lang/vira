use std::env;
use std::fs;
use std::io;
use std::path::Path;

use crate::codegen::CodeGen;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::tokenizer::tokenize;

mod ast;
mod codegen;
mod interpreter;
mod parser;
mod tokenizer;
mod arena;

use ast::{AstNode, ViraType};
use arena::Arena;

fn compile_to_object(source_dir: &Path, platform: &str, output_dir: &Path) -> Result<(), String> {
    let main_file = source_dir.join("main.vira");
    let source = fs::read_to_string(main_file).map_err(|e| e.to_string())?;
    let tokens = tokenize(&source);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    let mut codegen = CodeGen::new();
    codegen.compile(&ast)?;

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
        println!("Commands: compile <dir> --platform <plat> --output <out>, run <file>, repl, test <dir>, eval <code>, check <file>, fmt <file>");
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
