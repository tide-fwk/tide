use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use tidec_abi::TyAndLayout;
use tidec_lir::lir::{LirBody, LirTyCtx};
use tidec_lir::syntax::LirTy;

// FIXME(bruzzone): when `trait alias` is stable, we can use it to alias the `CodegenObject` trait
// pub trait CodegenObject = Copy + PartialEq + std::fmt::Debug;
/// This trait is used to define the types used in the codegen backend.
/// It is used to define the types used in the codegen backend.
pub trait CodegenBackendTypes<'be> {
    /// A `BasicBlock` is a basic block in the codegen backend.
    type BasicBlock: Copy + PartialEq + std::fmt::Debug;
    /// A `Type` is a type in the codegen backend.
    type Type: Copy + PartialEq + std::fmt::Debug;
    /// A `Value` is an instance of a type in the codegen backend.
    /// E.g., an instruction, constant, or argument.
    type Value: Copy + PartialEq + std::fmt::Debug;
    /// A `Function` is a function type in the codegen backend.
    /// It is usually a function value.
    type Function: Copy + PartialEq + std::fmt::Debug;
    /// A `MetadataType` is a metadata type in the codegen backend.
    type MetadataType: Copy + PartialEq + std::fmt::Debug;
    /// A `MetadataValue` is a metadata value in the codegen backend.
    /// E.g., a debug info node or TBAA (Type-Based Alias Analysis) node.
    type MetadataValue: Copy + PartialEq + std::fmt::Debug;

    // type Funclet: Copy + PartialEq + std::fmt::Debug;
    // type DIScope: Copy + PartialEq + std::fmt::Debug;
    // type DILocation: Copy + PartialEq + std::fmt::Debug;
    // type DIVariable: Copy + PartialEq + std::fmt::Debug;
}

// TODO: This trait should be generic over the LLVM backend.
pub trait CodegenMethods<'ll> {
    fn new(lir_ty_ctx: LirTyCtx, ll_context: &'ll Context, ll_module: Module<'ll>) -> Self;
    fn get_fn(&self, name: &str) -> Option<FunctionValue<'ll>>;
    fn new_fn(&self, lir_body: &LirBody) -> FunctionValue<'ll>;
}

// =================

// TODO: Make CodegenMethods generic (not LLVM specific)
pub trait BuilderMethods<'a, 'll> {
    type CodegenCtx: CodegenMethods<'ll>;

    fn build(ctx: &'a Self::CodegenCtx, llbb: BasicBlock) -> Self;

    fn append_basic_block(
        ctx: &Self::CodegenCtx,
        fn_value: FunctionValue<'ll>, // TODO: Make FunctionValue generic
        name: &str,
    ) -> BasicBlock<'ll>;

    fn layout_of(&self, ty: LirTy) -> TyAndLayout<LirTy>;
}
