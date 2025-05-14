pub mod builder;
pub mod context;
pub mod lir;
pub mod ssa;

use builder::CodegenBuilder;
use context::CodegenCtx;
use inkwell::context::Context;

use ssa::{compile_lir_unit, BuilderMethods, CodegenMethods};
use tidec_lir::lir::{LirTyCtx, LirUnit};

fn compile_codegen_unit<'ll>(lir_ty_ctx: LirTyCtx, lir_unit: LirUnit) {
    let ll_context = Context::create();
    let ll_module = ll_context.create_module(&lir_unit.metadata.unit_name);
    let ctx = CodegenCtx::new(lir_ty_ctx, &ll_context, ll_module);

    compile_lir_unit::<CodegenBuilder>(&ctx, lir_unit);
}
