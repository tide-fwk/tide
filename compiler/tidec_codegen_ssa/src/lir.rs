use crate::traits::{FnAbiOf, LayoutOf};
use crate::{
    entry::FnCtx,
    traits::{BuilderMethods, CodegenMethods},
};
use tidec_abi::layout::BackendRepr;
use tidec_abi::{
    layout::TyAndLayout,
    size_and_align::{Align, Size},
};
use tidec_lir::basic_blocks::ENTRY_BLOCK;
use tidec_lir::syntax::{ConstOperand, ConstValue};
use tidec_lir::{
    lir::LirBody,
    syntax::{LirTy, Local, LocalData},
};
use tidec_utils::index_vec::IdxVec;
use tracing::{debug, instrument};

#[derive(Debug, Clone, Copy)]
/// Represents a memory location or “place” during code generation.
///
/// `PlaceRef` encapsulates both the **backend-level representation** of a place
/// (how the value is passed, stored, or manipulated at the ABI/codegen level)
/// and its **type/layout information**, which is needed to generate correct
/// memory accesses, handle aggregates, and respect alignment and size requirements.
///
/// The type parameter `V` represents a backend-specific value, such as a machine
/// register, LLVM value, or other intermediate representation used by the backend.
pub struct PlaceRef<V: std::fmt::Debug> {
    /// The backend value of this place.
    ///
    /// This corresponds to the actual value used by the backend for code generation,
    /// e.g., a register, stack slot, or pointer. Its form is determined by the
    /// type’s `backend_repr` from the layout, which describes how the type is
    /// passed or stored (scalar, scalar pair, memory, etc.).
    pub place_val: PlaceVal<V>,
    /// The type and layout of this place.
    ///
    /// Provides size, alignment, and ABI information, which is essential for
    /// correct code generation, especially for aggregates, unsized types,
    /// or types with nontrivial ABI requirements.
    pub ty_layout: TyAndLayout<LirTy>,
}

#[derive(Debug, Clone, Copy)]
/// Represents a computed value or operand during code generation.
///
/// `OperandRef` holds a value that can be used directly in computations,
/// without necessarily having a memory location. This can include immediate
/// scalars, scalar pairs (e.g., fat pointers), or references to memory locations.
pub struct OperandRef<V: std::fmt::Debug> {
    /// The actual value of the operand in the backend.
    ///
    /// May be an immediate scalar, a pair of scalars, or a reference to a `PlaceVal`.
    pub operand_val: OperandVal<V>,
    /// The type and layout of the operand.
    ///
    /// Provides size, alignment, and ABI information needed for correct
    /// code generation and backend handling.
    pub ty_layout: TyAndLayout<LirTy>,
}

impl<V: std::fmt::Debug> OperandRef<V> {
    pub fn new_zst(ty_layout: TyAndLayout<LirTy>) -> Self {
        OperandRef {
            operand_val: OperandVal::Zst,
            ty_layout,
        }
    }

    pub fn new_immediate(value: V, ty_layout: TyAndLayout<LirTy>) -> Self {
        OperandRef {
            operand_val: OperandVal::Immediate(value),
            ty_layout,
        }
    }

    pub fn new_const<'a, 'be, B: BuilderMethods<'a, 'be, Value = V>>(
        builder: &mut B,
        const_val: ConstValue,
        lir_ty: LirTy,
    ) -> Self {
        let ty_layout = builder.ctx().layout_of(lir_ty);
        let be_val = match const_val {
            ConstValue::Scalar(const_scalar) => {
                assert!(matches!(ty_layout.backend_repr, BackendRepr::Scalar(_)));
                let be_val = builder.const_scalar_to_backend_value(const_scalar, ty_layout);
                OperandVal::Immediate(be_val)
            }
            ConstValue::ZST => {
                assert!(ty_layout.is_zst());
                OperandVal::Zst
            }
        };
        OperandRef {
            operand_val: be_val,
            ty_layout,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Backend representation of an operand value.
///
/// This enum captures the different forms a value may take at the backend:
/// - `Zst` — represents a zero-sized type (ZST) which has no data.
/// - `Immediate(V)` — a single scalar value (integer, float, pointer, etc.)
/// - `Pair(V, V)` — two scalars representing a compound value, such as a fat pointer (`&[T]` or `&str`)
/// - `Ref(PlaceVal<V>)` — a reference to a memory location, allowing indirect access
///   to the value.
pub enum OperandVal<V: std::fmt::Debug> {
    /// A zero-sized type (ZST) has no data and thus no value.
    Zst,
    /// A single immediate value.
    Immediate(V),
    /// Two values representing a compound operand.
    Pair(V, V),
    /// A reference to a place in memory.
    Ref(PlaceVal<V>),
}

impl<'a, 'be, V: Copy + PartialEq + std::fmt::Debug> PlaceRef<V> {
    pub fn alloca<B: BuilderMethods<'a, 'be, Value = V>>(
        builder: &mut B,
        ty_and_layout: TyAndLayout<LirTy>,
    ) -> Self {
        assert!(!ty_and_layout.is_zst());
        PlaceVal::alloca(
            builder,
            ty_and_layout.layout.size,
            ty_and_layout.layout.align.abi,
        )
        .with_layout(ty_and_layout)
    }
}

#[derive(Debug, Clone, Copy)]
/// A backend value paired with alignment information, representing the underlying storage
/// for a LIR place during codegen.
///
/// This struct abstracts over how a place is represented in the backend,
/// whether it be a memory address, an SSA value, or other representations.
///
/// This is tipically used in conjunction with [`PlaceRef`].
///
/// The type parameter `V` is the backend-specific representation of values.
pub struct PlaceVal<V: std::fmt::Debug> {
    /// The actual backend value for this place (e.g., pointer, immediate, etc.).
    pub value: V,
    /// Alignment of the value in memory.
    ///
    /// This is used to ensure proper access semantics and may affect how code is emitted,
    /// especially for aligned loads/stores and optimizations.
    pub align: Align,
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
            place_val: self,
            ty_layout: layout,
        }
    }
}

#[derive(Debug)]
/// A local reference in the LIR, representing a local variable or temporary
/// during code generation.
///
/// This enum is used to represent different kinds of local references
/// that can be used in the backend code generation process.
///
/// From a source-level perspective, locals can be thought of as
/// variables declared within a function scope.
pub enum LocalRef<V: std::fmt::Debug> {
    /// A local backed by a memory location with associated layout and alignment metadata.
    ///
    /// From a source-level perspective, this corresponds to variables
    /// that have a defined memory location, such as stack-allocated variables.
    /// See [`tided_lir::syntax::Place`] for more details.
    PlaceRef(PlaceRef<V>),
    /// A local represented as an operand value, which can be used directly in computations.
    ///
    /// From a source-level perspective, this corresponds to temporary values
    /// that do not have a dedicated memory location, such as intermediate
    /// results in expressions.
    /// See [`tidec_lir::syntax::Operand`] for more details.
    OperandRef(OperandRef<V>),
    /// A local that is yet to be assigned a value.
    /// This is a placeholder for locals that will be initialized later.
    /// It is used to represent uninitialized locals during code generation.
    PendingOperandRef,
}

#[instrument(level = "debug", skip(ctx, lir_body))]
/// Define (compile) a LIR function body into the backend representation.
// It corresponds to the:
// ```rust
// pub fn codegen_mir<'a, 'tcx, Bx: BuilderMethods<'a, 'tcx>>(
//     cx: &'a Bx::CodegenCx,
//     instance: Instance<'tcx>,
// ) { ... }
// ```
// function in rustc_codegen_ssa/src/mir/mod.rs
pub fn codegen_lir_body<'a, 'be, B: BuilderMethods<'a, 'be>>(
    ctx: &'a B::CodegenCtx,
    lir_body: &'a LirBody,
) {
    let fn_abi = ctx.fn_abi_of(ctx.lit_ty_ctx(), &lir_body.ret_and_args);
    let fn_value = ctx.get_or_define_fn(&lir_body.metadata, &lir_body.ret_and_args);
    let entry_bb = B::append_basic_block(ctx, fn_value, "entry");
    let mut start_builder = B::build(ctx, entry_bb);

    let cached_bbs = lir_body
        .basic_blocks
        .indices()
        .map(|bb| {
            if bb == ENTRY_BLOCK {
                Some(entry_bb)
            } else {
                None
            }
        })
        .collect();

    let mut fn_ctx = FnCtx::<'_, '_, B> {
        fn_abi,
        lir_body,
        fn_value,
        ctx,
        locals: IdxVec::new(),
        cached_bbs,
    };

    let mut allocate_locals =
        |locals: &IdxVec<Local, LocalData>| -> IdxVec<Local, LocalRef<B::Value>> {
            let mut local_allocas = IdxVec::new();

            for (local, local_data) in locals.iter_enumerated() {
                debug!("Allocating local {:?} of type {:?}", local, local_data.ty);
                let layout = start_builder.ctx().layout_of(local_data.ty);

                // Check if the local has to be stored in memory or can be an operand.
                let local_ref = if layout.is_memory() {
                    LocalRef::PlaceRef(PlaceRef::alloca(&mut start_builder, layout))
                } else if layout.is_zst() {
                    // ZSTs do not need to be allocated.
                    LocalRef::OperandRef(OperandRef::new_zst(layout))
                } else {
                    LocalRef::PendingOperandRef
                };

                // let local_ref = LocalRef::PlaceRef(PlaceRef::alloca(&mut start_builder, layout));
                local_allocas.push(local_ref);
            }

            local_allocas
        };

    // Allocate the return value and arguments
    let mut locals = allocate_locals(&fn_ctx.lir_body.ret_and_args);
    // Allocate the locals
    locals.append(&mut allocate_locals(&fn_ctx.lir_body.locals));

    // Initialize the locals in the function context.
    fn_ctx.locals = locals;

    // We can safely drop the builder now, as we will create new builders for each basic block.
    drop(start_builder);

    // Codegen each basic block in the function body.
    for bb in lir_body.basic_blocks.indices() {
        fn_ctx.codegen_basic_block(bb);
        // TODO(bruzzone): consider to remove unreached blocks here
    }
}
