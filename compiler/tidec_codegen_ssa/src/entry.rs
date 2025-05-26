use tidec_abi::calling_convention::function::FnAbi;
use tidec_lir::{
    lir::{LirBody, LirUnit},
    syntax::{Local, LocalData},
};
use tidec_utils::index_vec::IdxVec;
use tracing::instrument;

use crate::{
    lir::{self, LocalRef},
    traits::{BuilderMethods, PreDefineCodegenMethods},
};

pub struct FnCtx<'a, 'be, B: BuilderMethods<'a, 'be>> {
    /// The function ABI.
    /// This contains information about the calling convention,
    /// argument types, return type, etc.
    pub fn_abi: FnAbi,

    /// The body of the function in LIR.
    pub lir_body: LirBody,

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
    for lir_body in &lir_unit.body {
        ctx.predefine_fn(&lir_body);
    }

    // Create the functions
    for lir_body in lir_unit.body {
        lir::compile_lir_body::<B>(ctx, lir_body);
    }
}
