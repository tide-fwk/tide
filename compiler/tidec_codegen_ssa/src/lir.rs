use tidec_abi::calling_convention::function::FnAbi;
use tidec_lir::{
    lir::LirBody,
    syntax::{Local, LocalData},
};
use tidec_utils::index_vec::IdxVec;

use crate::{
    entry::FnCtx,
    traits::{BuilderMethods, CodegenMethods},
};

pub fn compile_lir_body<'a, 'be, B: BuilderMethods<'a, 'be>>(
    ctx: &'a B::CodegenCtx,
    lir_body: LirBody,
) {
    let fn_abi = FnAbi {}; // TODO: ctx.get_fn_abi(&lir_body);
    let fn_value = ctx.get_fn(&lir_body);
    let entry_bb = B::append_basic_block(&ctx, fn_value, "entry");
    let start_builder = B::build(ctx, entry_bb);

    let mut fn_ctx = FnCtx::<'_, '_, B> {
        fn_abi,
        lir_body,
        fn_value,
        ctx,
        locals: IdxVec::new(),
    };

    let allocate_locals =
        |fn_value: B::Value, locals: &IdxVec<Local, LocalData>| -> IdxVec<Local, B::Value> {
            let mut local_allocas = IdxVec::new();

            for (local, local_data) in locals.iter_enumerated() {
                let layout = start_builder.layout_of(local_data.ty);
                // let ref_local =
                // local_allocas[local] = ref_local;
            }

            local_allocas
        };

    // Allocate the return value and arguments
    let mut locals = allocate_locals(fn_ctx.fn_value, &fn_ctx.lir_body.ret_and_args);
    // Allocate the locals
    locals.append(&mut allocate_locals(
        fn_ctx.fn_value,
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
