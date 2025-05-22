use tidec_codegen_ssa::traits::{BuilderMethods, CodegenMethods, PreDefineCodegenMethods};
use tidec_lir::lir::{LirBody, LirUnit};
use tidec_lir::syntax::{Local, LocalData};
use tidec_utils::index_vec::IdxVec;
use tracing::instrument;

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
    let llfn_value = ctx.get_fn(&lir_body);
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
    for lir_body in &lir_unit.body {
        ctx.predefine_fn(&lir_body);
    }

    // Create the functions
    for lir_body in lir_unit.body {
        compile_lir_body::<B>(ctx, lir_body);
    }
}
