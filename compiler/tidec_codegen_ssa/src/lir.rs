use tidec_abi::{Align, TyAndLayout, calling_convention::function::FnAbi};
use tidec_lir::{
    lir::LirBody,
    syntax::{LirTy, Local, LocalData},
};
use tidec_utils::index_vec::IdxVec;

use crate::{
    entry::FnCtx,
    traits::{BuilderMethods, CodegenMethods},
};

/// A reference to a place in memory during code generation, used in LIR to backend lowering.
///
/// `PlaceRef` encapsulates both the backend representation of the place (e.g., a value)
/// and its type layout information. This is used to generate correct code when accessing, modifying,
/// or analyzing memory locations.
///
/// The type parameter `V` represents the backend-specific value, such as an LLVM value or a
/// machine register in custom backends.
pub struct PlaceRef<V> {
    /// The backend value and alignment for this place.
    ///
    /// This typically holds a pointer or immediate value, along with alignment metadata.
    place_value: PlaceVal<V>,
    /// The type and layout of the place, used for determining ABI, size, and alignment.
    ///
    /// This is essential for correct code generation, especially for aggregates, unsized types,
    /// or types with nontrivial ABI requirements.
    place_ty_layout: TyAndLayout<LirTy>,
}
// impl<V: Copy + PartialEq + std::fmt::Debug> PlaceRef<V> {}

/// A backend value paired with alignment information, representing the underlying storage
/// for a LIR place during codegen.
///
/// This struct abstracts over how a place is represented in the backend,
/// whether it be a memory address, an SSA value, or other representations.
///
/// This is tipically used in conjunction with [`PlaceRef`].
///
/// The type parameter `V` is the backend-specific representation of values.
pub struct PlaceVal<V> {
    /// The actual backend value for this place (e.g., pointer, immediate, etc.).
    value: V,
    /// Alignment of the value in memory.
    ///
    /// This is used to ensure proper access semantics and may affect how code is emitted,
    /// especially for aligned loads/stores and optimizations.
    align: Align,
}
// impl<V: Copy + PartialEq + std::fmt::Debug> PlaceVal<V> {}

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
