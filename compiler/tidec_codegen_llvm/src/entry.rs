use inkwell::context::Context;

use tidec_codegen_ssa::{entry::compile_lir_unit, traits::CodegenMethods};
use tidec_lir::lir::{LirTyCtx, LirUnit};

use crate::{builder::CodegenBuilder, context::CodegenCtx};

pub fn compile_codegen_unit<'ll>(lir_ty_ctx: LirTyCtx, lir_unit: LirUnit) {
    let ll_context = Context::create();
    let ll_module = ll_context.create_module(&lir_unit.metadata.unit_name);
    let ctx = CodegenCtx::new(lir_ty_ctx, &ll_context, ll_module);

    compile_lir_unit::<CodegenBuilder>(&ctx, lir_unit);
}
