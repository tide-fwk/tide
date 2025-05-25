use std::ops::Deref;

use inkwell::types::BasicType;
use inkwell::values::{AnyValueEnum, FunctionValue};
use inkwell::{basic_block::BasicBlock, builder::Builder};
use tidec_abi::TyAndLayout;
use tidec_codegen_ssa::traits::{BuilderMethods, CodegenBackendTypes};
use tidec_lir::syntax::LirTy;
use tracing::instrument;

use crate::context::CodegenCtx;
use crate::lir::lir_ty::BasicTypesUtils;

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

impl<'ll> CodegenBackendTypes for CodegenBuilder<'_, 'll> {
    type BasicBlock = <CodegenCtx<'ll> as CodegenBackendTypes>::BasicBlock;
    type Value = <CodegenCtx<'ll> as CodegenBackendTypes>::Value;
    type FunctionType = <CodegenCtx<'ll> as CodegenBackendTypes>::FunctionType;
    type Type = <CodegenCtx<'ll> as CodegenBackendTypes>::Type;
    type MetadataType = <CodegenCtx<'ll> as CodegenBackendTypes>::MetadataType;
    type MetadataValue = <CodegenCtx<'ll> as CodegenBackendTypes>::MetadataValue;
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
    fn build(ctx: &'a CodegenCtx<'ll>, llbb: BasicBlock) -> Self {
        let builder = CodegenBuilder::with_ctx(ctx);
        builder.builder.position_at_end(llbb);
        builder
    }

    /// Append a new basic block to the function.
    ///
    /// # Panic
    ///
    /// Panics if the function is not a valid function value.
    fn append_basic_block(
        ctx: &'a CodegenCtx<'ll>,
        fn_value: AnyValueEnum<'ll>,
        name: &str,
    ) -> BasicBlock<'ll> {
        let fn_value = fn_value.into_function_value(); // TODO: use some thing
                                                       // try_function_and_collect
                                                       // the error
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
