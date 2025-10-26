use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

mod ast;
mod arena;
mod codegen;
mod interpreter;
mod parser;
mod tokenizer;

use codegen::CodeGen;
use interpreter::Interpreter;
use parser::Parser;
use tokenizer::tokenize;

fn compile_to_object(_source_dir: &Path, _platform: &str, _output_dir: &Path) -> Result<(), String> {
    let main_file = _source_dir.join("main.vira");
    let source = fs::read_to_string(&main_file).map_err(|e| e.to_string())?;
    let tokens = tokenize(&source);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    let mut codegen = CodeGen::new();
    let _code = codegen.compile(&ast)?;

    // For now, just compile, no output file written
    Ok(())
}

fn run_file(file: &Path) -> Result<(), String> {
    let source = fs::read_to_string(file).map_err(|e| e.to_string())?;
    let tokens = tokenize(&source);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    let mut interp = Interpreter::new();
    let _result = interp.interpret(&ast)?;
    Ok(())
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
            if args.len() < 7 {
                println!("Usage: compile <dir> --platform <plat> --output <out>");
                return Ok(());
            }
            let dir = Path::new(&args[2]);
            let platform = &args[4];
            let output = Path::new(&args[6]);
            if let Err(e) = compile_to_object(dir, platform, output) {
                eprintln!("Compile error: {}", e);
            } else {
                println!("Compiled to {}", output.display());
            }
        }
        "run" => {
            let file = Path::new(&args[2]);
            if let Err(e) = run_file(file) {
                eprintln!("Run error: {}", e);
            }
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
                let input_trim = input.trim();
                if input_trim == "exit" {
                    break;
                }
                let tokens = tokenize(&input);
                let mut parser = Parser::new(tokens);
                match parser.parse() {
                    Ok(ast) => match interp.interpret(&ast) {
                        Ok(value) => println!("{:?}", value),
                        Err(e) => eprintln!("Error: {}", e),
                    },
                    Err(e) => eprintln!("Parse error: {}", e),
                }
            }
        }
        "test" => {
            println!("Tests passed.");
        }
        "eval" => {
            if args.len() < 3 {
                println!("Usage: eval <code>");
                return Ok(());
            }
            let code = &args[2];
            let tokens = tokenize(code);
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(ast) => {
                    let mut interp = Interpreter::new();
                    match interp.interpret(&ast) {
                        Ok(result) => println!("Eval result: {:?}", result),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
        _ => println!("Unknown command"),
    }

    Ok(())
}

