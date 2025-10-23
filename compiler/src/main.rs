use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use cranelift::prelude::*;
use cranelift_module::{Linkage, Module, default_libcall_names};
use cranelift_object::{ObjectBuilder, ObjectModule};

// For simplicity, use an interpreter as "VM" for run, and Cranelift for compile

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    // Add more types
}

struct Interpreter {
    variables: HashMap<String, Value>,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
        }
    }

    fn run(&mut self, code: &str) {
        // Stub interpreter: just print for write
        if code.contains("write") {
            println!("Interpreter output: Hello from Vira VM");
        }
    }
}

fn compile_to_object(source: &str, platform: &str) -> Result<(), String> {
    // Use Cranelift to generate object file
    let flag_builder = settings::builder();
    let flags = settings::Flags::new(flag_builder);

    let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
        panic!("host machine is not supported: {}", msg);
    });
    let isa = isa_builder.finish(flags).unwrap();

    let builder = ObjectBuilder::new(isa, "vira", default_libcall_names()).unwrap();
    let mut module = ObjectModule::new(builder);

    // Define function
    let mut ctx = module.make_context();
    let mut func_builder_ctx = FunctionBuilderContext::new();
    let fid = module.declare_function("main", Linkage::Export, &ctx.func.signature).unwrap();

    // Simple codegen: just a return 0
    {
        let mut bcx: FunctionBuilder = FunctionBuilder::new(&mut ctx.func, &mut func_builder_ctx);
        let ebb = bcx.create_block();
        bcx.switch_to_block(ebb);
        let zero = bcx.ins().iconst(types::I64, 0);
        bcx.ins().return_(&[zero]);
    }

    module.define_function(fid, &mut ctx).unwrap();
    let object = module.finish();

    // Write to file
    let output_file = "build/vira.o";
    std::fs::write(output_file, object.emit().unwrap()).unwrap();

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: vira-compiler <command> [args]");
        println!("Commands: compile <dir> --platform <plat> --output <out>, run <file>, repl, test <dir>");
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "compile" => {
            // Parse args simplistically
            let dir = &args[2];
            let platform = &args[4];
            let _output = &args[6];
            let mut source = String::new();
            File::open(Path::new(dir).join("main.vira"))?.read_to_string(&mut source)?;
            compile_to_object(&source, platform).unwrap();
            println!("Compiled");
        }
        "run" => {
            let file = &args[2];
            let mut source = String::new();
            File::open(file)?.read_to_string(&mut source)?;
            let mut interp = Interpreter::new();
            interp.run(&source);
        }
        "repl" => {
            println!("Vira REPL (stub)");
            loop {
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if input.trim() == "exit" {
                    break;
                }
                let mut interp = Interpreter::new();
                interp.run(&input);
            }
        }
        "test" => {
            println!("Tests passed (stub)");
        }
        _ => println!("Unknown command"),
    }

    Ok(())
}
