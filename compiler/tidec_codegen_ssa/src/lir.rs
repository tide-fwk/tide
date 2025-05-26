use tidec_abi::{
    calling_convention::function::FnAbi,
    layout::TyAndLayout,
    size_and_align::{Align, Size},
};
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

impl<'a, 'be, V: Copy + PartialEq + std::fmt::Debug> PlaceRef<V> {
    pub fn alloca<B: BuilderMethods<'a, 'be, Value = V>>(
        builder: &mut B,
        ty_and_layout: TyAndLayout<LirTy>,
    ) -> Self {
        // TODO: Assert that the ty is not unsized (through `TyAndLayout`).
        PlaceVal::alloca(
            builder,
            ty_and_layout.layout.size,
            ty_and_layout.layout.align.abi,
        )
        .with_layout(ty_and_layout)
    }
}

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
impl<'a, 'be, V: Copy + PartialEq + std::fmt::Debug> PlaceVal<V> {
    pub fn alloca<B: BuilderMethods<'a, 'be, Value = V>>(
        builder: &mut B,
        size: Size,
        align: Align,
    ) -> Self {
        let value = builder.alloca(size, align);
        PlaceVal { value, align }
    }

    pub fn with_layout(self, layout: TyAndLayout<LirTy>) -> PlaceRef<V> {
        // TODO: Assert that the type is not unsized (through `TyAndLayout`).
        PlaceRef {
            place_value: self,
            place_ty_layout: layout,
        }
    }
}

/// A local reference in the LIR, representing a local variable or temporary
/// during code generation.
///
/// This enum is used to represent different kinds of local references
/// that can be used in the backend code generation process.
///
/// `LocalRef` is a wrapper around `PlaceRef`, which provides
/// a way to refer to local variables in a type-safe manner
/// while also carrying the necessary metadata for code generation.
pub enum LocalRef<V> {
    /// A local backed by a memory location with associated layout and alignment metadata.
    PlaceRef(PlaceRef<V>),
}

pub fn compile_lir_body<'a, 'be, B: BuilderMethods<'a, 'be>>(
    ctx: &'a B::CodegenCtx,
    lir_body: LirBody,
) {
    let fn_abi = FnAbi {}; // TODO: ctx.get_fn_abi(&lir_body);
    let fn_value = ctx.get_fn(&lir_body);
    let entry_bb = B::append_basic_block(&ctx, fn_value, "entry");
    let mut start_builder = B::build(ctx, entry_bb);

    let mut fn_ctx = FnCtx::<'_, '_, B> {
        fn_abi,
        lir_body,
        fn_value,
        ctx,
        locals: IdxVec::new(),
    };

    let mut allocate_locals =
        |locals: &IdxVec<Local, LocalData>| -> IdxVec<Local, LocalRef<B::Value>> {
            let mut local_allocas = IdxVec::new();

            for (local, local_data) in locals.iter_enumerated() {
                let layout = start_builder.layout_of(local_data.ty);
                let local_ref = LocalRef::PlaceRef(PlaceRef::alloca(&mut start_builder, layout));
                local_allocas[local] = local_ref;
            }

            local_allocas
        };

    // Allocate the return value and arguments
    let mut locals = allocate_locals(&fn_ctx.lir_body.ret_and_args);
    // Allocate the locals
    locals.append(&mut allocate_locals(&fn_ctx.lir_body.locals));

    // Initialize the locals in the function context. Do not set they directly in the `FnCtx`.
    // fn_ctx.locals = locals;

    // Compile the basic blocks
    // for bb in lir_body.basic_blocks.iter() {
    // let llbb = LLVMBasicBlock::from(bb);
    // let builder = CodeGenBuilder::new(ctx);
    // }
}
