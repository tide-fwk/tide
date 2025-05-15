use tidec_abi::TyAndLayout;
use tidec_lir::lir::{LirBody, LirTyCtx, LirUnit};
use tidec_lir::syntax::{LirTy, Local, LocalData};
use tidec_utils::index_vec::IdxVec;
use tracing::instrument;

// =================
// ==== Traits =====
// =================

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

pub trait PreDefineMethods: Sized + CodegenBackendTypes {
    fn predefine_fn(&self, lir_body: &LirBody);
}

/// The codegen backend methods.
pub trait CodegenMethods<'be>:
    Sized + CodegenBackendTypes + CodegenBackend + PreDefineMethods
{
    fn new(lir_ty_ctx: LirTyCtx, context: &'be Self::Context, module: Self::Module) -> Self;
    fn get_fn(&self, name: &str) -> Option<Self::Value>;
    fn get_or_declare_fn(&self, lir_body: &LirBody) -> Self::Value;
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

    fn build(ctx: &'a Self::CodegenCtx, bb: Self::BasicBlock) -> Self;

    fn append_basic_block(
        ctx: &'a Self::CodegenCtx,
        fn_value: Self::Value,
        name: &str,
    ) -> Self::BasicBlock;

    fn layout_of(&self, ty: LirTy) -> TyAndLayout<LirTy>;
}

// =================
// === Functions ===
// =================
struct FnCtx<'a, 'll, B: BuilderMethods<'a, 'll>> {
    // pub locals: IdxVec<Local, LocalRef>,
    /// The body of the function in LIR.
    lir_body: LirBody,

    /// The LLVM function value.
    /// This is the function that will be generated.
    llfn_value: B::Value,

    /// The LLVM codegen context.
    ctx: &'a B::CodegenCtx,

    // The allocated locals and temporaries for the function.
    locals: IdxVec<Local, B::Value>,
}

impl<'ctx, 'll, B: BuilderMethods<'ctx, 'll>> FnCtx<'ctx, 'll, B> {
    pub fn init_local(&mut self, locals: &IdxVec<Local, LocalData>) {
        // TODO
    }
}

fn compile_lir_body<'a, 'll, B: BuilderMethods<'a, 'll>>(
    ctx: &'a B::CodegenCtx,
    lir_body: LirBody,
) {
    let llfn_value = ctx.get_or_declare_fn(&lir_body);
    let entry_bb = B::append_basic_block(&ctx, llfn_value, "entry");
    let start_builder = B::build(ctx, entry_bb);

    let mut fn_ctx = FnCtx::<'_, '_, B> {
        lir_body,
        llfn_value,
        ctx,
        locals: IdxVec::new(),
    };

    let allocate_locals =
        |fn_value: B::Value, locals: &IdxVec<Local, LocalData>| -> IdxVec<Local, B::Value> {
            let mut local_allocas = IdxVec::new();

            for (local, local_data) in locals.iter_enumerated() {
                let layout = start_builder.layout_of(local_data.ty);
                // let alloca =
                // local_allocas[local] = alloca;
            }

            local_allocas
        };

    // Allocate the return value and arguments
    let mut locals = allocate_locals(fn_ctx.llfn_value, &fn_ctx.lir_body.ret_and_args);
    // Allocate the locals
    locals.append(&mut allocate_locals(
        fn_ctx.llfn_value,
        &fn_ctx.lir_body.locals,
    ));

    // Initialize the locals in the function context
    fn_ctx.locals = locals;

    // Compile the basic blocks
    // for bb in lir_body.basic_blocks.iter() {
    // let llbb = LLVMBasicBlock::from(bb);
    // let builder = CodeGenBuilder::new(ctx);
    // }
}

#[instrument(skip(ctx, lir_unit))]
pub fn compile_lir_unit<'a, 'll, B: BuilderMethods<'a, 'll>>(
    ctx: &'a B::CodegenCtx,
    lir_unit: LirUnit,
) {
    // Create the functions
    for lir_body in lir_unit.body {
        compile_lir_body::<B>(ctx, lir_body);
    }
}
