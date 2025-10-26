use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use crate::ast::AstNode;

pub struct CodeGen {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: JITModule,
}

impl CodeGen {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let flags = settings::Flags::new(flag_builder);
        let isa = isa_builder.finish(flags).unwrap();
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        CodeGen {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        }
    }

    pub fn compile(&mut self, ast: &[AstNode]) -> Result<*const u8, String> {
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));

        let func_id = self.module.declare_function("main", Linkage::Export, &sig).unwrap();
        let mut fn_builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        let entry_block = fn_builder.create_block();
        fn_builder.switch_to_block(entry_block);
        fn_builder.seal_block(entry_block);

        for node in ast {
            CodeGen::codegen_node(&mut fn_builder, node)?;
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

    fn codegen_node(builder: &mut FunctionBuilder, node: &AstNode) -> Result<Value, String> {
        match node {
            AstNode::Literal(val) => Ok(builder.ins().iconst(types::I64, *val)),
            AstNode::FloatLiteral(val) => Ok(builder.ins().f64const(*val)),
            // Expand for other nodes, binary ops, etc.
            _ => Err("Unsupported node for codegen.".to_string()),
        }
    }
}
