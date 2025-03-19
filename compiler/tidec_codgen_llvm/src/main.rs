use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::OptimizationLevel;

use std::error::Error;
use std::path::Path;


struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl CodeGen<'_> {
    fn new<'ctx>(context: &'ctx Context, module: Module<'ctx>, builder: Builder<'ctx>) -> CodeGen<'ctx> {
        CodeGen {
            context,
            module,
            builder,
        }
    }
}

// cargo run --bin tidec_codgen_llvm; clang main.ll -o main; ./main; echo $?
//
// Create a simple main function that returns the value stored in the first place.
// ```c
// int main() {
//    int _0 = 5; // The first place
//    return _0;
// }
// ```
fn main() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("main");
    let codegen = CodeGen::new(&context, module, builder);

    let i32_type = codegen.context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = codegen.module.add_function("main", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    // It is important to set the position at the end of the basic block, which in this case is the
    // start of the entry block.
    codegen.builder.position_at_end(basic_block);

    // Declare an integer variable
    let _0 = codegen.builder.build_alloca(i32_type, "_0").unwrap();
    // Store the 5 in the first_place
    let i32_five = i32_type.const_int(5, false);
    let _ = codegen.builder.build_store(_0, i32_five).unwrap();


    // codegen.builder.build_return(Some(&i64_type.const_int(0, false))).unwrap(); // Reutrn 0
    // Dereference the _0 and return it
    let deref_0 = codegen.builder.build_load(i32_type, _0, "_0").unwrap();
    codegen.builder.build_return(Some(&deref_0)).unwrap();

    codegen.module.print_to_file(Path::new("main.ll")).unwrap();
    // module.print_to_stderr();
}
