use std::ops::Deref;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{FunctionValue, IntValue};
use inkwell::{basic_block::BasicBlock, builder::Builder};
use tidec_abi::TyAndLayout;
use tracing::instrument;

use crate::context::CodegenCtx;
use crate::lir::types::BasicTypesUtils;
use crate::ssa_traits::CodegenBackendTypes;
use crate::BuilderMethods;
use tidec_lir::lir::{LirBody, LirUnit};
use tidec_lir::syntax::{LirTy, Local, LocalData, RETURN_PLACE};

/// A builder for generating LLVM IR code.
///
/// This struct wraps the `inkwell::builder::Builder` and provides
/// additional methods for code generation.
pub struct CodegenBuilder<'a, 'll> {
    pub builder: Builder<'ll>,
    pub ctx: &'a CodegenCtx<'ll>,
}

impl<'ll> Deref for CodegenBuilder<'_, 'll> {
    type Target = CodegenCtx<'ll>;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'ll> CodegenBackendTypes<'ll> for CodegenBuilder<'_, 'll> {
    type BasicBlock = <CodegenCtx<'ll> as CodegenBackendTypes<'ll>>::BasicBlock;
    type Value = <CodegenCtx<'ll> as CodegenBackendTypes<'ll>>::Value;
    type Function = <CodegenCtx<'ll> as CodegenBackendTypes<'ll>>::Function;
    type Type = <CodegenCtx<'ll> as CodegenBackendTypes<'ll>>::Type;
    type MetadataType = <CodegenCtx<'ll> as CodegenBackendTypes<'ll>>::MetadataType;
    type MetadataValue = <CodegenCtx<'ll> as CodegenBackendTypes<'ll>>::MetadataValue;
}

impl<'a, 'll> CodegenBuilder<'a, 'll> {
    #[instrument(skip(ctx))]
    pub fn with_ctx(ctx: &'a CodegenCtx<'ll>) -> Self {
        let builder = ctx.ll_context.create_builder();
        CodegenBuilder { builder, ctx }
    }

    pub fn llvm_layout_of(&self, ty: LirTy) -> Result<TyAndLayout<LirTy>, String> {
        let size = ty.into_basic_type(self.ctx).size_of();
        // let align = ty.into_basic_type(self.ctx).get

        Err("tmp".to_string())
    }
}

impl<'a, 'll> BuilderMethods<'a, 'll> for CodegenBuilder<'a, 'll> {
    type CodegenCtx = CodegenCtx<'ll>;

    /// Create a new CodeGenBuilder from a CodeGenCtx and a BasicBlock.
    /// The builder is positioned at the end of the BasicBlock.
    fn build(ctx: &'a Self::CodegenCtx, llbb: BasicBlock) -> Self {
        let builder = CodegenBuilder::with_ctx(ctx);
        builder.builder.position_at_end(llbb);
        builder
    }

    /// Append a new basic block to the function.
    fn append_basic_block(
        ctx: &Self::CodegenCtx,
        fn_value: FunctionValue<'ll>,
        name: &str,
    ) -> BasicBlock<'ll> {
        ctx.ll_context.append_basic_block(fn_value, name)
    }

    fn layout_of(&self, ty: LirTy) -> TyAndLayout<LirTy> {
        match self.llvm_layout_of(ty) {
            Ok(layout) => layout,
            Err(err) => {
                // Fallback to the LIR type context
                self.lir_ty_ctx.layout_of(ty)
            }
        }
    }
}
