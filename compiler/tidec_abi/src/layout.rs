use crate::{
    size_and_align::{AbiAndPrefAlign, Size},
    target::AddressSpace,
};

/// Represents a type along with its size and alignment information.
///
/// This is commonly used during codegen and layout computation to reason about
/// how values should be represented in memory on the target platform.
pub struct TyAndLayout<T> {
    /// The type this layout refers to.
    ///
    /// This is usually a LIR type, but can be any type that has a size and alignment.
    pub ty: T,
    /// The layout information for the type, including size and alignment.
    pub layout: Layout,
}

/// Represents the layout of a type in the target architecture.
///
/// This struct contains the size, alignment, and backend representation
/// of a type, which is essential for code generation and memory layout decisions.
// TODO(bruzzone): Add fields and variants (tag union, struct, etc.).
pub struct Layout {
    /// The size of the type in bytes.
    pub size: Size,
    /// The ABI and preferred alignment of the type.
    pub align: AbiAndPrefAlign,
    /// `backend_repr` specifies how the value is represented to the codegen backend.
    ///
    /// This representation is independent of the type’s structural layout as described by
    /// `variants` and `fields`. For example, a type like `MyType<Result<isize, isize>>`
    /// may still use `ScalarPair` as its backend representation.
    ///
    /// Therefore, even when `backend_repr` is not `Memory`, you must still consider
    /// `fields` and `variants` to fully understand and access all parts of the layout.
    pub backend_repr: BackendRepr,
}

/// Represents how values are passed to the backend during code generation.
///
/// This is *not* the same as the platform's ABI (Application Binary Interface).
/// While the platform ABI may influence these choices, this enum primarily describes
/// the *syntactic form* used when emitting code — for example, whether a value is
/// passed as a scalar (like an integer or float) or as a memory reference.
///
/// Most codegen backends treat SSA values (e.g., scalars or vectors) differently from
/// values stored in memory. As a general rule, small values are best handled as scalars
/// or short vectors, while larger values are better represented as memory blobs.
///
/// Note: This representation does *not* guarantee how a value will be lowered to the
/// actual calling convention — that is determined separately by the ABI implementation.
pub enum BackendRepr {
    /// The value is represented as a scalar, such as an integer or float.
    Scalar(Primitive),
    /// Scalar pair, which is a pair of scalars. It is often used for
    /// returning multiple values from a function. This allows the backend to
    /// optimize the representation of multiple return values
    ScalarPair(Primitive, Primitive),
    /// The value is represented as a memory reference, such as a pointer or
    /// a reference to a struct or array.
    Memory,
}

pub enum Primitive {
    /// A signed integer type.
    I8,
    I16,
    I32,
    I64,
    I128,
    /// An unsigned integer type.
    U8,
    U16,
    U32,
    U64,
    U128,
    /// A floating-point type.
    F16,
    F32,
    F64,
    F128,
    /// A pointer type.
    Pointer(AddressSpace),
}

pub struct LayoutCtx {}

impl LayoutCtx {
    pub fn new() -> Self {
        LayoutCtx {}
    }

    /// Computes the layout for a given type.
    pub fn compute_layout<T>(&self, ty: T) -> TyAndLayout<T> {
        todo!()
    }
}
