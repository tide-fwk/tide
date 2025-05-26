use tidec_abi::{
    layout::TyAndLayout,
    size_and_align::{Align, Size},
};
use tidec_lir::{
    lir::{LirBody, LirTyCtx},
    syntax::LirTy,
};

/// This trait is used to define the types used in the codegen backend.
/// It is used to define the types used in the codegen backend.
// FIXME(bruzzone): when `trait alias` is stable, we can use it to alias the `CodegenObject` trait
// pub trait CodegenObject = Copy + PartialEq + std::fmt::Debug;
pub trait CodegenBackendTypes {
    /// A `BasicBlock` is a basic block in the codegen backend.
    type BasicBlock: Copy + PartialEq + std::fmt::Debug;
    /// A `Type` is a type in the codegen backend.
    type Type: Copy + PartialEq + std::fmt::Debug;
    /// A `Value` is an instance of a type in the codegen backend.
    /// Note that this should include `FunctionValue`.
    /// E.g., an instruction, constant, argument, or a function value.
    type Value: Copy + PartialEq + std::fmt::Debug;
    /// A `Function` is a function type in the codegen backend.
    type FunctionType: Copy + PartialEq + std::fmt::Debug;
    /// A `MetadataType` is a metadata type in the codegen backend.
    type MetadataType: Copy + PartialEq + std::fmt::Debug;
    /// A `MetadataValue` is a metadata value in the codegen backend.
    /// E.g., a debug info node or TBAA (Type-Based Alias Analysis) node.
    type MetadataValue: Copy + PartialEq + std::fmt::Debug;
}

/// The codegen backend trait.
/// It is used to define the methods used in the codegen backend.
/// The associated types are used to define the types used in the codegen backend.
pub trait CodegenBackend: Sized + CodegenBackendTypes {
    /// The associated codegen module type.
    // FIXME(bruzzone): add constraints to ensure that the module is compatible with the codegen backend.
    type Module;

    /// The associated codegen context type.
    // FIXME(bruzzone): add constraints to ensure that the context is compatible with the codegen backend.
    type Context;
}

pub trait PreDefineCodegenMethods: Sized + CodegenBackendTypes {
    fn predefine_fn(&self, lir_body: &LirBody);
}

/// The codegen backend methods.
pub trait CodegenMethods<'be>:
    Sized + CodegenBackendTypes + CodegenBackend + PreDefineCodegenMethods
{
    /// Creates a new codegen context for the given LIR type context and module.
    fn new(lir_ty_ctx: LirTyCtx, context: &'be Self::Context, module: Self::Module) -> Self;
    /// Returns the function value for the given LIR body.
    fn get_fn(&self, lir_body: &LirBody) -> Self::Value;
}

/// The builder methods for the codegen backend.
/// This trait is used to define the methods used in the codegen backend.
pub trait BuilderMethods<'a, 'be>: Sized + CodegenBackendTypes {
    /// The associated codegen context type.
    /// This ensures that the codegen context is compatible with the codegen backend types.
    type CodegenCtx: CodegenMethods<
            'be,
            BasicBlock = Self::BasicBlock,
            Type = Self::Type,
            Value = Self::Value,
            FunctionType = Self::FunctionType,
            MetadataType = Self::MetadataType,
            MetadataValue = Self::MetadataValue,
        >;

    fn alloca(&self, size: Size, align: Align) -> Self::Value;

    fn build(ctx: &'a Self::CodegenCtx, bb: Self::BasicBlock) -> Self;

    fn append_basic_block(
        ctx: &'a Self::CodegenCtx,
        fn_value: Self::Value,
        name: &str,
    ) -> Self::BasicBlock;

    fn layout_of(&self, ty: LirTy) -> TyAndLayout<LirTy>;
}
