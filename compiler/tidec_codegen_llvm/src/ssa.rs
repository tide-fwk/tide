use tidec_abi::TyAndLayout;
use tidec_lir::lir::{LirBody, LirTyCtx};
use tidec_lir::syntax::LirTy;

/// This trait is used to define the types used in the codegen backend.
/// It is used to define the types used in the codegen backend.
// FIXME(bruzzone): when `trait alias` is stable, we can use it to alias the `CodegenObject` trait
// pub trait CodegenObject = Copy + PartialEq + std::fmt::Debug;
pub trait CodegenBackendTypes<'be> {
    /// A `BasicBlock` is a basic block in the codegen backend.
    type BasicBlock: Copy + PartialEq + std::fmt::Debug;
    /// A `Type` is a type in the codegen backend.
    type Type: Copy + PartialEq + std::fmt::Debug;
    /// A `Value` is an instance of a type in the codegen backend.
    /// E.g., an instruction, constant, or argument.
    type Value: Copy + PartialEq + std::fmt::Debug;
    /// A `Function` is a function type in the codegen backend.
    type FunctionType: Copy + PartialEq + std::fmt::Debug;
    /// A `FunctionValue` is a function value in the codegen backend.
    /// E.g., a function pointer or a function definition.
    type FunctionValue: Copy + PartialEq + std::fmt::Debug;
    /// A `MetadataType` is a metadata type in the codegen backend.
    type MetadataType: Copy + PartialEq + std::fmt::Debug;
    /// A `MetadataValue` is a metadata value in the codegen backend.
    /// E.g., a debug info node or TBAA (Type-Based Alias Analysis) node.
    type MetadataValue: Copy + PartialEq + std::fmt::Debug;
}

/// The codegen backend trait.
/// It is used to define the methods used in the codegen backend.
/// The associated types are used to define the types used in the codegen backend.
pub trait CodegenBackend<'be>: Sized + CodegenBackendTypes<'be> {
    /// The associated codegen module type.
    // FIXME(bruzzone): add constraints to ensure that the module is compatible with the codegen backend.
    type Module;

    /// The associated codegen context type.
    // FIXME(bruzzone): add constraints to ensure that the context is compatible with the codegen backend.
    type Context;
}

/// The codegen backend methods.
pub trait CodegenMethods<'be>: Sized + CodegenBackendTypes<'be> + CodegenBackend<'be> {
    fn new(lir_ty_ctx: LirTyCtx, context: &'be Self::Context, module: Self::Module) -> Self;
    fn get_fn(&self, name: &str) -> Option<Self::FunctionValue>;
    fn new_fn(&self, lir_body: &LirBody) -> Self::FunctionValue;
}

// =================

/// The builder methods for the codegen backend.
/// This trait is used to define the methods used in the codegen backend.
pub trait BuilderMethods<'a, 'be>: Sized + CodegenBackendTypes<'be> {
    /// The associated codegen context type.
    /// This ensures that the codegen context is compatible with the codegen backend types.
    type CodegenCtx: CodegenMethods<
        'be,
        BasicBlock = Self::BasicBlock,
        Type = Self::Type,
        Value = Self::Value,
        FunctionType = Self::FunctionType,
        FunctionValue = Self::FunctionValue,
        MetadataType = Self::MetadataType,
        MetadataValue = Self::MetadataValue,
    >;

    fn build(ctx: &'a Self::CodegenCtx, bb: Self::BasicBlock) -> Self;

    fn append_basic_block(
        ctx: &'a Self::CodegenCtx,
        fn_value: Self::FunctionValue,
        name: &str,
    ) -> Self::BasicBlock;

    fn layout_of(&self, ty: LirTy) -> TyAndLayout<LirTy>;
}
