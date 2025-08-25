use std::ops::Deref;

use inkwell::llvm_sys::core::{LLVMIsAGlobalVariable, LLVMIsGlobalConstant};
use inkwell::llvm_sys::prelude::LLVMBool;
use inkwell::values::{AnyValueEnum, AsValueRef, BasicValue, GlobalValue};
use inkwell::{basic_block::BasicBlock, builder::Builder};
use tidec_abi::size_and_align::{Align, Size};
use tidec_codegen_ssa::lir::{OperandRef, PlaceRef};
use tidec_codegen_ssa::traits::{BuilderMethods, CodegenBackendTypes};
use tracing::instrument;

use crate::context::CodegenCtx;
use crate::lir::lir_ty::BasicTypesUtils;

/// A builder for generating LLVM IR code.
///
/// This struct wraps the `inkwell::builder::Builder` and provides
/// additional methods for code generation.
pub struct CodegenBuilder<'a, 'll> {
    pub ll_builder: Builder<'ll>,
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
        let ll_builder = ctx.ll_context.create_builder();
        CodegenBuilder { ll_builder, ctx }
    }
}

impl<'a, 'll> BuilderMethods<'a, 'll> for CodegenBuilder<'a, 'll> {
    type CodegenCtx = CodegenCtx<'ll>;

    fn ctx(&self) -> &Self::CodegenCtx {
        self.ctx
    }

    /// Create a new CodeGenBuilder from a CodeGenCtx and a BasicBlock.
    /// The builder is positioned at the end of the BasicBlock.
    fn build(ctx: &'a CodegenCtx<'ll>, llbb: BasicBlock) -> Self {
        let builder = CodegenBuilder::with_ctx(ctx);
        builder.ll_builder.position_at_end(llbb);
        builder
    }

    #[instrument(skip(self))]
    /// Allocate memory for a value of the given size and alignment.
    fn alloca(&self, size: Size, align: Align) -> Self::Value {
        let builder = CodegenBuilder::with_ctx(self.ctx);
        builder
            .ll_builder
            .position_at_end(builder.ll_builder.get_insert_block().unwrap());
        let ty = self
            .ctx
            .ll_context
            .i8_type()
            .array_type(size.bytes() as u32);
        let name = ""; // Generate a unique name for the alloca

        match builder.ll_builder.build_alloca(ty, name) {
            Ok(pointer_value) => {
                if let Err(err) = pointer_value
                    .as_instruction()
                    .unwrap()
                    .set_alignment(align.bytes() as u32)
                {
                    panic!("Failed to set alignment: {}", err);
                }
                pointer_value.into()
            }
            Err(err) => {
                panic!("Failed to allocate memory: {}", err);
            }
        }
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
        let fn_value = fn_value.into_function_value(); // TODO: use something try_function_and_collect the error
        ctx.ll_context.append_basic_block(fn_value, name)
    }

    #[instrument(level = "trace", skip(self))]
    fn load_operand(&mut self, place_ref: &PlaceRef<Self::Value>) -> OperandRef<Self::Value> {
        if place_ref.ty_layout.is_zst() {
            return OperandRef::new_zst(place_ref.ty_layout);
        }

        if place_ref.ty_layout.is_immediate() {
            let mut ll_global_const: Option<AnyValueEnum> = None;
            let llty = place_ref.ty_layout.ty.into_basic_type(self.ctx);

            // TODO: Move to a safe version
            unsafe {
                let llval = LLVMIsAGlobalVariable(place_ref.place_val.value.as_value_ref());
                if LLVMIsGlobalConstant(llval) == LLVMBool::from(0) {
                    let global_val = GlobalValue::new(llval);
                    let loaded_val = global_val.get_initializer().unwrap();
                    assert_eq!(loaded_val.get_type(), llty);
                    ll_global_const = Some(loaded_val.into());
                }
            }

            let llval = ll_global_const.unwrap_or_else(|| {
                let loaded_val = self.build_load(
                    llty,
                    place_ref.place_val.value,
                    place_ref.place_val.align,
                );
                // TODO: Here we should call: 
                //
                // 1) scalar_load_metadata(...) 
                // Attaches LLVM metadata to the load instruction (the one that just pulled load from memory). 
                // This metadata guides LLVM optimizations and correctness: 
                // e.g. alignment info, nonnull if it’s a pointer, range for integers, noalias hints, etc. 
                // So if you load an &T, the compiler may add metadata saying “this pointer is non-null”. 
                //
                // 2) self.to_immediate_scalar(load, scalar) 
                // Converts the loaded LLVM value (load) into an immediate scalar representation in Tide’s codegen world. 
                // Why? Because some scalars (e.g., booleans) need normalization: Tide booleans are guaranteed to be 0 or 1, 
                // but LLVM might treat them as any non-zero integer. to_immediate_scalar ensures consistency with Tide’s semantics.
                loaded_val
            });

            OperandRef::new_immediate(llval, place_ref.ty_layout)
        } else {
            todo!("Handle non-immediate types — when the layout is, for example, `Memory`");
        }
    }

    /// Build a return instruction for the given builder.
    /// If the return value is `None`, it means that the function returns `void`,
    /// otherwise it returns the given value.
    fn build_return(&mut self, ret_val: Option<AnyValueEnum<'ll>>) {
        match ret_val {
            None => {
                self.ll_builder.build_return(None);
            }
            Some(val) => {
                todo!("Handle return value");
                // self.ll_builder.build_return(Some(&val));
            }
        }
    }

    /// Build a load instruction to load a value from the given pointer.
    fn build_load(&mut self, ty: Self::Type, ptr: Self::Value, align: Align) -> Self::Value {
        let load_inst = match self.ll_builder.build_load(ty, ptr.into_pointer_value(), "") {
            Ok(v) => v,
            Err(err) => panic!("Failed to build load instruction: {}", err),
        };

        load_inst
            .as_instruction_value()
            .unwrap()
            .set_alignment(align.bytes() as u32)
            .expect("Failed to set alignment");

        load_inst.into()
    }
}
