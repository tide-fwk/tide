use std::path::Path;

use crate::{builder::CodegenBuilder, context::CodegenCtx};
use inkwell::targets::{InitializationConfig, Target, TargetMachine, RelocMode, CodeModel, FileType};
use inkwell::context::Context;
use tidec_codegen_ssa::{entry::compile_lir_unit, traits::CodegenMethods};
use tidec_lir::lir::{LirTyCtx, LirUnit};
use tracing::{debug, instrument};

#[instrument(level = "info", skip(lir_ty_ctx, lir_unit, print_ir), fields(unit = %lir_unit.metadata.unit_name))]
// TODO(bruzzone): try to move it to `tidec_codegen_ssa`
pub fn compile_codegen_unit<'ll>(lir_ty_ctx: LirTyCtx, lir_unit: LirUnit, print_ir: bool) {
    let ll_context = Context::create();
    let ll_module = ll_context.create_module(&lir_unit.metadata.unit_name);
    let ctx = CodegenCtx::new(lir_ty_ctx, &ll_context, ll_module);

    compile_lir_unit::<CodegenBuilder>(&ctx, lir_unit);

    Target::initialize_all(&InitializationConfig::default());
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).expect("Failed to get target from triple");
    let cpu = TargetMachine::get_host_cpu_name().to_string();
    let features = TargetMachine::get_host_cpu_features().to_string();
    let target_machine = target
        .create_target_machine(
            &triple,
            &cpu,
            &features,
            inkwell::OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .expect("Failed to create target machine");
    let obj_path = format!("{}.o", &ctx.ll_module.get_name().to_str().unwrap());
    target_machine
        .write_to_file(&ctx.ll_module, FileType::Object, Path::new(&obj_path))
        .expect("Failed to write object file");
    debug!("Wrote object file to {}", obj_path);


    if print_ir {
        ctx.ll_module.print_to_stderr();
    }
}
