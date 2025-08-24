use tidec_abi::calling_convention::function::FnAbi;
use tidec_lir::{
    lir::{LirBody, LirUnit},
    syntax::{Local, LocalData},
};
use tidec_utils::index_vec::IdxVec;
use tracing::instrument;

use crate::{
    lir::LocalRef,
    traits::{BuilderMethods, DefineCodegenMethods, PreDefineCodegenMethods},
};

pub struct FnCtx<'a, 'be, B: BuilderMethods<'a, 'be>> {
    /// The function ABI.
    /// This contains information about the calling convention,
    /// argument types, return type, etc.
    pub fn_abi: FnAbi,

    /// The body of the function in LIR.
    pub lir_body: &'a LirBody,

    /// The function value.
    /// This is the function that will be generated.
    pub fn_value: B::Value,

    /// The codegen context.
    pub ctx: &'a B::CodegenCtx,

    /// The allocated locals and temporaries for the function.
    ///
    /// Note that the `B::Value` type is used to represent the local references.
    pub locals: IdxVec<Local, LocalRef<B::Value>>,
}

impl<'ctx, 'll, B: BuilderMethods<'ctx, 'll>> FnCtx<'ctx, 'll, B> {
    pub fn init_local(&mut self, locals: &IdxVec<Local, LocalData>) {
        // TODO
    }
}

#[instrument(skip(ctx, lir_unit))]
pub fn compile_lir_unit<'a, 'be, B: BuilderMethods<'a, 'be>>(
    ctx: &'a B::CodegenCtx,
    lir_unit: LirUnit,
) {
    // Predefine the functions. That is, create the function declarations.
    for lir_body in &lir_unit.bodies {
        ctx.predefine_body(&lir_body.metadata, &lir_body.ret_and_args);
    }

    // Now that all functions are pre-defined, we can compile the bodies.
    for lir_body in &lir_unit.bodies {
        // It corresponds to:
        // ```rust
        // for &(mono_item, item_data) in &mono_items {
        //     mono_item.define::<Builder<'_, '_, '_>>(&mut cx, cgu_name.as_str(), item_data);
        // }
        // ```
        // in rustc_codegen_llvm/src/base.rs
        // lir::define_lir_body::<B>(ctx, lir_body);
        ctx.define_body(lir_body);
    }
}
