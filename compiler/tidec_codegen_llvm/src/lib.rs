pub mod builder;
pub mod context;
pub mod lir; // FIXME

use std::ops::Deref;

use builder::CodegenBuilder;
use context::CodegenCtx;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::FunctionValue;
use inkwell::{basic_block::BasicBlock, builder::Builder};

use lir::types::BasicTypesUtils;
use tidec_abi::TyAndLayout;
use tidec_lir::lir::{LirBody, LirTyCtx, LirUnit};
use tidec_lir::syntax::{LirTy, Local, LocalData, RETURN_PLACE};
use tidec_utils::index_vec::IdxVec;

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

struct FnCtx<'a, 'll, B: BuilderMethods<'a, 'll>> {
    // pub locals: IdxVec<Local, LocalRef>,
    /// The body of the function in LIR.
    lir_body: LirBody,

    /// The LLVM function value.
    /// This is the function that will be generated.
    llfn_value: FunctionValue<'ll>,

    /// The LLVM codegen context.
    ctx: &'a B::CodegenCtx,

    // The allocated locals and temporaries for the function.
    locals: IdxVec<Local, BasicTypeEnum<'ll>>,
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
    let llfn_value = ctx.new_fn(&lir_body);
    let entry_bb = B::append_basic_block(&ctx, llfn_value, "entry");
    let start_builder = B::build(ctx, entry_bb);

    let mut fn_ctx = FnCtx::<'_, '_, B> {
        lir_body,
        llfn_value,
        ctx,
        locals: IdxVec::new(),
    };

    let allocate_locals = |fn_value: FunctionValue<'ll>,
                           locals: &IdxVec<Local, LocalData>|
     -> IdxVec<Local, BasicTypeEnum<'ll>> {
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

fn compile_lir_unit<'a, 'll, B: BuilderMethods<'a, 'll>>(
    ctx: &'a B::CodegenCtx,
    lir_unit: LirUnit,
) {
    // Create the functions
    for lir_body in lir_unit.body {
        compile_lir_body::<B>(ctx, lir_body);
    }
}

fn compile_codegen_unit<'ll>(lir_ty_ctx: LirTyCtx, lir_unit: LirUnit) {
    let ll_context = Context::create();
    let ll_module = ll_context.create_module(&lir_unit.metadata.unit_name);
    let ctx = CodegenCtx::new(lir_ty_ctx, &ll_context, ll_module);

    compile_lir_unit::<CodegenBuilder>(&ctx, lir_unit);
}
